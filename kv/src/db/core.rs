use arc_swap::ArcSwap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::compaction::{compaction_worker, CompactionMessage};
use crate::db::iterator::DbIterator;
use crate::flush::{flush_memtable_to_disk, flush_worker, FlushMessage, FlushQueue};
use crate::sst::SSTReader;
use crate::storage::Memtable;
use crossbeam_channel::Sender;

use super::config::{MAX_SSTABLES, MEMTABLE_SIZE_THRESHOLD};
use super::Result;

pub struct Db {
    dir: PathBuf,
    memtable: Arc<ArcSwap<Memtable>>,
    immutable_memtables: Arc<ArcSwap<Vec<Arc<Memtable>>>>,
    sstables: Arc<ArcSwap<Vec<SSTReader>>>,
    next_sst_id: Arc<AtomicU64>,
    flush_sender: Sender<FlushMessage>,
    compaction_sender: Sender<CompactionMessage>,
    flush_thread: Option<JoinHandle<()>>,
    compaction_thread: Option<JoinHandle<()>>,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;

        let mut sst_ids = Vec::new();
        for entry in read_dir(&dir)? {
            let e = entry?;
            let name = e.file_name().into_string().unwrap_or_default();
            if let Some(s) = name
                .strip_prefix("sst-")
                .and_then(|s| s.strip_suffix(".db"))
            {
                if let Ok(id) = s.parse::<u64>() {
                    sst_ids.push(id);
                }
            }
        }

        sst_ids.sort_unstable();
        let next_id = sst_ids.last().map(|&id| id + 1).unwrap_or(1);

        // open SSTables in reverse order -> newest first for faster lookups
        let mut sstables = Vec::new();
        for id in sst_ids.iter().rev() {
            let path = dir.join(format!("sst-{}.db", id));
            if let Ok(reader) = SSTReader::open(&path) {
                sstables.push(reader);
            }
        }

        let sstables = Arc::new(ArcSwap::from_pointee(sstables));
        let next_sst_id = Arc::new(AtomicU64::new(next_id));

        let flush_queue = FlushQueue::new();
        let flush_sender = flush_queue.sender();
        let flush_receiver = flush_queue.receiver();

        let flush_dir = dir.clone();
        let flush_sstables = Arc::clone(&sstables);
        let flush_next_id = Arc::clone(&next_sst_id);

        let flush_thread = thread::spawn(move || {
            flush_worker(flush_receiver, flush_dir, flush_sstables, flush_next_id)
        });

        let (compaction_sender, compaction_receiver) = crossbeam_channel::unbounded();

        let compaction_dir = dir.clone();
        let compaction_sstables = Arc::clone(&sstables);
        let compaction_next_id = Arc::clone(&next_sst_id);

        let compaction_thread = thread::spawn(move || {
            compaction_worker(
                compaction_receiver,
                compaction_dir,
                compaction_sstables,
                compaction_next_id,
            )
        });

        Ok(Self {
            dir,
            memtable: Arc::new(ArcSwap::from_pointee(Memtable::new())),
            immutable_memtables: Arc::new(ArcSwap::from_pointee(Vec::new())),
            sstables,
            next_sst_id,
            flush_sender,
            compaction_sender,
            flush_thread: Some(flush_thread),
            compaction_thread: Some(compaction_thread),
        })
    }

    // write goes to the memtable
    // if memtable reaches a certain max size that memtable is freezed and a new empty memtable
    // takes its place
    // at any time 1 mutable memtable and 2 immutable memtables are allowed, if immutable memtables
    // crosses 2 then the oldest one gets flushed in the SST file
    pub fn put(&self, key: &[u8], val: &[u8]) -> Result<()> {
        let memtable = self.memtable.load();
        memtable.put(key.to_vec(), val.to_vec());

        // memtables are configured to be of a certain max size to cap the memory usage after that
        // limit is reached the memtables should be freezed and pushed to the flush queue which
        // will then write the memtable to sst file
        let should_flush = memtable.size_bytes() >= MEMTABLE_SIZE_THRESHOLD;

        if should_flush {
            // replace the memtable with a new empty one so that writes don't have to wait until
            // the memtable is being flushed to the file sys
            let new_memtable = Arc::new(Memtable::new());
            let old_memtable = self.memtable.swap(new_memtable);

            if !old_memtable.is_empty() {
                loop {
                    // at a time only 1 mutable and 2 immutable memtables are allowed to be in the
                    // memory, after the limit of 2 is reached the oldest immutable memtable is
                    // sent to the flush queue which will write it to the sst file
                    let current = self.immutable_memtables.load();
                    let mut new_immutables = (**current).clone();
                    new_immutables.push(old_memtable.clone());

                    let oldest = if new_immutables.len() > 2 {
                        Some(new_immutables.remove(0))
                    } else {
                        None
                    };

                    // swap the immutable memtables with new one
                    // the oldest one being replaced by a new emtpy one
                    let prev = self
                        .immutable_memtables
                        .compare_and_swap(&current, Arc::new(new_immutables));

                    // send the oldest immutable memtable to the flush queue
                    if Arc::ptr_eq(&*prev, &*current) {
                        if let Some(oldest) = oldest {
                            let _ = self.flush_sender.send(FlushMessage::Flush(oldest));
                        }
                        break;
                    }
                }
            }

            // at a moment only certain number of sstables are allowed after reaching that limit
            // the sstables are sent for compaction, where they are merged into one big sstable
            // removing all the duplicates, tombstones
            let sst_count = self.sstables.load().len();
            if sst_count >= MAX_SSTABLES {
                let _ = self.compaction_sender.send(CompactionMessage::Compact);
            }
        }

        Ok(())
    }

    // first we'll check the mutable memtable that's there for current writes
    // then check the 2 immutable memtable
    // if not found then fallback to SSTs
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        //  mutable memtable
        let memtable = self.memtable.load();
        if let Some(val) = memtable.get(key) {
            if val.is_empty() {
                return None;
            }
            return Some(val);
        }

        //  immutable memtables
        let immutables = self.immutable_memtables.load();
        for mt in immutables.iter().rev() {
            if let Some(val) = mt.get(key) {
                if val.is_empty() {
                    return None;
                }
                return Some(val);
            }
        }

        //  sstables
        let sstables = self.sstables.load();
        for sst in sstables.iter() {
            match sst.get(key) {
                Ok(Some(val)) => {
                    if val.is_empty() {
                        return None;
                    }
                    return Some(val);
                }
                Ok(None) => continue,
                Err(_) => continue,
            }
        }

        None
    }

    // deletion is not on-spot, rather its like putting a tombstone (i.e. emtpy value) to that
    // particular key, after compaction the old entries with some value are removed, also the
    // emtpy value entry is also removed
    pub fn del(&self, key: &[u8]) -> Result<()> {
        self.put(key, &[])
    }

    //TODO: performance can be enhanced for empty scans
    // i.e. i scanned from ff:11 to ff::66 but when ff:11 to ff::66 nothing exists it takes too
    // much time
    pub fn scan(&self, start: Option<&[u8]>, end: Option<&[u8]>) -> DbIterator {
        let memtable = Arc::clone(&self.memtable.load());
        let immutables = (**self.immutable_memtables.load()).clone();
        let sstables = (**self.sstables.load()).clone();

        let start_bound = start.map(|s| s.to_vec());
        let end_bound = end.map(|e| e.to_vec());

        DbIterator::new(memtable, immutables, sstables, start_bound, end_bound)
    }
}

// custom memory drop implementation to join all the threads (i.e. the compaction and flush queue
// threads) working in the background, and to flush all the data currently in memory to disk so
// that no data is lost during shutdown
impl Drop for Db {
    fn drop(&mut self) {
        let remaining_mt = self.memtable.load_full();

        if !remaining_mt.is_empty() {
            // flush the mutable memtable with whatever data it has
            let _ =
                flush_memtable_to_disk(&remaining_mt, &self.dir, &self.sstables, &self.next_sst_id);
        }

        let immutable = self.immutable_memtables.load_full();

        for mt in immutable.iter() {
            // flush all the immutable memtables to the disk
            if !mt.is_empty() {
                let _ = flush_memtable_to_disk(mt, &self.dir, &self.sstables, &self.next_sst_id);
            }
        }

        // stop the flush and compaction workers
        let _ = self.flush_sender.send(FlushMessage::Shutdown);
        let _ = self.compaction_sender.send(CompactionMessage::Shutdown);

        // join the background threads
        if let Some(handle) = self.flush_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.compaction_thread.take() {
            let _ = handle.join();
        }
    }
}
