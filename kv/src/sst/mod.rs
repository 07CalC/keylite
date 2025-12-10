// SSTables - String sorted tables
//
// format :
// ┌─────────────────────────────────────────┐
// │            data Blocks (N)              │
// │ each block:                             │
// │  block_len (u32)                        │
// │  entries:                               │
// │    key_len (u16)                        │
// │    val_len (u32)                        │
// │    key bytes                            │
// │    seq (u64)                            │
// │    value bytes                          │
// │  block_crc32 (u32)                      │
// ├─────────────────────────────────────────┤
// │               index Block               │
// │  num_entries (u32)                      │
// │  repeated: key_len | offset | key       │
// │  crc32                                  │
// ├─────────────────────────────────────────┤
// │              bloom Filter               │
// │  bloom_len (u32)                        │
// │  bloom_data[...]                        │
// │  crc32                                  │
// ├─────────────────────────────────────────┤
// │                 footer                  │
// │  magic (u64)                            │
// │  version (u32)                          │
// │  index_offset (u64)                     │
// │  bloom_offset (u64)                     │
// │  num_entries (u64)                      │
// │  max_sequence (u64)                     │
// └─────────────────────────────────────────┘
//
//

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
pub const FOOTER_SIZE: usize = 44;
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
    pub max_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct BlockIndex {
    pub first_key: Box<[u8]>,
    pub offset: u64,
}
