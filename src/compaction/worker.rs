use crossbeam_channel::Receiver;
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;

use crate::error::DbError;
use crate::sst::{SSTIterator, SSTReader, SSTWriter};

type Result<T> = std::result::Result<T, DbError>;

pub enum CompactionMessage {
    Compact,
    Shutdown,
}

struct MergeEntry {
    key: Vec<u8>,
    value: Vec<u8>,
    sst_idx: usize,
}

impl PartialEq for MergeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for MergeEntry {}

impl PartialOrd for MergeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MergeEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .key
            .cmp(&self.key)
            .then(other.sst_idx.cmp(&self.sst_idx))
    }
}

pub fn compaction_worker(
    receiver: Receiver<CompactionMessage>,
    dir: std::path::PathBuf,
    sstables: Arc<RwLock<Vec<SSTReader>>>,
    next_sst_id: Arc<AtomicU64>,
) {
    loop {
        match receiver.recv() {
            Ok(CompactionMessage::Compact) => {
                if let Err(e) = compact_sstables(&dir, &sstables, &next_sst_id) {
                    eprintln!("Error during compaction: {}", e);
                }
            }
            Ok(CompactionMessage::Shutdown) | Err(_) => break,
        }
    }
}

fn compact_sstables(
    dir: &Path,
    sstables: &Arc<RwLock<Vec<SSTReader>>>,
    next_sst_id: &Arc<AtomicU64>,
) -> Result<()> {
    let old_sstables = {
        let mut sstables = sstables.write();
        std::mem::take(&mut *sstables)
    };

    if old_sstables.is_empty() {
        return Ok(());
    }

    let mut iterators: Vec<_> = old_sstables
        .iter()
        .rev()
        .map(|sst| SSTIterator::new(SSTReader::open(sst.path()).unwrap()))
        .collect();

    let mut heap = BinaryHeap::new();
    for (idx, iter) in iterators.iter_mut().enumerate() {
        if let Some(Ok((key, value))) = iter.next() {
            heap.push(MergeEntry {
                key,
                value,
                sst_idx: idx,
            });
        }
    }

    let sst_id = next_sst_id.fetch_add(1, AtomicOrdering::Relaxed);
    let sst_path = dir.join(format!("sst-{}.db", sst_id));
    let mut writer = SSTWriter::new(&sst_path)?;

    let mut last_key: Option<Vec<u8>> = None;
    let mut entry_count = 0;

    while let Some(entry) = heap.pop() {
        if let Some(ref lk) = last_key {
            if lk == &entry.key {
                // Same key, skip (already processed from newer SST)
                if let Some(Ok((key, value))) = iterators[entry.sst_idx].next() {
                    heap.push(MergeEntry {
                        key,
                        value,
                        sst_idx: entry.sst_idx,
                    });
                }
                continue;
            }
        }

        if !entry.value.is_empty() {
            writer.add(&entry.key, &entry.value)?;
            entry_count += 1;
        }

        last_key = Some(entry.key);

        if let Some(Ok((key, value))) = iterators[entry.sst_idx].next() {
            heap.push(MergeEntry {
                key,
                value,
                sst_idx: entry.sst_idx,
            });
        }
    }

    if entry_count == 0 {
        for sst in old_sstables {
            let _ = std::fs::remove_file(sst.path());
        }
        return Ok(());
    }

    writer.finish()?;

    let reader = SSTReader::open(&sst_path)?;
    let mut sstables = sstables.write();
    *sstables = vec![reader];

    for sst in old_sstables {
        let _ = std::fs::remove_file(sst.path());
    }

    Ok(())
}
