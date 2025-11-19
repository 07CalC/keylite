use std::{cmp::Ordering, collections::BinaryHeap, sync::Arc};

use crate::{
    sst::{SSTIterator, SSTReader},
    storage::Memtable,
};

#[derive(Clone, Debug)]
pub struct IterEntry {
    key: Vec<u8>,
    value: Vec<u8>,
    priority: usize,
}

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

enum IterSource {
    Memtable {
        data: Vec<(Vec<u8>, Vec<u8>)>,
        pos: usize,
        priority: usize,
    },
    SST {
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
        let memtable_priority = immutable_memtable.len() + sstables.len();

        // Collect memtable data into a Vec to avoid lifetime issues
        // Sort the data by key to maintain order
        let mut memtable_data: Vec<(Vec<u8>, Vec<u8>)> = memtable.iter().collect();
        memtable_data.sort_by(|a, b| a.0.cmp(&b.0));
        sources.push(IterSource::Memtable {
            data: memtable_data,
            pos: 0,
            priority: memtable_priority,
        });

        for (i, imt) in immutable_memtable.iter().enumerate() {
            let priority = memtable_priority - 1 - i;
            let mut imt_data: Vec<(Vec<u8>, Vec<u8>)> = imt.iter().collect();
            imt_data.sort_by(|a, b| a.0.cmp(&b.0));
            sources.push(IterSource::Memtable {
                data: imt_data,
                pos: 0,
                priority,
            });
        }

        for (i, sst) in sstables.iter().enumerate() {
            let priority = immutable_memtable.len() - i - 1;
            let iter = SSTIterator::new(sst.clone());
            sources.push(IterSource::SST { iter, priority });
        }

        for i in 0..sources.len() {
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
            let (key, value, priority) = match &mut sources[source_idx] {
                IterSource::Memtable {
                    data,
                    pos,
                    priority,
                } => {
                    if *pos >= data.len() {
                        return None;
                    }
                    let (k, v) = data[*pos].clone();
                    *pos += 1;
                    (k, v, *priority)
                }
                IterSource::SST { iter, priority } => {
                    match iter.next()? {
                        Ok((k, v)) => (k, v, *priority),
                        Err(_) => return None, // Skip corrupted entries
                    }
                }
            };
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
                priority,
            });
        }
    }
}

impl Iterator for DbIterator {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.heap.pop()?;

            // Find the source that produced this entry and advance it
            // We must do this BEFORE checking for duplicates to ensure all sources progress
            for idx in 0..self.sources.len() {
                let source_priority = match &self.sources[idx] {
                    IterSource::Memtable { priority, .. } => *priority,
                    IterSource::SST { priority, .. } => *priority,
                };

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

            // Skip duplicates - only process the highest priority version
            if let Some(ref last) = self.last_key {
                if &entry.key == last {
                    continue;
                }
            }

            // Update last_key to skip duplicates from lower priority sources
            self.last_key = Some(entry.key.clone());

            // Skip tombstones (empty values represent deletions)
            // Note: We still update last_key above so lower-priority versions are skipped
            if entry.value.is_empty() {
                continue;
            }

            return Some((entry.key, entry.value));
        }
    }
}
