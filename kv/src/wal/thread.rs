use crossbeam_channel::Receiver;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::wal::{reader::WalEntry, writer::WalWriter};

pub enum WalMessage {
    Append(WalEntry),
    Truncate,
    Shutdown,
}

pub fn wal_thread(path: PathBuf, rx: Receiver<WalMessage>, flush_interval_ms: u64) {
    let mut wal = WalWriter::new(&path).expect("Failed to open WAL");

    let mut last_flush = Instant::now();

    loop {
        match rx.recv() {
            Ok(rec) => match rec {
                WalMessage::Append(entry) => {
                    let _ = wal.append(&entry.key, &entry.val, entry.seq);
                }
                WalMessage::Truncate => {
                    let _ = wal.sync();
                    drop(wal);
                    let _ = std::fs::remove_file(&path).expect("failed to removed wal file");
                    wal = WalWriter::new(&path).expect("failed to recreate Wal");
                    last_flush = Instant::now();
                }
                WalMessage::Shutdown => {
                    break;
                }
            },

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
