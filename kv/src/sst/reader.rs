use crc32fast::Hasher;
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
    min_sequence: u64,
    max_sequence: u64,
}

impl SSTReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // sanity check: file must at least contain a footer
        if mmap.len() < FOOTER_SIZE {
            return Err(SSTError::Corrupt);
        }

        let footer_bytes = &mmap[mmap.len() - FOOTER_SIZE..];
        let footer = Self::parse_footer(footer_bytes)?;

        let block_indexes = Self::read_index_block(&mmap, footer.index_offset)?;
        let bloom_filter = super::bloom::read_bloom_filter(&mmap, footer.bloom_offset)?;
        let min_sequence = footer.min_sequence;
        let max_sequence = footer.max_sequence;

        Ok(Self {
            path,
            mmap,
            block_indexes: Arc::new(block_indexes),
            bloom_filter: Arc::new(bloom_filter),
            min_sequence,
            max_sequence,
        })
    }

    fn parse_footer(bytes: &[u8]) -> Result<Footer> {
        // footer layout (must match writer):
        // 0..8    magic (u64) = "KEYLT"
        // 8..12   version (u32)
        // 12..20  index_offset (u64)
        // 20..28  bloom_offset (u64)
        // 28..36  num_entries (u64)
        // 36..44  min_sequence (u64)
        // 44..52 max_sequence (u64)
        debug_assert!(bytes.len() == FOOTER_SIZE);

        let magic = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        if magic != MAGIC {
            return Err(SSTError::InvalidMagic);
        }

        let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let index_offset = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
        let bloom_offset = u64::from_le_bytes(bytes[20..28].try_into().unwrap());
        let num_entries = u64::from_le_bytes(bytes[28..36].try_into().unwrap());
        let min_sequence = u64::from_le_bytes(bytes[36..44].try_into().unwrap());
        let max_sequence = u64::from_le_bytes(bytes[44..52].try_into().unwrap());

        Ok(Footer {
            magic,
            version,
            index_offset,
            bloom_offset,
            num_entries,
            min_sequence,
            max_sequence,
        })
    }

    fn read_index_block(mmap: &Mmap, offset: u64) -> Result<Vec<BlockIndex>> {
        let mut pos = offset as usize;

        // index block is stored as:
        // [block_len: u32][block_data...][crc32: u32]
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

        // Index block format:
        // num_entries (u32)
        // repeated:
        //   key_len (u16)
        //   offset (u64)
        //   first_key bytes
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

    /// simple point lookup (no version merging yet).
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // fast negative path via bloom filter
        if !self.bloom_filter.might_contain(key) {
            return Ok(None);
        }

        // find the block whose first_key <= key
        let block_idx = match self
            .block_indexes
            .binary_search_by(|idx| idx.first_key.as_ref().cmp(key))
        {
            Ok(i) => i,
            Err(0) => return Ok(None),
            Err(i) => i - 1,
        };

        // IMPORTANT: Because block boundaries are determined by size (not by key changes),
        // entries for the same key with different sequence numbers can span multiple blocks.
        // The entry with the highest sequence number will appear first in the file.
        //
        // Simple solution: Check the found block and the previous block (if it exists).
        // This handles the common case where a key's versions span at most 2 blocks.

        let start_idx = if block_idx > 0 { block_idx - 1 } else { 0 };

        // Search from start_idx to block_idx, returning the FIRST result found.
        // IMPORTANT: If we find a tombstone (Ok(None)), we must return it immediately,
        // not continue searching, because the tombstone has the highest sequence number
        // and represents the most recent state of the key.
        for idx in start_idx..=block_idx {
            match self.search_block(self.block_indexes[idx].offset, key) {
                Ok(val) => {
                    // Found the key (either with a value or as a tombstone)
                    return Ok(val);
                }
                Err(super::SSTError::NotFound) => {
                    // Key not in this block, continue to next block
                    continue;
                }
                Err(e) => {
                    // Other error
                    return Err(e);
                }
            }
        }

        Ok(None)
    }

    pub fn get_seq(&self, key: &[u8], snapshot_seq: u64) -> Result<Option<Vec<u8>>> {
        // Quick check: if snapshot is before this SST's min sequence, no data visible
        if snapshot_seq <= self.min_sequence {
            return Ok(None);
        }

        if !self.bloom_filter.might_contain(key) {
            return Ok(None);
        }

        // If snapshot is after all data in this SST, use regular get
        if snapshot_seq > self.max_sequence {
            return self.get(key);
        }

        let block_idx = match self
            .block_indexes
            .binary_search_by(|idx| idx.first_key.as_ref().cmp(key))
        {
            Ok(i) => i,
            Err(0) => return Ok(None),
            Err(i) => i - 1,
        };

        let stard_idx = if block_idx > 0 { block_idx - 1 } else { 0 };

        let mut best: Option<(u64, Option<Vec<u8>>)> = None;

        for idx in stard_idx..=block_idx {
            if let Some((ent_seq, ent_val)) =
                self.search_block_seq(self.block_indexes[idx].offset, key, snapshot_seq)?
            {
                if best.is_none() || ent_seq > best.as_ref().unwrap().0 {
                    best = Some((ent_seq, ent_val));
                }
            }
        }
        match best {
            Some((_, Some(v))) => Ok(Some(v)),
            Some((_, None)) => Ok(None),
            None => Ok(None),
        }
    }

    /// search for a key within a specific block.
    ///
    /// returns:
    /// - `Ok(Some(val))` if key found with a non-empty value
    /// - `Ok(None)` if key found with empty value (tombstone/deleted)
    /// - `Err(SSTError::NotFound)` if key not in this block
    /// - `Err(other)` for other errors
    fn search_block(&self, offset: u64, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // read and verify block at given offset:
        // [block_len: u32][block_data...][crc32: u32]
        let block_data = {
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

            Arc::new(block_data.to_vec())
        };

        // entry format in block_data (must match writer) :
        // key_len (u16)
        // val_len (u32)
        // key [key_len]
        // seq (u64)
        // value [val_len]

        #[derive(Debug)]
        struct EntryMeta {
            key_start: usize,
            key_len: usize,
            val_start: usize,
            val_len: usize,
        }

        let mut entries: Vec<EntryMeta> = Vec::new();
        let mut idx = 0;
        let len = block_data.len();

        while idx < len {
            // need at least key_len + val_len
            if idx + 6 > len {
                break;
            }

            let key_len = u16::from_le_bytes(block_data[idx..idx + 2].try_into().unwrap()) as usize;
            idx += 2;

            let val_len = u32::from_le_bytes(block_data[idx..idx + 4].try_into().unwrap()) as usize;
            idx += 4;

            // now we need: key_len + 8(seq) + val_len bytes
            if idx + key_len + 8 + val_len > len {
                break;
            }

            let key_start = idx;
            idx += key_len;

            // nead seq but we don't use it yet in get()
            // let seq = u64::from_le_bytes(block_data[idx..idx + 8].try_into().unwrap());
            idx += 8;

            let val_start = idx;
            idx += val_len;

            entries.push(EntryMeta {
                key_start,
                key_len,
                val_start,
                val_len,
            });
        }

        // binary search over keys inside the block, then find the entry with highest seq
        match entries.binary_search_by(|entry| {
            let entry_key = &block_data[entry.key_start..entry.key_start + entry.key_len];
            entry_key.cmp(key)
        }) {
            Ok(mut pos) => {
                // found a match, but there might be multiple entries for the same key
                // with different sequence numbers within this block, find the one with the
                // highest sequence number
                // entries are sorted by key first, then by sequence (descending), so we need
                // to find the first occurrence of this key (which has the highest seq).

                // move backward to find the first occurrence of this key
                while pos > 0 {
                    let prev_entry = &entries[pos - 1];
                    let prev_key = &block_data
                        [prev_entry.key_start..prev_entry.key_start + prev_entry.key_len];
                    if prev_key == key {
                        pos -= 1;
                    } else {
                        break;
                    }
                }

                let entry = &entries[pos];
                let entry_val = &block_data[entry.val_start..entry.val_start + entry.val_len];

                // tombstone: empty value represents deletion
                if entry_val.is_empty() {
                    return Ok(None);
                }

                Ok(Some(entry_val.to_vec()))
            }
            Err(_) => Err(super::SSTError::NotFound),
        }
    }

    fn search_block_seq(
        &self,
        offset: u64,
        key: &[u8],
        snapshot_seq: u64,
    ) -> Result<Option<(u64, Option<Vec<u8>>)>> {
        let block_data = {
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

            block_data
        };

        #[derive(Clone, Debug)]
        struct Entry {
            seq: u64,
            key_start: usize,
            key_len: usize,
            val_start: usize,
            val_len: usize,
        }

        let mut entries = Vec::new();
        let mut idx = 0;
        let len = block_data.len();

        while idx < len {
            if idx + 6 > len {
                break;
            }

            let key_len = u16::from_le_bytes(block_data[idx..idx + 2].try_into().unwrap()) as usize;
            idx += 2;

            let val_len = u32::from_le_bytes(block_data[idx..idx + 4].try_into().unwrap()) as usize;
            idx += 4;

            if idx + key_len + 8 + val_len > len {
                break;
            }

            let key_start = idx;
            idx += key_len;

            let seq = u64::from_le_bytes(block_data[idx..idx + 8].try_into().unwrap());
            idx += 8;

            let val_start = idx;
            idx += val_len;

            entries.push(Entry {
                seq,
                key_start,
                key_len,
                val_start,
                val_len,
            });
        }

        // binary search for key
        let pos = match entries.binary_search_by(|e| {
            let entry_key = &block_data[e.key_start..e.key_start + e.key_len];
            entry_key.cmp(key)
        }) {
            Ok(pos) => pos,
            Err(_) => return Ok(None),
        };

        let mut first = pos;
        while first > 0 {
            let pk = &block_data[entries[first - 1].key_start
                ..entries[first - 1].key_start + entries[first - 1].key_len];
            if pk == key {
                first -= 1;
            } else {
                break;
            }
        }

        // iterate entries for this key for seq to be in descending order
        let mut i = first;
        while i < entries.len() {
            let e = &entries[i];
            let entry_key = &block_data[e.key_start..e.key_start + e.key_len];
            if entry_key != key {
                break;
            }

            // for snapshot isolation
            // only return entries with seq < snapshot_seq (strict inequality)
            if e.seq < snapshot_seq {
                let val_slice = &block_data[e.val_start..e.val_start + e.val_len];
                let val_opt = if val_slice.is_empty() {
                    None
                } else {
                    Some(val_slice.to_vec())
                };
                return Ok(Some((e.seq, val_opt)));
            }

            i += 1;
        }

        Ok(None)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn min_sequence(&self) -> u64 {
        self.min_sequence
    }
    pub fn max_sequence(&self) -> u64 {
        self.max_sequence
    }
}

impl Clone for SSTReader {
    fn clone(&self) -> Self {
        // cheap clone: reuse the same mmap and indexes via Arc
        Self {
            path: self.path.clone(),
            mmap: unsafe {
                // SAFETY: original file is still open by OS, mmap is read-only
                let file = File::open(&self.path).expect("Failed to reopen SST file");
                Mmap::map(&file).expect("Failed to mmap SST file")
            },
            block_indexes: Arc::clone(&self.block_indexes),
            bloom_filter: Arc::clone(&self.bloom_filter),
            min_sequence: self.min_sequence,
            max_sequence: self.max_sequence,
        }
    }
}
