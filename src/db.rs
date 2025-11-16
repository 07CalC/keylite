use parking_lot::{Mutex, RwLock};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::index::{Index, IndexEntry};
use crate::segment::{Segment, SegmentCache, MAX_SEGMENT_SIZE};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("segment: {0}")]
    Segment(#[from] crate::segment::SegmentError),
}

pub type Result<T> = std::result::Result<T, DbError>;

pub struct Db {
    dir: PathBuf,
    current_segment_id: AtomicU64,
    current_segment: Mutex<Segment>,
    index: RwLock<Index>,
    segment_cache: SegmentCache,
    dirty: AtomicBool,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        let mut highest = 0u64;
        for entry in read_dir(&dir)? {
            let e = entry?;
            let name = e.file_name().into_string().unwrap_or_default();
            if let Some(s) = name
                .strip_prefix("segment-")
                .and_then(|s| s.strip_suffix(".log"))
            {
                if let Ok(id) = s.parse::<u64>() {
                    highest = highest.max(id);
                }
            }
        }
        let segment = if highest == 0 {
            Segment::create(&dir, 1)?
        } else {
            Segment::open(&dir, highest)?
        };
        let db = Self {
            segment_cache: SegmentCache::new(dir.clone()),
            dir,
            current_segment_id: AtomicU64::new(if highest == 0 { 1 } else { highest }),
            current_segment: Mutex::new(segment),
            index: RwLock::new(Index::new()),
            dirty: AtomicBool::new(false),
        };
        db.rebuild_index()?;
        Ok(db)
    }

    fn rebuild_index(&self) -> Result<()> {
        let mut idx = self.index.write();
        let current_id = self.current_segment_id.load(Ordering::Acquire);
        for id in 1..=current_id {
            let recs = Segment::iter_records(&self.dir, id)?;
            for (k, v, pos) in recs {
                if v.is_empty() {
                    idx.remove(&k);
                } else {
                    idx.insert(
                        k,
                        IndexEntry {
                            segment_id: id,
                            offset: pos,
                        },
                    );
                }
            }
        }
        Ok(())
    }

    pub fn put(&self, key: &[u8], val: &[u8]) -> Result<()> {
        {
            let seg = self.current_segment.lock();
            if seg.size() >= MAX_SEGMENT_SIZE {
                drop(seg);

                let mut seg = self.current_segment.lock();
                seg.flush()?;
                drop(seg);

                let old_id = self.current_segment_id.load(Ordering::Acquire);
                let new_id = old_id + 1;
                let new_seg = Segment::create(&self.dir, new_id)?;

                let mut seg = self.current_segment.lock();
                *seg = new_seg;
                drop(seg);

                self.current_segment_id.store(new_id, Ordering::Release);
            }
        }

        let mut seg = self.current_segment.lock();
        let offset = seg.append(key, val)?;
        let segment_id = seg.id;
        drop(seg);

        self.dirty.store(true, Ordering::Release);
        self.segment_cache.invalidate(segment_id);

        let mut idx = self.index.write();
        idx.insert(key.to_vec(), IndexEntry { segment_id, offset });
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        if self.dirty.load(Ordering::Acquire) {
            let mut seg = self.current_segment.lock();
            seg.flush()?;
            self.dirty.store(false, Ordering::Release);
        }
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if self.dirty.load(Ordering::Acquire) {
            let mut seg = self.current_segment.lock();
            seg.flush()?;
            self.dirty.store(false, Ordering::Release);
        }

        let idx = self.index.read();
        if let Some(e) = idx.get(key) {
            let segment_id = e.segment_id;
            let offset = e.offset;
            drop(idx);

            let val = self.segment_cache.read_at(segment_id, offset)?;
            if val.is_empty() {
                Ok(None)
            } else {
                Ok(Some(val))
            }
        } else {
            Ok(None)
        }
    }

    pub fn del(&self, key: &[u8]) -> Result<()> {
        self.put(key, &[])
    }

    pub fn compact(&mut self) -> Result<()> {
        self.flush()?;

        let old_id = self.current_segment_id.load(Ordering::Acquire);
        let new_id = old_id + 1;
        let mut new_seg = Segment::create(&self.dir, new_id)?;

        let idx = self.index.read();
        let keys_to_compact: Vec<(Box<[u8]>, u64, u64)> = idx
            .map
            .iter()
            .map(|(k, entry)| (k.clone(), entry.segment_id, entry.offset))
            .collect();
        drop(idx);

        let mut new_offsets = Vec::with_capacity(keys_to_compact.len());
        for (k, seg_id, offset) in keys_to_compact {
            let val = self.segment_cache.read_at(seg_id, offset)?;
            if !val.is_empty() {
                let new_offset = new_seg.append(&k, &val)?;
                new_offsets.push((k, new_offset));
            }
        }
        new_seg.flush()?;

        let seg = Segment::open(&self.dir, new_id)?;
        self.current_segment_id.store(new_id, Ordering::Release);
        let mut guard = self.current_segment.lock();
        *guard = seg;
        drop(guard);

        let mut idx = self.index.write();
        for (k, new_offset) in new_offsets {
            idx.insert(
                k.into_vec(),
                IndexEntry {
                    segment_id: new_id,
                    offset: new_offset,
                },
            );
        }
        drop(idx);

        for id in 1..new_id {
            self.segment_cache.invalidate(id);
        }

        Ok(())
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
