use crossbeam_skiplist::SkipMap;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    u64,
};

#[derive(Clone, PartialEq, Eq)]
pub struct VersionedKey {
    pub key: Vec<u8>,
    pub seq: u64,
}

impl Ord for VersionedKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key.cmp(&other.key) {
            std::cmp::Ordering::Equal => other.seq.cmp(&self.seq),
            ord => ord,
        }
    }
}

impl PartialOrd for VersionedKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// look up heirerchy:
/// memtable -> immutable memtable -> sst
pub struct Memtable {
    data: SkipMap<VersionedKey, Vec<u8>>,
    size_bytes: AtomicUsize,
}

impl Memtable {
    pub fn new() -> Self {
        Self {
            data: SkipMap::new(),
            size_bytes: AtomicUsize::new(0),
        }
    }

    pub fn put(&self, key: Vec<u8>, value: Vec<u8>, seq: u64) {
        let key_size = key.len();
        let val_size = value.len();

        let vk = VersionedKey { key, seq };

        self.size_bytes
            .fetch_add(key_size + val_size + 8, Ordering::Relaxed);

        self.data.insert(vk, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let range = self.data.range(
            VersionedKey {
                key: key.to_vec(),
                seq: u64::MAX,
            }..=VersionedKey {
                key: key.to_vec(),
                seq: 0,
            },
        );
        for entry in range {
            if entry.key().key == key {
                let val = entry.value();
                if val.is_empty() {
                    return None;
                }
                return Some(val.clone());
            } else {
                break;
            }
        }
        None
    }

    pub fn get_seq(&self, key: &[u8], snapshot_seq: u64) -> Option<Vec<u8>> {
        // For snapshot isolation, we need to find the latest version with seq < snapshot_seq
        // VersionedKey is ordered by (key ASC, seq DESC), so we iterate from highest seq down
        let range = self.data.range(
            VersionedKey{
                key: key.to_vec(),
                seq: u64::MAX,  // Start from highest possible sequence
            }..=VersionedKey{
                key: key.to_vec(),
                seq: 0,
            },
        );
        for entry in range {
            if entry.key().key == key {
                // Only return entries with seq < snapshot_seq (strict inequality for snapshot isolation)
                if entry.key().seq < snapshot_seq {
                    let val = entry.value();
                    if val.is_empty(){
                        return None;
                    }
                    return Some(val.clone());
                }
                // Continue searching for older versions
            } else {
                break;
            }
        }
        None
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

    pub fn iter(&self) -> impl Iterator<Item = (VersionedKey, Vec<u8>)> + '_ {
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
