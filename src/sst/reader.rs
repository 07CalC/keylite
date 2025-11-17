use crc32fast::Hasher;
use memmap2::Mmap;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{
    bloom::BloomFilter, BlockIndex, Footer, Result, SSTError, BLOCK_CACHE_SIZE, FOOTER_SIZE, MAGIC,
};

#[derive(Clone)]
pub(super) struct CachedBlock {
    pub data: Arc<Vec<u8>>,
    pub access_count: u64,
}

pub(super) struct LRUCache {
    map: HashMap<u64, CachedBlock>,
    global_counter: u64,
}

impl LRUCache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            global_counter: 0,
        }
    }

    fn get(&mut self, key: &u64) -> Option<CachedBlock> {
        if let Some(block) = self.map.get_mut(key) {
            self.global_counter += 1;
            block.access_count = self.global_counter;
            Some(block.clone())
        } else {
            None
        }
    }

    fn insert(&mut self, key: u64, data: Arc<Vec<u8>>) {
        self.global_counter += 1;

        if self.map.len() >= BLOCK_CACHE_SIZE {
            if let Some((&lru_key, _)) = self.map.iter().min_by_key(|(_, block)| block.access_count)
            {
                self.map.remove(&lru_key);
            }
        }

        self.map.insert(
            key,
            CachedBlock {
                data,
                access_count: self.global_counter,
            },
        );
    }
}

pub struct SSTReader {
    path: PathBuf,
    pub(super) mmap: Mmap,
    pub(super) block_indexes: Vec<BlockIndex>,
    bloom_filter: BloomFilter,
    pub(super) block_cache: Mutex<LRUCache>,
}

impl SSTReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // read footer from end of file
        // footer is of fixed size always
        // if size doesn't match means that SST is corrupted
        // TODO: what to do with corrupted SSTs?
        if mmap.len() < FOOTER_SIZE {
            return Err(SSTError::Corrupt);
        }

        let footer_bytes = &mmap[mmap.len() - FOOTER_SIZE..];
        let footer = Self::parse_footer(footer_bytes)?;

        let block_indexes = Self::read_index_block(&mmap, footer.index_offset)?;

        let bloom_filter = super::bloom::read_bloom_filter(&mmap, footer.bloom_offset)?;

        Ok(Self {
            path,
            mmap,
            block_indexes,
            bloom_filter,
            block_cache: Mutex::new(LRUCache::new()),
        })
    }

    fn parse_footer(bytes: &[u8]) -> Result<Footer> {
        let magic = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        if magic != MAGIC {
            return Err(SSTError::InvalidMagic);
        }

        let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let index_offset = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
        let bloom_offset = u64::from_le_bytes(bytes[20..28].try_into().unwrap());
        let num_entries = u64::from_le_bytes(bytes[28..36].try_into().unwrap());

        Ok(Footer {
            magic,
            version,
            index_offset,
            bloom_offset,
            num_entries,
        })
    }

    fn read_index_block(mmap: &Mmap, offset: u64) -> Result<Vec<BlockIndex>> {
        let mut pos = offset as usize;

        let block_len = u32::from_le_bytes(mmap[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        let block_data = &mmap[pos..pos + block_len];
        pos += block_len;

        let crc = u32::from_le_bytes(mmap[pos..pos + 4].try_into().unwrap());
        let mut hasher = Hasher::new();
        hasher.update(block_data);
        if hasher.finalize() != crc {
            return Err(SSTError::Corrupt);
        }

        let mut indexes = Vec::new();
        let num_entries = u32::from_le_bytes(block_data[0..4].try_into().unwrap()) as usize;
        let mut idx = 4;

        for _ in 0..num_entries {
            let key_len = u16::from_le_bytes(block_data[idx..idx + 2].try_into().unwrap()) as usize;
            idx += 2;

            let offset = u64::from_le_bytes(block_data[idx..idx + 8].try_into().unwrap());
            idx += 8;

            let key = block_data[idx..idx + key_len].to_vec().into_boxed_slice();
            idx += key_len;

            indexes.push(BlockIndex {
                first_key: key,
                offset,
            });
        }

        Ok(indexes)
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if !self.bloom_filter.might_contain(key) {
            return Ok(None);
        }

        // Binary search on sparse index to find the block
        // Find the rightmost block whose first_key <= key
        let block_idx = match self.block_indexes.binary_search_by(|idx| {
            if idx.first_key.as_ref() <= key {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        }) {
            Ok(_) => unreachable!(),   // We never return Equal
            Err(0) => return Ok(None), // key is before first block
            Err(i) => i - 1,           // The block to search
        };

        // Read and search the data block
        self.search_block(self.block_indexes[block_idx].offset, key)
    }

    fn search_block(&self, offset: u64, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Try to get from cache first
        let cached_block = {
            let mut cache = self.block_cache.lock();
            cache.get(&offset)
        };

        let block_data = if let Some(cached) = cached_block {
            // Use cached block (already CRC verified)
            cached.data
        } else {
            // Load and verify block from disk
            let mut pos = offset as usize;

            // Read block length
            let block_len =
                u32::from_le_bytes(self.mmap[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;

            let block_data = &self.mmap[pos..pos + block_len];
            pos += block_len;

            // Verify CRC
            let crc = u32::from_le_bytes(self.mmap[pos..pos + 4].try_into().unwrap());
            let mut hasher = Hasher::new();
            hasher.update(block_data);
            if hasher.finalize() != crc {
                return Err(SSTError::Corrupt);
            }

            // Cache the block with LRU eviction
            let block_vec = Arc::new(block_data.to_vec());
            let mut cache = self.block_cache.lock();
            cache.insert(offset, block_vec.clone());
            block_vec
        };

        // Parse all entries in block first (for binary search)
        let mut entries = Vec::new();
        let mut idx = 0;

        while idx < block_data.len() {
            if idx + 6 > block_data.len() {
                break;
            }

            let key_len = u16::from_le_bytes(block_data[idx..idx + 2].try_into().unwrap()) as usize;
            idx += 2;

            let val_len = u32::from_le_bytes(block_data[idx..idx + 4].try_into().unwrap()) as usize;
            idx += 4;

            if idx + key_len + val_len > block_data.len() {
                break;
            }

            let entry_key = &block_data[idx..idx + key_len];
            idx += key_len;

            let entry_val = &block_data[idx..idx + val_len];
            idx += val_len;

            entries.push((entry_key, entry_val));
        }

        // Binary search on sorted entries
        match entries.binary_search_by(|(entry_key, _)| entry_key.cmp(&key)) {
            Ok(pos) => {
                let (_, entry_val) = entries[pos];
                // Empty value means deletion tombstone
                if entry_val.is_empty() {
                    return Ok(None);
                }
                Ok(Some(entry_val.to_vec()))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
