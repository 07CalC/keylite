use crc32fast::Hasher;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::u64;

use super::{BlockIndex, Footer, BLOCK_SIZE, FOOTER_SIZE, MAGIC};

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub struct SSTWriter {
    file: BufWriter<File>,
    current_block: Vec<u8>,
    block_indexes: Vec<BlockIndex>,
    current_block_offset: u64,
    total_bytes_written: u64,
    num_entries: u64,
    bloom_filter: Vec<u8>,
    max_sequence: u64,
}

impl SSTWriter {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            file: BufWriter::new(file),
            current_block: Vec::new(),
            block_indexes: Vec::new(),
            current_block_offset: 0,
            total_bytes_written: 0,
            num_entries: 0,
            bloom_filter: vec![0u8; 16384],
            max_sequence: u64::MIN,
        })
    }

    pub fn add(&mut self, key: &[u8], value: &[u8], seq: u64) -> Result<()> {
        if self.current_block.is_empty() {
            self.block_indexes.push(BlockIndex {
                first_key: key.to_vec().into_boxed_slice(),
                offset: self.current_block_offset,
            });
        }

        self.add_to_bloom_filter(key);

        self.current_block
            .extend_from_slice(&(key.len() as u16).to_le_bytes());
        self.current_block
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.current_block.extend_from_slice(key);
        self.current_block.extend_from_slice(&(seq).to_le_bytes());
        self.current_block.extend_from_slice(value);

        self.num_entries += 1;
        self.max_sequence = self.max_sequence.max(seq);

        if self.current_block.len() >= BLOCK_SIZE {
            self.flush_block()?;
        }

        Ok(())
    }

    fn add_to_bloom_filter(&mut self, key: &[u8]) {
        let hash1 = self.hash_key(key, 0);
        let hash2 = self.hash_key(key, 1);

        let bits_len = (self.bloom_filter.len() * 8) as u64;
        let bit_index1 = (hash1 % bits_len) as usize;
        let bit_index2 = (hash2 % bits_len) as usize;
        let bit_index3 = ((hash1.wrapping_add(hash2)) % bits_len) as usize;

        self.bloom_filter[bit_index1 / 8] |= 1 << (bit_index1 % 8);
        self.bloom_filter[bit_index2 / 8] |= 1 << (bit_index2 % 8);
        self.bloom_filter[bit_index3 / 8] |= 1 << (bit_index3 % 8);
    }

    fn hash_key(&self, key: &[u8], seed: u32) -> u64 {
        let mut hasher = Hasher::new();
        hasher.update(&seed.to_le_bytes());
        hasher.update(key);
        hasher.finalize() as u64
    }

    fn flush_block(&mut self) -> Result<()> {
        if self.current_block.is_empty() {
            return Ok(());
        }

        let mut hasher = Hasher::new();
        hasher.update(&self.current_block);
        let crc = hasher.finalize();

        self.file
            .write_all(&(self.current_block.len() as u32).to_le_bytes())?;
        self.file.write_all(&self.current_block)?;
        self.file.write_all(&crc.to_le_bytes())?;

        let block_total_size = 4 + self.current_block.len() + 4;
        self.total_bytes_written += block_total_size as u64;
        self.current_block_offset = self.total_bytes_written;

        self.current_block.clear();

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.flush_block()?;

        let index_offset = self.total_bytes_written;
        let mut index_block = Vec::new();

        index_block.extend_from_slice(&(self.block_indexes.len() as u32).to_le_bytes());

        for idx in &self.block_indexes {
            index_block.extend_from_slice(&(idx.first_key.len() as u16).to_le_bytes());
            index_block.extend_from_slice(&idx.offset.to_le_bytes());
            index_block.extend_from_slice(&idx.first_key);
        }

        let mut hasher = Hasher::new();
        hasher.update(&index_block);
        let index_crc = hasher.finalize();

        self.file
            .write_all(&(index_block.len() as u32).to_le_bytes())?;
        self.file.write_all(&index_block)?;
        self.file.write_all(&index_crc.to_le_bytes())?;

        self.total_bytes_written += 4 + index_block.len() as u64 + 4;

        let bloom_offset = self.total_bytes_written;
        let mut hasher = Hasher::new();
        hasher.update(&self.bloom_filter);
        let bloom_crc = hasher.finalize();

        self.file
            .write_all(&(self.bloom_filter.len() as u32).to_le_bytes())?;
        self.file.write_all(&self.bloom_filter)?;
        self.file.write_all(&bloom_crc.to_le_bytes())?;

        self.total_bytes_written += 4 + self.bloom_filter.len() as u64 + 4;

        let footer = Footer {
            magic: MAGIC,
            version: 1,
            index_offset,
            bloom_offset,
            num_entries: self.num_entries,
            max_sequence: self.max_sequence,
        };

        let mut footer_bytes = [0u8; FOOTER_SIZE];
        footer_bytes[0..8].copy_from_slice(&footer.magic.to_le_bytes());
        footer_bytes[8..12].copy_from_slice(&footer.version.to_le_bytes());
        footer_bytes[12..20].copy_from_slice(&footer.index_offset.to_le_bytes());
        footer_bytes[20..28].copy_from_slice(&footer.bloom_offset.to_le_bytes());
        footer_bytes[28..36].copy_from_slice(&footer.num_entries.to_le_bytes());
        footer_bytes[36..44].copy_from_slice(&footer.max_sequence.to_le_bytes());

        self.file.write_all(&footer_bytes)?;
        self.file.flush()?;

        Ok(())
    }
}
