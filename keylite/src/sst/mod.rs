pub mod bloom;
pub mod iterator;
pub mod reader;
pub mod writer;

use std::io;
use thiserror::Error;

pub use iterator::SSTIterator;
pub use reader::SSTReader;
pub use writer::SSTWriter;

pub const BLOCK_SIZE: usize = 16 * 1024;
pub const FOOTER_SIZE: usize = 36;
pub const MAGIC: u64 = 0x4B45594C54_u64;

#[derive(Debug, Error)]
pub enum SSTError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("corrupt SSTable")]
    Corrupt,
    #[error("invalid magic number")]
    InvalidMagic,
    #[error("key not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, SSTError>;

#[derive(Debug, Clone)]
pub struct Footer {
    pub magic: u64,
    pub version: u32,
    pub index_offset: u64,
    pub bloom_offset: u64,
    pub num_entries: u64,
}

#[derive(Debug, Clone)]
pub struct BlockIndex {
    pub first_key: Box<[u8]>,
    pub offset: u64,
}
