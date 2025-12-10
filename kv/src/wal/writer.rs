use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Result, Write},
    path::Path,
};

use crc32fast::Hasher;

pub struct WalWriter {
    seq: u64,
    file: BufWriter<File>,
}

impl WalWriter {
    pub fn new(seq: u64, path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            seq,
            file: BufWriter::new(file),
        })
    }
    pub fn append(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&(key.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(val.len() as u32).to_le_bytes());
        buf.extend_from_slice(&key);
        buf.extend_from_slice(&val);

        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let crc = hasher.finalize();

        buf.extend_from_slice(&crc.to_le_bytes());

        self.file.write(&buf)?;

        Ok(())
    }

    pub fn sync(&mut self) -> Result<()> {
        self.file.flush()?;
        self.file.get_ref().sync_all()?;
        Ok(())
    }
}
