use arc_swap::ArcSwap;
use crossbeam_channel::Receiver;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::DbError;
use crate::sst::{SSTReader, SSTWriter};
use crate::storage::Memtable;

use super::queue::FlushMessage;

type Result<T> = std::result::Result<T, DbError>;

pub fn flush_worker(
    receiver: Receiver<FlushMessage>,
    dir: std::path::PathBuf,
    sstables: Arc<ArcSwap<Vec<SSTReader>>>,
    next_sst_id: Arc<AtomicU64>,
) {
    loop {
        match receiver.recv() {
            Ok(FlushMessage::Flush(memtable)) => {
                if let Err(e) = flush_memtable_to_disk(&memtable, &dir, &sstables, &next_sst_id) {
                    eprintln!("Error flushing memtable: {}", e);
                }
            }
            Ok(FlushMessage::Shutdown) | Err(_) => break,
        }
    }
}

pub fn flush_memtable_to_disk(
    memtable: &Memtable,
    dir: &Path,
    sstables: &Arc<ArcSwap<Vec<SSTReader>>>,
    next_sst_id: &Arc<AtomicU64>,
) -> Result<()> {
    if memtable.is_empty() {
        return Ok(());
    }

    let sst_id = next_sst_id.fetch_add(1, Ordering::Relaxed);
    let sst_path = dir.join(format!("sst-{}.db", sst_id));

    let mut writer = SSTWriter::new(&sst_path)?;
    for (key, val) in memtable.iter() {
        writer.add(&key, &val)?;
    }
    writer.finish()?;

    let reader = SSTReader::open(&sst_path)?;

    loop {
        let current = sstables.load();
        let mut new_sstables = (**current).clone();
        new_sstables.insert(0, reader.clone());

        let prev = sstables.compare_and_swap(&current, Arc::new(new_sstables));
        if Arc::ptr_eq(&*prev, &*current) {
            break;
        }
    }

    Ok(())
}
