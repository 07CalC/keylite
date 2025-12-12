use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Result, Write},
    path::Path,
    sync::Arc,
};

use crc32fast::Hasher;

pub struct WalWriter {
    buf: BufWriter<File>,
    file: Arc<File>,
}

impl WalWriter {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            buf: BufWriter::new(file.try_clone()?),
            file: Arc::new(file),
        })
    }

    pub fn append(&mut self, key: &[u8], val: &[u8], seq: u64) -> Result<()> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&seq.to_le_bytes());
        buf.extend_from_slice(&(key.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(val.len() as u32).to_le_bytes());
        buf.extend_from_slice(key);
        buf.extend_from_slice(val);

        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let crc = hasher.finalize();
        buf.extend_from_slice(&crc.to_le_bytes());

        self.buf.write_all(&buf)?;

        Ok(())
    }

    pub fn sync(&mut self) -> Result<()> {
        self.buf.flush()?;
        self.file.sync_all()?;
        Ok(())
    }

    pub fn file_handle(&self) -> Arc<File> {
        Arc::clone(&self.file)
    }
}
