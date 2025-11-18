pub mod config;
mod core;

pub use config::{MAX_SSTABLES, MEMTABLE_SIZE_THRESHOLD};
pub use core::Db;

pub type Result<T> = std::result::Result<T, crate::error::DbError>;
