// DbIterator performs a priority based k-way merge over multiple sources
// sources: mutable memtable (highest priority), immutable memtable(s) (second highest priority)
// and the SSTables (least priority)
//
use crate::{
    sst::{SSTIterator, SSTReader},
    storage::Memtable,
};
use std::{cmp::Ordering, collections::BinaryHeap, sync::Arc};
#[derive(Clone, Debug)]
pub struct IterEntry {
    key: Vec<u8>,
    value: Vec<u8>,
    priority: usize,
}

// custom cmp implementation to give the reverse ordering
// used in the binary heap to get the entry with the smallest key on top rather than the biggest
// which is the default implementation of the BinaryHeap
impl std::cmp::Ord for IterEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.key.cmp(&self.key) {
            Ordering::Equal => self.priority.cmp(&other.priority),
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
        self.key == other.key
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
        data: Vec<(Vec<u8>, Vec<u8>)>,
        pos: usize,
        priority: usize,
    },
    SST {
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
}

impl DbIterator {
    pub fn new(
        memtable: Arc<Memtable>,
        immutable_memtable: Vec<Arc<Memtable>>,
        sstables: Vec<SSTReader>,
        start_bound: Option<Vec<u8>>,
        end_bound: Option<Vec<u8>>,
    ) -> Self {
        let mut sources = Vec::new();
        let mut heap = BinaryHeap::new();

        // memtable priority will be highest
        // i.e. number of sstables + number of immutable memtables
        let memtable_priority = immutable_memtable.len() + sstables.len();

        // collect the data stored in the mutable memtable
        // iter is implemented for memtable, see /storage/memtable.rs
        let mut memtable_data: Vec<(Vec<u8>, Vec<u8>)> = memtable.iter().collect();
        memtable_data.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        // add the memtable source in sources
        sources.push(IterSource::Memtable {
            data: memtable_data,
            pos: 0,
            priority: memtable_priority,
        });

        // collect the data stored in the immutable memtable(s)
        for (i, imt) in immutable_memtable.iter().enumerate() {
            let priority = memtable_priority - 1 - i;
            let mut imt_data: Vec<(Vec<u8>, Vec<u8>)> = imt.iter().collect();
            imt_data.sort_unstable_by(|a, b| a.0.cmp(&b.0));

            // add immutable_memtables to sources
            sources.push(IterSource::Memtable {
                data: imt_data,
                pos: 0,
                priority,
            });
        }

        for (i, sst) in sstables.iter().enumerate() {
            let priority = if immutable_memtable.len() > 0 {
                immutable_memtable.len() - i - 1
            } else {
                memtable_priority - i - 1
            };
            let iter = SSTIterator::new(sst.clone()).with_start_bound(start_bound.clone());
            sources.push(IterSource::SST { iter, priority });
        }

        for i in 0..sources.len() {
            // put one entry from each source into the heap for k-based merge and advance that
            // source
            if let Some(entry) = Self::advance_source(&mut sources, i, &start_bound, &end_bound) {
                heap.push(entry);
            }
        }

        Self {
            heap,
            sources,
            last_key: None,
            start_bound,
            end_bound,
        }
    }

    fn advance_source(
        sources: &mut [IterSource],
        source_idx: usize,
        start_bound: &Option<Vec<u8>>,
        end_bound: &Option<Vec<u8>>,
    ) -> Option<IterEntry> {
        loop {
            // get the key, value, priority if the IterSource is Memtable, because an iterator
            // is not implemented for memtables
            let (key, value, priority) = match &mut sources[source_idx] {
                IterSource::Memtable {
                    data,
                    pos,
                    priority,
                } => {
                    // if pos is greater than the data's length hence we've reached the end of data
                    // present in that particular source
                    if *pos >= data.len() {
                        return None;
                    }

                    if let Some(start) = start_bound {
                        if *pos == 0 {
                            match data.binary_search_by(|(k, _)| k.as_slice().cmp(start)) {
                                Ok(idx) => *pos = idx,
                                Err(idx) => *pos = idx,
                            }
                            if *pos >= data.len() {
                                return None;
                            }
                        }
                    }

                    let (k, v) = &data[*pos];
                    *pos += 1;
                    (k.clone(), v.clone(), *priority)
                }
                IterSource::SST { iter, priority } => match iter.next()? {
                    Ok((k, v)) => (k, v, *priority),
                    Err(_) => return None,
                },
            };

            // if key is less that start that means we don't need that entry and we need something
            // higher than that
            if let Some(start) = start_bound {
                if &key < start {
                    continue;
                }
            }

            // similarly if the key is more than or equal to the end bound means we don't need that entry we
            // need something lower that that, and since we've already reached to a point where key
            // is greater than or equal to the end bound means we won't be getting anything of our use after
            // that (end bound is exclusive)
            if let Some(end) = end_bound {
                if &key >= end {
                    return None;
                }
            }
            return Some(IterEntry {
                key,
                value,
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
            // take one entry from the heap (i.e. the smallest one)
            let entry = self.heap.pop()?;

            // advance the source that produced this entry
            for idx in 0..self.sources.len() {
                let source_priority = match &self.sources[idx] {
                    IterSource::Memtable { priority, .. } => *priority,
                    IterSource::SST { priority, .. } => *priority,
                };

                // if we get the source where we got the source from we adnvace that source
                if source_priority == entry.priority {
                    if let Some(next_entry) = Self::advance_source(
                        &mut self.sources,
                        idx,
                        &self.start_bound,
                        &self.end_bound,
                    ) {
                        self.heap.push(next_entry);
                    }
                    break;
                }
            }

            // if the entry's key is same as the last key, hence it's duplicate and one key with
            // same value has already been pushed into the Iterator, so we have to ignore it
            if let Some(ref last) = self.last_key {
                if entry.key == *last {
                    continue;
                }
            }

            // value is emtpy hence it's tombstoned
            if entry.value.is_empty() {
                self.last_key = Some(entry.key);
                continue;
            }

            // change the last_key to the key we're about to return
            self.last_key = Some(entry.key.clone());
            return Some((entry.key, entry.value));
        }
    }
}
