use std::{fs::File, sync::Arc};

pub fn start_wal_sync_thread(file: Arc<File>, interval_ms: u64) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
        let _ = file.sync_all();
    })
}
