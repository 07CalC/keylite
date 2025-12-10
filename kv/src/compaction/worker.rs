use arc_swap::ArcSwap;
use crossbeam_channel::Receiver;
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
    seq: u64,
    sst_idx: usize,
}

impl PartialEq for MergeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.seq == other.seq
    }
}

impl Eq for MergeEntry {}

impl PartialOrd for MergeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// cmp function is reversed to give the reversed order
// i.e. in binary heap it will pop the smallest key first
impl Ord for MergeEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // smallest user key should come out of the heap first
        other
            .key
            .cmp(&self.key)
            // for same key, we want the newest version first → higher seq first
            .then(self.seq.cmp(&other.seq))
            // tie-breaker on sst index (newer/older sst)
            .then(other.sst_idx.cmp(&self.sst_idx))
    }
}

pub fn compaction_worker(
    receiver: Receiver<CompactionMessage>,
    dir: std::path::PathBuf,
    sstables: Arc<ArcSwap<Vec<SSTReader>>>,
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
    sstables: &Arc<ArcSwap<Vec<SSTReader>>>,
    next_sst_id: &Arc<AtomicU64>,
) -> Result<()> {
    // take the sstables atomically with retries
    // load the old sstables and swap then with emtpy Vecs so that new writes can go there
    let old_sstables = loop {
        let current = sstables.load();
        let prev = sstables.compare_and_swap(&current, Arc::new(Vec::new()));
        if Arc::ptr_eq(&*prev, &*current) {
            break (**current).clone();
        }
    };

    // no sst to flush to disk
    if old_sstables.is_empty() {
        return Ok(());
    }

    // take all the iterators for sstables
    // implemented in /sst/iterator.rs
    // rev() because new sstables will override the older ones, newest data to be considered as
    // truth
    let mut iterators: Vec<_> = old_sstables
        .iter()
        .rev()
        .map(|sst| SSTIterator::new(SSTReader::open(sst.path()).unwrap()))
        .collect();

    // binaryheap works as max heap
    // but since the cmp method is over written (see line 39 of this file) to give the reverse
    // ordering it works as min heap (smallest key on top)
    let mut heap = BinaryHeap::new();

    // put the first Entry from each sst to the binary heap to proceed with k-way merge
    for (idx, iter) in iterators.iter_mut().enumerate() {
        if let Some(Ok((key, value, seq))) = iter.next() {
            heap.push(MergeEntry {
                key,
                value,
                seq,
                sst_idx: idx,
            });
        }
    }

    // get the new sst_id and add one to the db struct
    let sst_id = next_sst_id.fetch_add(1, AtomicOrdering::Relaxed);
    let sst_path = dir.join(format!("sst-{}.db", sst_id));
    let mut writer = SSTWriter::new(&sst_path)?;

    // store last key to dodge duplication
    let mut last_key: Option<Vec<u8>> = None;
    let mut entry_count = 0;

    // heap.pop() will give the smallest key entry
    while let Some(entry) = heap.pop() {
        // the popped key is equal to the last key, means we need not to use it,
        // i.e. we will keep the Entry from the latest sstable, and discard all the entry with the
        // same key from old sstabels
        if let Some(ref lk) = last_key {
            if lk == &entry.key {
                // get a new entry from the same sst to componsate the duplicate entry
                if let Some(Ok((key, value, seq))) = iterators[entry.sst_idx].next() {
                    heap.push(MergeEntry {
                        key,
                        value,
                        seq,
                        sst_idx: entry.sst_idx,
                    });
                }
                // don't put the duplicate key in the sst writer, hence continue
                continue;
            }
        }

        // if value is not empty (i.e. it's not tombstoned, deletion is equivalent of putting an
        // emtpy value for that particular key) only then add it to sstwriter
        if !entry.value.is_empty() {
            // pass seq into the new SST, preserving version ordering
            writer.add(&entry.key, &entry.value, entry.seq)?;
            entry_count += 1;
        }

        // if the entry is valid to be added to writer store it in last_key so that any other entry
        // with the same key can be discarded in the next loop
        last_key = Some(entry.key);

        // push a new entry from the same sst to the heap
        if let Some(Ok((key, value, seq))) = iterators[entry.sst_idx].next() {
            heap.push(MergeEntry {
                key,
                value,
                seq,
                sst_idx: entry.sst_idx,
            });
        }
    }

    // if entry_count is 0 hence the compaction doesn't produce any entry, hence every entry was
    // tombstoned, so all the sst are waste of storage containing only the tombstones, hence they
    // all should be deleted
    if entry_count == 0 {
        for sst in old_sstables {
            let _ = std::fs::remove_file(sst.path());
        }
        // return on the spot, we do not need to do the writing to file stuff, because technically
        // no new sst is created
        return Ok(());
    }

    // writed.finish() method flushes the newly created sst to the disk
    // check /sst/writer.rs
    writer.finish()?;

    // create a new sst reader to put in the db struct
    let reader = SSTReader::open(&sst_path)?;

    // replace the sst list with the newly created sst
    loop {
        let current = sstables.load();
        let prev = sstables.compare_and_swap(&current, Arc::new(vec![reader.clone()]));
        if Arc::ptr_eq(&*prev, &*current) {
            break;
        }
    }

    // remove the old ssts from the file syst
    for sst in old_sstables {
        let _ = std::fs::remove_file(sst.path());
    }

    Ok(())
}
