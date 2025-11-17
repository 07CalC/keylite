use crc32fast::Hasher;
use memmap2::Mmap;

use super::{Result, SSTError};

pub struct BloomFilter {
    data: Vec<u8>,
}

impl BloomFilter {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn might_contain(&self, key: &[u8]) -> bool {
        let hash1 = self.hash_key(key, 0);
        let hash2 = self.hash_key(key, 1);
        let hash3 = self.hash_key(key, 2);

        let bit_index1 = (hash1 % (self.data.len() * 8) as u64) as usize;
        let bit_index2 = (hash2 % (self.data.len() * 8) as u64) as usize;
        let bit_index3 = (hash3 % (self.data.len() * 8) as u64) as usize;

        let bit1 = (self.data[bit_index1 / 8] >> (bit_index1 % 8)) & 1;
        let bit2 = (self.data[bit_index2 / 8] >> (bit_index2 % 8)) & 1;
        let bit3 = (self.data[bit_index3 / 8] >> (bit_index3 % 8)) & 1;

        bit1 == 1 && bit2 == 1 && bit3 == 1
    }

    fn hash_key(&self, key: &[u8], seed: u32) -> u64 {
        let mut hasher = Hasher::new();
        hasher.update(&seed.to_le_bytes());
        hasher.update(key);
        hasher.finalize() as u64
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

pub fn read_bloom_filter(mmap: &Mmap, offset: u64) -> Result<BloomFilter> {
    let mut pos = offset as usize;

    let bloom_len = u32::from_le_bytes(mmap[pos..pos + 4].try_into().unwrap()) as usize;
    pos += 4;

    let bloom_data = &mmap[pos..pos + bloom_len];
    pos += bloom_len;

    let crc = u32::from_le_bytes(mmap[pos..pos + 4].try_into().unwrap());
    let mut hasher = Hasher::new();
    hasher.update(bloom_data);
    if hasher.finalize() != crc {
        return Err(SSTError::Corrupt);
    }

    Ok(BloomFilter::new(bloom_data.to_vec()))
}
