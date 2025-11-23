//TODO: make these sizes to be configurable in the DB::open() method

pub const MEMTABLE_SIZE_THRESHOLD: usize = 12 * 1024 * 1024;
pub const MAX_SSTABLES: usize = 3;
pub const BLOCK_CACHE_CAPACITY: usize = 256;
