// a simple bloom filter implementation used inside sstables for negative lookups
//
// this implementation uses:
// - 3 hash bit indexes (H1, H2, H1+H2)
// - crc32fast for hashing
// - simple bit vector as the bloom data

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

    // returns true if the key MIGHT be present in the SSTable
    //
    // example:
    //
    // lets say the BloomFiler data looks like:
    // [01101010, 10000001] // vector of u8 with 2 elements
    //
    // bits_len = 8*2 = 16 // we have that many bits
    //
    // let's say we look for the key "apple" and the h1, h2, h3 comes out to be 3, 6, and 16
    // respectively
    //
    // so the bit_index1 = 3 % 16 = 3,
    //
    // bit_index2 = 6 % 16 = 6
    //
    // bit_index3 = 16 % 16 = 0
    //
    // not let's check for bit_index1
    //
    // bit_index1 / 8 = 3/8 = 0 hence the first bit is present in the first byte (01101010),
    // self.data[0]
    //
    // bit_index1 % 8 = 3 % 8 = 3, hence the bit to check is the third bit in the first byte
    //
    // now if we do 01101010 >> 3, means right shift 3 times
    // so it becomes 00001101
    //
    // now it we take & with 1 it will result in 1, hence the bit_index1 is correct
    //
    // similarly check other 2 indexes and they'll also be 1 hence this sst MIGHT contain the given
    // key
    pub fn might_contain(&self, key: &[u8]) -> bool {
        let hash1 = self.hash_key(key, 0);
        let hash2 = self.hash_key(key, 1);

        // self.data is Vec<u8>
        // each byte has 8 bits
        // hence total bits_len = bytes * 8
        let bits_len = (self.data.len() * 8) as u64;

        // compute the 3 bit positions that has to be 1 for that particular key to exist
        let bit_index1 = (hash1 % bits_len) as usize;
        let bit_index2 = (hash2 % bits_len) as usize;
        let bit_index3 = ((hash1.wrapping_add(hash2)) % bits_len) as usize;

        // shift the targe bit to the LSB to check if it's 1 or not
        if (self.data[bit_index1 / 8] >> (bit_index1 % 8)) & 1 == 0 {
            return false;
        }

        if (self.data[bit_index2 / 8] >> (bit_index2 % 8)) & 1 == 0 {
            return false;
        }

        (self.data[bit_index3 / 8] >> (bit_index3 % 8)) & 1 == 1
    }

    // computes crc32 based hash for the given key using the given seed
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

// reads the bloom data from the SSTable using mmap
//
// the bloom filter data is stored in the file as follows:
//
// | bloom len | bloom data in bytes | crc32 |
// |  u32      |    [u8; len]        |  u32  |
//
// first read the bloom len which is 4 bytes long
// then read the bloom data which is len bytes long
// then read and verify the crc32 which is 4 bytes long
// if the crc32 doesn't match then we can tell that the bloom filter is corrupted
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
