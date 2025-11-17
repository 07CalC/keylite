use std::collections::BTreeMap;

/// loop up heirerchy:
/// memtable -> immutable memtable -> sst
pub struct Memtable {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    size_bytes: usize,
}

impl Memtable {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
            size_bytes: 0,
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let key_size = key.len();
        let val_size = value.len();

        if let Some(old_val) = self.data.get(&key) {
            self.size_bytes -= old_val.len();
        } else {
            self.size_bytes += key_size;
        }
        self.size_bytes += val_size;

        self.data.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.data.iter().map(|(k, v)| (k.as_slice(), v.as_slice()))
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.size_bytes = 0;
    }
}

impl Default for Memtable {
    fn default() -> Self {
        Self::new()
    }
}
