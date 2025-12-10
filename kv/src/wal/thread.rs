use crossbeam_channel::Receiver;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::wal::{reader::WalEntry, writer::WalWriter};

pub fn wal_thread(path: PathBuf, rx: Receiver<WalEntry>, flush_interval_ms: u64) {
    let mut wal = WalWriter::new(&path).expect("Failed to open WAL");

    let mut last_flush = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(flush_interval_ms)) {
            Ok(rec) => {
                let _ = wal.append(&rec.key, &rec.val, rec.seq);
            }
            Err(_) => {
                break;
            }
        }

        if last_flush.elapsed().as_millis() as u64 >= flush_interval_ms {
            let _ = wal.sync();
            last_flush = Instant::now();
        }
    }
}
