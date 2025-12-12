// flush worker runs on a background thread, it's job is to flush the memtable to the disk and
// remove the memtables that are flushed from memory, ONLY AND ONLY after successfull addition of a
// new SSTable
// while the process is running flush worked only flushes the oldes immutable memtable

use arc_swap::ArcSwap;
use crossbeam_channel::{Receiver, Sender};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::DbError;
use crate::memtable::Memtable;
use crate::sst::{SSTReader, SSTWriter};
use crate::wal::thread::WalMessage;

use super::queue::FlushMessage;

type Result<T> = std::result::Result<T, DbError>;

pub fn flush_worker(
    receiver: Receiver<FlushMessage>,
    dir: std::path::PathBuf,
    sstables: Arc<ArcSwap<Vec<SSTReader>>>,
    immutable_memtables: Arc<ArcSwap<Vec<Arc<Memtable>>>>,
    next_sst_id: Arc<AtomicU64>,
    wal_tx: Sender<WalMessage>,
) {
    loop {
        match receiver.recv() {
            Ok(FlushMessage::Flush(memtable)) => {
                // println!("[FLUSH] Starting flush of immutable memtable ({} entries, {} bytes)",
                //     memtable.len(), memtable.size_bytes());
                if let Err(e) = flush_and_remove_memtable(
                    &memtable,
                    &dir,
                    &sstables,
                    &immutable_memtables,
                    &next_sst_id,
                    wal_tx.clone(),
                ) {
                    eprintln!("Error flushing memtable: {}", e);
                } else {
                    // println!("[FLUSH] Completed flush of immutable memtable");
                }
            }
            Ok(FlushMessage::Shutdown) | Err(_) => break,
        }
    }
}

// flush a memtable and then remove it from the immutable memtables list onlfy after successfull
// flush and creation of SSTable
fn flush_and_remove_memtable(
    memtable: &Arc<Memtable>,
    dir: &Path,
    sstables: &Arc<ArcSwap<Vec<SSTReader>>>,
    immutable_memtables: &Arc<ArcSwap<Vec<Arc<Memtable>>>>,
    next_sst_id: &Arc<AtomicU64>,
    wal_tx: Sender<WalMessage>,
) -> Result<()> {
    // flush the memtable to disk
    flush_memtable_to_disk(memtable, dir, sstables, next_sst_id, wal_tx)?;

    // now that the SST is added
    // we can remove the immutable memtable from the list
    // find and remove this specific immutable memtable from the immutable memtables list
    loop {
        let current = immutable_memtables.load();
        let mut new_immutables = (**current).clone();

        // find the index of this memtable (comparing by pointer)
        if let Some(pos) = new_immutables
            .iter()
            .position(|mt| Arc::ptr_eq(mt, memtable))
        {
            new_immutables.remove(pos);

            let prev = immutable_memtables.compare_and_swap(&current, Arc::new(new_immutables));
            if Arc::ptr_eq(&*prev, &*current) {
                break;
            }
        } else {
            // memtable not found in the list, might have been already removed
            break;
        }
    }

    Ok(())
}

pub fn flush_memtable_to_disk(
    memtable: &Memtable,
    dir: &Path,
    sstables: &Arc<ArcSwap<Vec<SSTReader>>>,
    next_sst_id: &Arc<AtomicU64>,
    wal_tx: Sender<WalMessage>,
) -> Result<()> {
    // if memtable is empty there is nothing to flush
    if memtable.is_empty() {
        return Ok(());
    }

    // get the next sst_id that the worker gonna write to the disk
    let sst_id = next_sst_id.fetch_add(1, Ordering::Relaxed);
    let sst_path = dir.join(format!("sst-{}.db", sst_id));

    // create new SSTWriter, implemented in /sst/writer.rs
    let mut writer = SSTWriter::new(&sst_path)?;

    // iterate over memtable entries in sorted order (skipmap is already sorted)
    for (vk, val) in memtable.iter() {
        // writer.add method adds the entry in the buffer
        writer.add(&vk.key, &val, vk.seq)?;
    }

    // writer.finish method wrties all the entries is has in the buffer, with the bloom filters,
    // block indexes and the footer
    writer.finish()?;

    let reader = SSTReader::open(&sst_path)?;

    // add the newly created SST to the sstables list
    loop {
        let current = sstables.load();
        let mut new_sstables = (**current).clone();
        new_sstables.insert(0, reader.clone());

        // insert the newly created sstable in the global sst list
        let prev = sstables.compare_and_swap(&current, Arc::new(new_sstables));
        if Arc::ptr_eq(&*prev, &*current) {
            break;
        }
    }

    let _ = wal_tx.send(WalMessage::Truncate);
    Ok(())
}
