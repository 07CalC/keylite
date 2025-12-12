use std::{
    fs::File,
    io::{BufReader, Read, Result},
    path::Path,
};

use crc32fast::Hasher;

pub struct WalEntry {
    pub seq: u64,
    pub key: Vec<u8>,
    pub val: Vec<u8>,
}

pub struct WalReader {
    reader: BufReader<File>,
}

impl WalReader {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    pub fn next_entry(&mut self) -> Result<Option<WalEntry>> {
        // 14 bytes,
        // 8 for seq
        // 2 for key_len
        // 4 for val_len
        let mut header = [0u8; 14];

        if self.reader.read_exact(&mut header).is_err() {
            return Ok(None);
        }

        let seq = u64::from_le_bytes(
            header[0..8]
                .try_into()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid seq bytes"))?,
        );
        let key_len = u16::from_le_bytes(
            header[8..10]
                .try_into()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid key_len bytes"))?,
        ) as usize;
        let val_len = u32::from_le_bytes(
            header[10..14]
                .try_into()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid val_len bytes"))?,
        ) as usize;

        let total_len = key_len + val_len;
        let mut data = vec![0u8; total_len];

        self.reader.read_exact(&mut data)?;

        let key = data[0..key_len].to_vec();
        let val = data[key_len..].to_vec();

        let mut crc_bytes = [0u8; 4];
        self.reader.read_exact(&mut crc_bytes)?;
        let stored_crc = u32::from_le_bytes(crc_bytes);

        let mut hasher = Hasher::new();
        hasher.update(&header);
        hasher.update(&data);
        let computed_crc = hasher.finalize();

        if stored_crc != computed_crc {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "WAL corruption detected",
            ));
        }

        Ok(Some(WalEntry { seq, key, val }))
    }
}
