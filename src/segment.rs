use crc32fast::Hasher;
use memmap2::Mmap;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufWriter, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SegmentError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("corrupt segment")]
    Corrupt,
    #[error("offset out of bounds")]
    OutOfBounds,
}

const MAGIC: u16 = 0xDA7A;
const HEADER_SIZE: usize = 12;
const BUFFER_SIZE: usize = 8 * 1024 * 1024;
pub const MAX_SEGMENT_SIZE: u64 = 32 * 1024 * 1024; // 32MB per segment

pub type Result<T> = std::result::Result<T, SegmentError>;

pub struct Segment {
    pub id: u64,
    file: BufWriter<File>,
    pub path: PathBuf,
    offset: u64,
}

impl Segment {
    pub fn create(path: &Path, id: u64) -> Result<Self> {
        let p = path.join(format!("segment-{}.log", id));
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&p)?;
        Ok(Self {
            id,
            file: BufWriter::with_capacity(BUFFER_SIZE, file),
            path: p,
            offset: 0,
        })
    }

    pub fn open(path: &Path, id: u64) -> Result<Self> {
        let p = path.join(format!("segment-{}.log", id));
        let mut file = OpenOptions::new().write(true).read(true).open(&p)?;
        let offset = file.seek(SeekFrom::End(0))?;
        Ok(Self {
            id,
            file: BufWriter::with_capacity(BUFFER_SIZE, file),
            path: p,
            offset,
        })
    }

    pub fn size(&self) -> u64 {
        self.offset
    }

    pub fn append(&mut self, key: &[u8], val: &[u8]) -> Result<u64> {
        let offset = self.offset;

        let record_len = HEADER_SIZE + key.len() + val.len() + 4;
        let mut buf = Vec::with_capacity(record_len);

        buf.extend_from_slice(&MAGIC.to_le_bytes());
        buf.extend_from_slice(&(record_len as u32).to_le_bytes());
        buf.extend_from_slice(&(key.len() as u16).to_le_bytes());
        buf.extend_from_slice(&(val.len() as u32).to_le_bytes());
        buf.extend_from_slice(key);
        buf.extend_from_slice(val);

        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let crc = hasher.finalize();
        buf.extend_from_slice(&crc.to_le_bytes());

        self.file.write_all(&buf)?;
        self.offset += buf.len() as u64;

        Ok(offset)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        self.file.get_mut().sync_data()?;
        Ok(())
    }

    pub fn iter_records(path: &Path, id: u64) -> Result<Vec<(Vec<u8>, Vec<u8>, u64)>> {
        let p = path.join(format!("segment-{}.log", id));
        let file = File::open(&p)?;
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            return Ok(Vec::new());
        }

        let mmap = unsafe { Mmap::map(&file)? };
        let mut out = Vec::new();
        let mut pos = 0usize;

        while pos + HEADER_SIZE + 4 <= mmap.len() {
            let start_pos = pos;

            let magic = u16::from_le_bytes([mmap[pos], mmap[pos + 1]]);
            if magic != MAGIC {
                return Err(SegmentError::Corrupt);
            }
            pos += 2;

            let record_len =
                u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]])
                    as usize;
            pos += 4;

            if start_pos + record_len > mmap.len() {
                break;
            }

            let key_len = u16::from_le_bytes([mmap[pos], mmap[pos + 1]]) as usize;
            pos += 2;

            let val_len =
                u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]])
                    as usize;
            pos += 4;

            let key = mmap[pos..pos + key_len].to_vec();
            pos += key_len;

            let val = mmap[pos..pos + val_len].to_vec();
            pos += val_len;

            let crc = u32::from_le_bytes([mmap[pos], mmap[pos + 1], mmap[pos + 2], mmap[pos + 3]]);
            pos += 4;

            let mut hasher = Hasher::new();
            hasher.update(&mmap[start_pos..start_pos + record_len - 4]);
            if hasher.finalize() != crc {
                return Err(SegmentError::Corrupt);
            }

            out.push((key, val, start_pos as u64));
        }
        Ok(out)
    }

    pub fn read_at(path: &Path, id: u64, offset: u64) -> Result<Vec<u8>> {
        let p = path.join(format!("segment-{}.log", id));
        let file = File::open(&p)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let pos = offset as usize;
        if pos + HEADER_SIZE + 4 > mmap.len() {
            return Err(SegmentError::OutOfBounds);
        }

        let magic = u16::from_le_bytes([mmap[pos], mmap[pos + 1]]);
        if magic != MAGIC {
            return Err(SegmentError::Corrupt);
        }

        let mut idx = pos + 2;

        let record_len =
            u32::from_le_bytes([mmap[idx], mmap[idx + 1], mmap[idx + 2], mmap[idx + 3]]) as usize;
        idx += 4;

        if pos + record_len > mmap.len() {
            return Err(SegmentError::OutOfBounds);
        }

        let key_len = u16::from_le_bytes([mmap[idx], mmap[idx + 1]]) as usize;
        idx += 2;

        let val_len =
            u32::from_le_bytes([mmap[idx], mmap[idx + 1], mmap[idx + 2], mmap[idx + 3]]) as usize;
        idx += 4;

        idx += key_len;

        let val = mmap[idx..idx + val_len].to_vec();
        idx += val_len;

        let crc = u32::from_le_bytes([mmap[idx], mmap[idx + 1], mmap[idx + 2], mmap[idx + 3]]);

        let mut hasher = Hasher::new();
        hasher.update(&mmap[pos..pos + record_len - 4]);
        if hasher.finalize() != crc {
            return Err(SegmentError::Corrupt);
        }

        Ok(val)
    }
}

pub struct SegmentCache {
    cache: Mutex<HashMap<u64, Arc<Mmap>>>,
    dir: PathBuf,
}

impl SegmentCache {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            dir,
        }
    }

    pub fn read_at(&self, segment_id: u64, offset: u64) -> Result<Vec<u8>> {
        let mut cache = self.cache.lock();

        if !cache.contains_key(&segment_id) {
            let p = self.dir.join(format!("segment-{}.log", segment_id));
            let file = File::open(&p)?;
            let mmap = unsafe { Mmap::map(&file)? };
            cache.insert(segment_id, Arc::new(mmap));
        }

        let mmap = cache.get(&segment_id).unwrap().clone();
        drop(cache);

        let pos = offset as usize;
        if pos + HEADER_SIZE + 4 > mmap.len() {
            return Err(SegmentError::OutOfBounds);
        }

        let magic = u16::from_le_bytes([mmap[pos], mmap[pos + 1]]);
        if magic != MAGIC {
            return Err(SegmentError::Corrupt);
        }

        let mut idx = pos + 2;

        let record_len =
            u32::from_le_bytes([mmap[idx], mmap[idx + 1], mmap[idx + 2], mmap[idx + 3]]) as usize;
        idx += 4;

        if pos + record_len > mmap.len() {
            return Err(SegmentError::OutOfBounds);
        }

        let key_len = u16::from_le_bytes([mmap[idx], mmap[idx + 1]]) as usize;
        idx += 2;

        let val_len =
            u32::from_le_bytes([mmap[idx], mmap[idx + 1], mmap[idx + 2], mmap[idx + 3]]) as usize;
        idx += 4;

        idx += key_len;

        let val = mmap[idx..idx + val_len].to_vec();
        idx += val_len;

        let crc = u32::from_le_bytes([mmap[idx], mmap[idx + 1], mmap[idx + 2], mmap[idx + 3]]);

        let mut hasher = Hasher::new();
        hasher.update(&mmap[pos..pos + record_len - 4]);
        if hasher.finalize() != crc {
            return Err(SegmentError::Corrupt);
        }

        Ok(val)
    }

    pub fn invalidate(&self, segment_id: u64) {
        self.cache.lock().remove(&segment_id);
    }
}
