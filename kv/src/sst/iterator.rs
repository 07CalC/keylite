// iterator implementation for for SSTables which will return key/val pairs
// SSTIterator provides a forward only iterator over a single sstable file
// the sst file is stored memory mapped and further divided into blocks currently each block is of
// size 16 kB
//
// each block look something like this:
//
// | block len (u32) | block_data (len bytes) | block crc32 (u32) |
//
// the block_data looks something like this:
//
// | key len (u16) | val len (u32) | key (key_len bytes) | val (val_len bytes) |
//

use crc32fast::Hasher;

use super::{Result, SSTError, SSTReader};

pub struct SSTIterator {
    reader: SSTReader,
    block_idx: usize,
    current_block_data: Vec<u8>,
    current_block_pos: usize,
}

impl SSTIterator {
    pub fn new(reader: SSTReader) -> Self {
        Self {
            reader,
            block_idx: 0,
            current_block_data: Vec::new(),
            current_block_pos: 0,
        }
    }

    fn load_next_block(&mut self) -> Result<bool> {
        if self.block_idx >= self.reader.block_indexes.len() {
            return Ok(false);
        }

        let offset = self.reader.block_indexes[self.block_idx].offset;
        let mut pos = offset as usize;

        let block_len = super::to_u32(&self.reader.mmap[pos..pos + 4])? as usize;
        pos += 4;

        self.current_block_data = self.reader.mmap[pos..pos + block_len].to_vec();
        pos += block_len;

        let crc = super::to_u32(&self.reader.mmap[pos..pos + 4])?;
        let mut hasher = Hasher::new();
        hasher.update(&self.current_block_data);
        if hasher.finalize() != crc {
            return Err(SSTError::Corrupt);
        }

        self.block_idx += 1;
        self.current_block_pos = 0;

        Ok(true)
    }
}

impl Iterator for SSTIterator {
    type Item = Result<(Vec<u8>, Vec<u8>, u64)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_block_pos >= self.current_block_data.len() {
                match self.load_next_block() {
                    Ok(true) => continue,
                    Ok(false) => return None,
                    Err(e) => return Some(Err(e)),
                }
            }

            let idx = self.current_block_pos;
            let data = &self.current_block_data;

            if idx + 6 > data.len() {
                self.current_block_pos = data.len();
                continue;
            }

            let key_len = match super::to_u16(&data[idx..idx + 2]) {
                Ok(len) => len as usize,
                Err(e) => return Some(Err(e)),
            };

            let val_len = match super::to_u32(&data[idx + 2..idx + 6]) {
                Ok(len) => len as usize,
                Err(e) => return Some(Err(e)),
            };

            let key_start = idx + 6;
            let seq_start = key_start + key_len;
            let val_start = seq_start + 8;

            if val_start + val_len > data.len() {
                self.current_block_pos = data.len();
                continue;
            }

            let key = data[key_start..key_start + key_len].to_vec();

            let seq = match super::to_u64(&data[seq_start..seq_start + 8]) {
                Ok(s) => s,
                Err(e) => return Some(Err(e)),
            };

            let value = data[val_start..val_start + val_len].to_vec();

            self.current_block_pos = val_start + val_len;

            return Some(Ok((key, value, seq)));
        }
    }
}
