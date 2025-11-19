use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// look up heirerchy:
/// memtable -> immutable memtable -> sst
pub struct Memtable {
    data: DashMap<Vec<u8>, Vec<u8>>,
    size_bytes: AtomicUsize,
}

impl Memtable {
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
            size_bytes: AtomicUsize::new(0),
        }
    }

    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) {
        let key_size = key.len();
        let val_size = value.len();

        if let Some(old_val) = self.data.get(&key) {
            self.size_bytes.fetch_sub(old_val.len(), Ordering::Relaxed);
        } else {
            self.size_bytes.fetch_add(key_size, Ordering::Relaxed);
        }
        self.size_bytes.fetch_add(val_size, Ordering::Relaxed);

        self.data.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).map(|v| v.value().clone())
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes.load(Ordering::Relaxed)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)> + use<'_> {
        self.data
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
    }

    pub fn clear(&self) {
        self.data.clear();
        self.size_bytes.store(0, Ordering::Relaxed);
    }
}

impl Default for Memtable {
    fn default() -> Self {
        Self::new()
    }
}
