use crossbeam_skiplist::{SkipList, SkipMap};

use crate::core::{Db, Result};

pub enum TxnOp {
    Put { key: Vec<u8>, val: Vec<u8> },
    Del { key: Vec<u8> },
}

pub struct Transaction<'a> {
    seq: u64,
    buf: SkipMap<Vec<u8>, Vec<u8>>,
    db: &'a Db,
}

impl<'a> Transaction<'a> {
    pub fn new(seq: u64, db: &'a Db) -> Self {
        Self {
            seq,
            buf: SkipMap::new(),
            db,
        }
    }
    pub fn put(&mut self, key: &[u8], val: &[u8]) {
        self.buf.insert(key.to_vec(), val.to_vec());
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if let Some(entry) = self.buf.get(key) {
            if entry.value().is_empty() {
                return Ok(None);
            }
            return Ok(Some(entry.value().to_vec()));
        }
        self.db.get_seq(key, self.seq)
    }
    pub fn del(&mut self, key: &[u8]) {
        self.buf.insert(key.to_vec(), vec![]);
    }

    pub fn commit(self) -> Result<()> {
        // Get a NEW sequence number for this transaction's commit
        // All operations in the transaction will use the same (new) sequence number
        // to ensure atomicity - they all appear to happen at the same instant
        let commit_seq = self.db.next_sequence();

        for entry in self.buf.iter() {
            self.db.put_seq(entry.key(), entry.value(), commit_seq)?;
        }

        Ok(())
    }

    pub fn abort(&mut self) {
        self.buf.clear();
    }
}
