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

        let block_len =
            u32::from_le_bytes(self.reader.mmap[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        self.current_block_data = self.reader.mmap[pos..pos + block_len].to_vec();
        pos += block_len;

        let crc = u32::from_le_bytes(self.reader.mmap[pos..pos + 4].try_into().unwrap());
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
    type Item = Result<(Vec<u8>, Vec<u8>)>;

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
            if idx + 6 > self.current_block_data.len() {
                // Move to next block
                self.current_block_pos = self.current_block_data.len();
                continue;
            }

            let key_len =
                u16::from_le_bytes(self.current_block_data[idx..idx + 2].try_into().unwrap())
                    as usize;
            let val_len = u32::from_le_bytes(
                self.current_block_data[idx + 2..idx + 6]
                    .try_into()
                    .unwrap(),
            ) as usize;

            if idx + 6 + key_len + val_len > self.current_block_data.len() {
                self.current_block_pos = self.current_block_data.len();
                continue;
            }

            let key = self.current_block_data[idx + 6..idx + 6 + key_len].to_vec();
            let val =
                self.current_block_data[idx + 6 + key_len..idx + 6 + key_len + val_len].to_vec();

            self.current_block_pos = idx + 6 + key_len + val_len;

            return Some(Ok((key, val)));
        }
    }
}
