use arc_swap::ArcSwap;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::compaction::{compaction_worker, CompactionMessage};
use crate::core::iterator::DbIterator;
use crate::error::{DbError, Result};
use crate::flush::{flush_memtable_to_disk, flush_worker, FlushMessage, FlushQueue};
use crate::memtable::Memtable;
use crate::sst::SSTReader;
use crate::transaction::Transaction;
use crate::wal::reader::{WalEntry, WalReader};
use crate::wal::thread::{wal_thread, WalMessage};
use crossbeam_channel::Sender;

use super::config::{MAX_SSTABLES, MEMTABLE_SIZE_THRESHOLD};

pub struct Db {
    dir: PathBuf,
    memtable: Arc<ArcSwap<Memtable>>,
    immutable_memtables: Arc<ArcSwap<Vec<Arc<Memtable>>>>,
    sstables: Arc<ArcSwap<Vec<SSTReader>>>,
    next_sst_id: Arc<AtomicU64>,
    flush_sender: Sender<FlushMessage>,
    compaction_sender: Sender<CompactionMessage>,
    wal_sender: Sender<WalMessage>,
    flush_thread: Option<JoinHandle<()>>,
    compaction_thread: Option<JoinHandle<()>>,
    wal_thread: Option<JoinHandle<()>>,
    global_sequence: Arc<AtomicU64>,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;

        let mut sst_ids = Vec::new();
        let mut has_wal = false;
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
            if name.starts_with("wal") {
                has_wal = true;
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
        let immutable_memtables = Arc::new(ArcSwap::from_pointee(Vec::new()));

        let flush_queue = FlushQueue::new();
        let flush_sender = flush_queue.sender();
        let flush_receiver = flush_queue.receiver();

        let flush_dir = dir.clone();
        let flush_sstables = Arc::clone(&sstables);
        let flush_immutables = Arc::clone(&immutable_memtables);
        let flush_next_id = Arc::clone(&next_sst_id);

        let (wal_tx, wal_rx) = crossbeam_channel::unbounded();
        let wal_tx_for_flush = wal_tx.clone();
        let flush_thread = thread::spawn(move || {
            flush_worker(
                flush_receiver,
                flush_dir,
                flush_sstables,
                flush_immutables,
                flush_next_id,
                wal_tx_for_flush,
            )
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
        let mut max_seq = 0;
        let memtable = Memtable::new();

        for sst in sstables.load().iter() {
            max_seq = max_seq.max(sst.max_sequence());
        }

        if has_wal {
            let wal_path = dir.join("wal.log");
            if wal_path.exists() {
                let mut wal_reader = WalReader::new(&wal_path)?;
                while let Some(record) = wal_reader.next_entry()? {
                    max_seq = max_seq.max(record.seq);
                    memtable.put(record.key, record.val, record.seq);
                    if memtable.size_bytes() >= MEMTABLE_SIZE_THRESHOLD {
                        flush_memtable_to_disk(
                            &memtable,
                            &dir,
                            &sstables,
                            &next_sst_id,
                            wal_tx.clone(),
                        )?;
                        memtable.clear();
                    }
                }
            }
        }

        let global_sequence = Arc::new(AtomicU64::new(max_seq.saturating_add(1)));

        let wal_path = dir.join("wal.log");

        let wal_thread = thread::spawn(move || {
            wal_thread(wal_path, wal_rx, 20);
        });

        Ok(Self {
            dir,
            memtable: Arc::new(ArcSwap::from_pointee(memtable)),
            immutable_memtables,
            sstables,
            next_sst_id,
            flush_sender,
            compaction_sender,
            wal_sender: wal_tx,
            flush_thread: Some(flush_thread),
            compaction_thread: Some(compaction_thread),
            global_sequence,
            wal_thread: Some(wal_thread),
        })
    }

    pub fn begin(&self) -> Transaction<'_> {
        Transaction::new(self.global_sequence.load(Ordering::Acquire), self)
    }

    // sllocate a new sequence number for transaction commits or other operations
    pub(crate) fn next_sequence(&self) -> u64 {
        self.global_sequence.fetch_add(1, Ordering::SeqCst)
    }

    // write goes to the memtable
    // if memtable reaches a certain max size that memtable is freezed and a new empty memtable
    // takes its place
    // at any time 1 mutable memtable and 2 immutable memtables are allowed, if immutable memtables
    // crosses 2 then the oldest one gets flushed in the SST file
    pub fn put(&self, key: &[u8], val: &[u8]) -> Result<()> {
        let seq = self.global_sequence.fetch_add(1, Ordering::SeqCst);

        let _ = self.wal_sender.send(WalMessage::Append(WalEntry {
            seq,
            key: key.to_vec(),
            val: val.to_vec(),
        }));

        let memtable = self.memtable.load();
        memtable.put(key.to_vec(), val.to_vec(), seq);

        // flush if needed
        self.flush_if_needed();

        Ok(())
    }

    // put but with of a particular seq
    // used in transactions
    pub fn put_seq(&self, key: &[u8], val: &[u8], seq: u64) -> Result<()> {
        let _ = self.wal_sender.send(WalMessage::Append(WalEntry {
            seq,
            key: key.to_vec(),
            val: val.to_vec(),
        }));
        let memtable = self.memtable.load();
        memtable.put(key.to_vec(), val.to_vec(), seq);

        self.flush_if_needed();

        Ok(())
    }

    // first we'll check the mutable memtable that's there for current writes
    // then check the 2 immutable memtable
    // if not found then fallback to SSTs
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        //  mutable memtable
        let memtable = self.memtable.load();
        if let Some(val) = memtable.get(key) {
            if val.is_empty() {
                return Ok(None);
            }
            return Ok(Some(val));
        }

        //  immutable memtables
        let immutables = self.immutable_memtables.load();
        for mt in immutables.iter().rev() {
            if let Some(val) = mt.get(key) {
                if val.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(val));
            }
        }

        //  sstables
        let sstables = self.sstables.load();
        for sst in sstables.iter() {
            match sst.get(key) {
                Ok(Some(val)) => {
                    if val.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(val));
                }
                Ok(None) => continue,
                Err(e) => return Err(DbError::SST(e)),
            }
        }

        Ok(None)
    }

    pub fn get_seq(&self, key: &[u8], seq: u64) -> Result<Option<Vec<u8>>> {
        let memtable = self.memtable.load();

        if let Some(val) = memtable.get_seq(key, seq) {
            if val.is_empty() {
                return Ok(None);
            }
            return Ok(Some(val));
        }

        let immutables = self.immutable_memtables.load();
        for imt in immutables.iter().rev() {
            if let Some(val) = imt.get_seq(key, seq) {
                if val.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(val));
            }
        }

        let ssts = self.sstables.load();

        for sst in ssts.iter() {
            if let Some(val) = sst.get_seq(key, seq)? {
                if val.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(val));
            }
        }

        Ok(None)
    }

    // deletion is not on spot, rather its like putting a tombstone (i.e. emtpy value) to that
    // particular key, after compaction the old entries with some value are removed, also the
    // emtpy value entry is also removed
    pub fn del(&self, key: &[u8]) -> Result<()> {
        self.put(key, &[])
    }

    // deletion with a particular seq
    // used in transactions
    pub fn del_seq(&self, key: &[u8], seq: u64) -> Result<()> {
        self.put_seq(key, &[], seq)
    }

    pub fn flush_if_needed(&self) {
        let memtable = self.memtable.load();
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
                    // keep up to 2 immutable memtables in memory. When a 3rd one is created,
                    // send the oldest to the flush queue. The memtable will remain in the list
                    // until the flush worker successfully writes it to an SST file, to dodge race
                    // conditions
                    // this ensures data is always available during async flush operations.
                    let current = self.immutable_memtables.load();
                    let mut new_immutables = (**current).clone();
                    new_immutables.push(old_memtable.clone());

                    // only send to flush if we have more than 2 immutable memtables
                    // but dont' remove it from the list yet,
                    // its the job of the flush worker
                    // after successful flushing and creation on SSTable, flush worker will remove
                    // the immutable memtable that just got flushed from the memory
                    let should_flush = if new_immutables.len() > 2 {
                        Some(new_immutables[0].clone()) // send the oldest to flush
                    } else {
                        None
                    };

                    // swap in the new list
                    let prev = self
                        .immutable_memtables
                        .compare_and_swap(&current, Arc::new(new_immutables));

                    if Arc::ptr_eq(&*prev, &*current) {
                        // send to flush queue if needed
                        if let Some(oldest) = should_flush {
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
    }

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
            let _ = flush_memtable_to_disk(
                &remaining_mt,
                &self.dir,
                &self.sstables,
                &self.next_sst_id,
                self.wal_sender.clone(),
            );
        }

        let immutable = self.immutable_memtables.load_full();

        for mt in immutable.iter() {
            // flush all the immutable memtables to the disk
            if !mt.is_empty() {
                let _ = flush_memtable_to_disk(
                    mt,
                    &self.dir,
                    &self.sstables,
                    &self.next_sst_id,
                    self.wal_sender.clone(),
                );
            }
        }

        // stop the flush and compaction workers
        let _ = self.flush_sender.send(FlushMessage::Shutdown);
        let _ = self.compaction_sender.send(CompactionMessage::Shutdown);
        let _ = self.wal_sender.send(WalMessage::Shutdown);

        // join the background threads
        if let Some(handle) = self.flush_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.compaction_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.wal_thread.take() {
            let _ = handle.join();
        }
    }
}
