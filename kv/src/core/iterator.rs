// DbIterator performs a priority based k-way merge over multiple sources
// sources: mutable memtable (highest priority), immutable memtable(s) (second highest priority)
// and the SSTables (least priority)
//
use crate::{
    memtable::{skipmap::VersionedKey, Memtable},
    sst::{SSTIterator, SSTReader},
};
use std::{cmp::Ordering, collections::BinaryHeap, sync::Arc};

#[derive(Clone, Debug)]
pub struct IterEntry {
    key: Vec<u8>,
    value: Vec<u8>,
    seq: u64,
    priority: usize, // determines source order (mem → immut → sst)
}

// custom cmp implementation to give the reverse ordering
// used in the binary heap to get the entry with the smallest key on top rather than the biggest
// which is the default implementation of the BinaryHeap
impl std::cmp::Ord for IterEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // FIRST: smallest user-key wins in heap
        match other.key.cmp(&self.key) {
            Ordering::Equal => {
                // SECOND: newer seq wins among equal keys
                match self.seq.cmp(&other.seq) {
                    Ordering::Equal => self.priority.cmp(&other.priority),
                    ord => ord,
                }
            }
            ord => ord,
        }
    }
}

impl std::cmp::PartialOrd for IterEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for IterEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.seq == other.seq
    }
}

impl std::cmp::Eq for IterEntry {}

// we have two types of source for each entry
// it can be SSTable or Memtable (immutable or mutable)
// each source will be given some priority based on their age
// priority will be used to resolve duplicacies, data from coming from source of higher priority
// will be considered and the rest will be discarded
// i.e. highest priority will be of the mutable memtable which has the latest data
// second highest will be of immutable memtables -> newest will have more priority than the oldest
// least priority will be given to SST -> newest will have more than oldest
enum IterSource {
    Memtable {
        data: Vec<(VersionedKey, Vec<u8>)>,
        pos: usize,
        priority: usize,
    },
    Sst {
        // since we have already implemented a iterator for SST we can directly consume it here
        iter: SSTIterator,
        priority: usize,
    },
}

pub struct DbIterator {
    heap: BinaryHeap<IterEntry>,
    sources: Vec<IterSource>,
    last_key: Option<Vec<u8>>,
    start_bound: Option<Vec<u8>>,
    end_bound: Option<Vec<u8>>,
    max_seq: Option<u64>, // for snapshot isolation, we should only see the values with seq less
                          // than this
}

impl DbIterator {
    pub fn new(
        memtable: Arc<Memtable>,
        immutable_memtable: Vec<Arc<Memtable>>,
        sstables: Vec<SSTReader>,
        start_bound: Option<Vec<u8>>,
        end_bound: Option<Vec<u8>>,
    ) -> Self {
        Self::new_with_seq(
            memtable,
            immutable_memtable,
            sstables,
            start_bound,
            end_bound,
            None,
        )
    }

    pub fn new_with_seq(
        memtable: Arc<Memtable>,
        immutable_memtable: Vec<Arc<Memtable>>,
        sstables: Vec<SSTReader>,
        start_bound: Option<Vec<u8>>,
        end_bound: Option<Vec<u8>>,
        max_seq: Option<u64>,
    ) -> Self {
        let mut sources = Vec::new();
        let mut heap = BinaryHeap::new();

        // memtable priority will be highest
        // i.e. number of sstables + number of immutable memtables
        let memtable_priority = immutable_memtable.len() + sstables.len();

        // collect the data stored in the mutable memtable
        // iter is implemented for memtable, see /storage/memtable.rs
        // skipmap iterator is already sorted, no need to sort again
        let memtable_data: Vec<(VersionedKey, Vec<u8>)> = memtable.iter().collect();

        // add the memtable source in sources
        sources.push(IterSource::Memtable {
            data: memtable_data,
            pos: 0,
            priority: memtable_priority,
        });

        // collect the data stored in the immutable memtable(s)
        for (i, imt) in immutable_memtable.iter().enumerate() {
            let priority = memtable_priority - 1 - i;

            let imt_data: Vec<(VersionedKey, Vec<u8>)> = imt.iter().collect();

            // add immutable_memtables to sources
            sources.push(IterSource::Memtable {
                data: imt_data,
                pos: 0,
                priority,
            });
        }

        for (i, sst) in sstables.iter().enumerate() {
            let priority = if !immutable_memtable.is_empty() {
                immutable_memtable.len() - i - 1
            } else {
                memtable_priority - i - 1
            };
            let iter = SSTIterator::new(sst.clone());
            sources.push(IterSource::Sst { iter, priority });
        }

        // preload one entry from each source
        for i in 0..sources.len() {
            if let Some(entry) =
                Self::advance_source(&mut sources, i, &start_bound, &end_bound, &max_seq)
            {
                heap.push(entry);
            }
        }

        Self {
            heap,
            sources,
            last_key: None,
            start_bound,
            end_bound,
            max_seq,
        }
    }

    fn advance_source(
        sources: &mut [IterSource],
        source_idx: usize,
        start_bound: &Option<Vec<u8>>,
        end_bound: &Option<Vec<u8>>,
        max_seq: &Option<u64>,
    ) -> Option<IterEntry> {
        loop {
            let (key, value, seq, priority) = match &mut sources[source_idx] {
                IterSource::Memtable {
                    data,
                    pos,
                    priority,
                } => {
                    if *pos >= data.len() {
                        return None;
                    }
                    let (vk, v) = data[*pos].clone();
                    *pos += 1;

                    (vk.key, v, vk.seq, *priority)
                }
                IterSource::Sst { iter, priority } => match iter.next()? {
                    Ok((k, v, seq)) => (k, v, seq, *priority),
                    Err(_) => return None,
                },
            };

            // kkip entries with sequence numbers >= max_seq, for snapshot isolation
            // we use >= to match the strict inequality in get_seq (seq < snapshot_seq)
            if let Some(max) = max_seq {
                if seq >= *max {
                    continue;
                }
            }

            if let Some(start) = start_bound {
                if &key < start {
                    continue;
                }
            }

            if let Some(end) = end_bound {
                if &key >= end {
                    return None;
                }
            }

            return Some(IterEntry {
                key,
                value,
                seq,
                priority,
            });
        }
    }
}

impl Iterator for DbIterator {
    type Item = (Vec<u8>, Vec<u8>);

    // the next implementation for the DbIterator
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.heap.pop()?;

            // advance the source that produced this entry
            for idx in 0..self.sources.len() {
                let source_priority = match &self.sources[idx] {
                    IterSource::Memtable { priority, .. } => *priority,
                    IterSource::Sst { priority, .. } => *priority,
                };

                if source_priority == entry.priority {
                    if let Some(next_entry) = Self::advance_source(
                        &mut self.sources,
                        idx,
                        &self.start_bound,
                        &self.end_bound,
                        &self.max_seq,
                    ) {
                        self.heap.push(next_entry);
                    }
                    break;
                }
            }

            // if the entry's key is same as the last key, hence it's duplicate
            // and one key with same value has already been pushed into the Iterator
            if let Some(ref last) = self.last_key {
                if &entry.key == last {
                    continue;
                }
            }

            // change the last_key to the key we're about to return
            self.last_key = Some(entry.key.clone());

            // value is empty hence it's tombstoned
            if entry.value.is_empty() {
                continue;
            }

            return Some((entry.key, entry.value));
        }
    }
}
