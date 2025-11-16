use rustc_hash::FxHashMap;

pub type Key = Box<[u8]>;

pub struct IndexEntry {
    pub segment_id: u64,
    pub offset: u64,
}

pub struct Index {
    pub map: FxHashMap<Key, IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::with_capacity_and_hasher(1024, Default::default()),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
        }
    }
    pub fn insert(&mut self, key: Vec<u8>, entry: IndexEntry) {
        self.map.insert(key.into_boxed_slice(), entry);
    }
    pub fn remove(&mut self, key: &[u8]) {
        self.map.remove(key);
    }
    pub fn get(&self, key: &[u8]) -> Option<&IndexEntry> {
        self.map.get(key)
    }
}
