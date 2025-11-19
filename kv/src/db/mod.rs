pub mod config;
mod core;
mod iterator;

pub use config::{MAX_SSTABLES, MEMTABLE_SIZE_THRESHOLD};
pub use core::Db;
pub use iterator::DbIterator;

pub type Result<T> = std::result::Result<T, crate::error::DbError>;
