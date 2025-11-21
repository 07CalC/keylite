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

    // loads the next 16kB block from the file into the memory
    //
    // returns Result of bool
    // true when block loaded successfully
    // false if there are no more blocks left
    // and give corrupt error if the blocks crc32 doesn't match
    fn load_next_block(&mut self) -> Result<bool> {
        // no block left to load
        if self.block_idx >= self.reader.block_indexes.len() {
            return Ok(false);
        }

        let offset = self.reader.block_indexes[self.block_idx].offset;
        let mut pos = offset as usize;

        // read the first 4 bytes to get the block len
        let block_len =
            u32::from_le_bytes(self.reader.mmap[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;

        // load the block into memory from mmap
        self.current_block_data = self.reader.mmap[pos..pos + block_len].to_vec();
        pos += block_len;

        // verify the crc32
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

    // returns the next kv pair from the sstable
    // used to get all the kv pairs from a particular sst
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
                self.current_block_pos = self.current_block_data.len();
                continue;
            }

            // load the key_len, the first 2 bytes
            let key_len =
                u16::from_le_bytes(self.current_block_data[idx..idx + 2].try_into().unwrap())
                    as usize;

            // load the val_len, the next 4 bytes
            let val_len = u32::from_le_bytes(
                self.current_block_data[idx + 2..idx + 6]
                    .try_into()
                    .unwrap(),
            ) as usize;

            // load the actual key value
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
