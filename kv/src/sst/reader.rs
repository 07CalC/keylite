use crc32fast::Hasher;
use dashmap::DashMap;
use memmap2::Mmap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{bloom::BloomFilter, BlockIndex, Footer, Result, SSTError, FOOTER_SIZE, MAGIC};

pub struct SSTReader {
    path: PathBuf,
    pub(super) mmap: Mmap,
    pub(super) block_indexes: Arc<Vec<BlockIndex>>,
    bloom_filter: Arc<BloomFilter>,
    pub(super) block_cache: Arc<DashMap<u64, Arc<Vec<u8>>>>,
}

impl SSTReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // read footer from end of file
        // footer is of fixed size always
        // if size doesn't match means that SST is corrupted
        // even in the case where sst file is empty there must be footer present there
        // so if mmap.len() < FOOTER_SIZE hence the file is corrupted
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
            block_indexes: Arc::new(block_indexes),
            bloom_filter: Arc::new(bloom_filter),
            // TODO: make the block_cache a lru so that it doesn't grow indefinitly
            block_cache: Arc::new(DashMap::new()),
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

        let block_idx = match self
            .block_indexes
            .binary_search_by(|idx| idx.first_key.as_ref().cmp(key))
        {
            Ok(i) => i,
            Err(0) => return Ok(None),
            Err(i) => i - 1,
        };

        self.search_block(self.block_indexes[block_idx].offset, key)
    }

    fn search_block(&self, offset: u64, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let block_data = if let Some(cached) = self.block_cache.get(&offset) {
            Arc::clone(cached.value())
        } else {
            let mut pos = offset as usize;

            let block_len =
                u32::from_le_bytes(self.mmap[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;

            let block_data = &self.mmap[pos..pos + block_len];
            pos += block_len;

            let crc = u32::from_le_bytes(self.mmap[pos..pos + 4].try_into().unwrap());
            let mut hasher = Hasher::new();
            hasher.update(block_data);
            if hasher.finalize() != crc {
                return Err(SSTError::Corrupt);
            }

            let block_vec = Arc::new(block_data.to_vec());
            self.block_cache.insert(offset, Arc::clone(&block_vec));
            block_vec
        };

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
            let entry_key_start = idx;
            idx += key_len;

            let entry_val_start = idx;
            idx += val_len;

            entries.push((entry_key, entry_key_start, entry_val_start, val_len));
        }

        match entries.binary_search_by(|(entry_key, _, _, _)| entry_key.cmp(&key)) {
            Ok(pos) => {
                let (_, _, val_start, val_len) = entries[pos];
                let entry_val = &block_data[val_start..val_start + val_len];

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

impl Clone for SSTReader {
    fn clone(&self) -> Self {
        match Self::open(&self.path) {
            Ok(reader) => reader,
            Err(_) => {
                let file = File::open(&self.path).expect("Failed to reopen SST file");
                let mmap = unsafe { Mmap::map(&file).expect("Failed to mmap SST file") };

                Self {
                    path: self.path.clone(),
                    mmap,
                    block_indexes: Arc::clone(&self.block_indexes),
                    bloom_filter: Arc::clone(&self.bloom_filter),
                    block_cache: Arc::clone(&self.block_cache),
                }
            }
        }
    }
}
