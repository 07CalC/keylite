use parking_lot::RwLock;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::compaction::{compaction_worker, CompactionMessage};
use crate::flush::{flush_memtable_to_disk, flush_worker, FlushMessage, FlushQueue};
use crate::sst::SSTReader;
use crate::storage::Memtable;
use crossbeam_channel::Sender;

use super::config::{MAX_SSTABLES, MEMTABLE_SIZE_THRESHOLD};
use super::Result;

pub struct Db {
    dir: PathBuf,
    memtable: RwLock<Memtable>,
    immutable_memtables: RwLock<Vec<Memtable>>,
    sstables: Arc<RwLock<Vec<SSTReader>>>,
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
        // might not need to open all the SSTs at once
        // TODO: make it more efficient
        let mut sstables = Vec::new();
        for id in sst_ids.iter().rev() {
            let path = dir.join(format!("sst-{}.db", id));
            if let Ok(reader) = SSTReader::open(&path) {
                sstables.push(reader);
            }
        }

        let sstables = Arc::new(RwLock::new(sstables));
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
            memtable: RwLock::new(Memtable::new()),
            immutable_memtables: RwLock::new(Vec::new()),
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
        let mut mt = self.memtable.write();
        mt.put(key.to_vec(), val.to_vec());

        let should_flush = mt.size_bytes() >= MEMTABLE_SIZE_THRESHOLD;

        if should_flush {
            let old_mt = {
                let mut new_mt = Memtable::new();
                std::mem::swap(&mut *mt, &mut new_mt);
                new_mt
            };
            drop(mt);

            if !old_mt.is_empty() {
                let mut immutable = self.immutable_memtables.write();
                immutable.push(old_mt);

                if immutable.len() > 2 {
                    let oldest = immutable.remove(0);
                    let _ = self.flush_sender.send(FlushMessage::Flush(oldest));
                }
                drop(immutable);
            }

            let sst_count = self.sstables.read().len();
            if sst_count >= MAX_SSTABLES {
                let _ = self.compaction_sender.send(CompactionMessage::Compact);
            }
        } else {
            drop(mt);
        }

        Ok(())
    }

    // first we'll check the mutable memtable that's there for current writes
    // then check the 2 immutable memtable
    // if not found then fallback to SSTs
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        {
            let mt = self.memtable.read();
            if let Some(val) = mt.get(key) {
                if val.is_empty() {
                    return None;
                }
                return Some(val.to_vec());
            }
        }

        {
            let immutable = self.immutable_memtables.read();
            for mt in immutable.iter().rev() {
                if let Some(val) = mt.get(key) {
                    if val.is_empty() {
                        return None;
                    }
                    return Some(val.to_vec());
                }
            }
        }

        let sstables = self.sstables.read();
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

    pub fn del(&self, key: &[u8]) -> Result<()> {
        self.put(key, &[])
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        let remaining_mt = {
            let mut mt = self.memtable.write();
            std::mem::take(&mut *mt)
        };

        if !remaining_mt.is_empty() {
            let _ =
                flush_memtable_to_disk(&remaining_mt, &self.dir, &self.sstables, &self.next_sst_id);
        }

        let immutable = {
            let mut immutable = self.immutable_memtables.write();
            std::mem::take(&mut *immutable)
        };

        for mt in immutable {
            if !mt.is_empty() {
                let _ = flush_memtable_to_disk(&mt, &self.dir, &self.sstables, &self.next_sst_id);
            }
        }

        let _ = self.flush_sender.send(FlushMessage::Shutdown);
        let _ = self.compaction_sender.send(CompactionMessage::Shutdown);
        if let Some(handle) = self.flush_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.compaction_thread.take() {
            let _ = handle.join();
        }
    }
}
