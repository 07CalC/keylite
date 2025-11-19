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

        let bits_len = (self.data.len() * 8) as u64;

        let bit_index1 = (hash1 % bits_len) as usize;
        let bit_index2 = (hash2 % bits_len) as usize;
        let bit_index3 = ((hash1.wrapping_add(hash2)) % bits_len) as usize;

        if (self.data[bit_index1 / 8] >> (bit_index1 % 8)) & 1 == 0 {
            return false;
        }

        if (self.data[bit_index2 / 8] >> (bit_index2 % 8)) & 1 == 0 {
            return false;
        }

        (self.data[bit_index3 / 8] >> (bit_index3 % 8)) & 1 == 1
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
