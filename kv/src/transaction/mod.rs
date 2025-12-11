use crate::db::{Db, Result};

pub enum TxnOp {
    Put { key: Vec<u8>, val: Vec<u8> },
    Del { key: Vec<u8> },
}

pub struct Transaction<'a> {
    seq: u64,
    buf: Vec<TxnOp>,
    db: &'a Db,
}

impl<'a> Transaction<'a> {
    pub fn new(seq: u64, db: &'a Db) -> Self {
        Self {
            seq,
            buf: Vec::new(),
            db,
        }
    }
    pub fn put(&mut self, key: &[u8], val: &[u8]) {
        self.buf.push(TxnOp::Put {
            key: key.to_vec(),
            val: val.to_vec(),
        });
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.db.get_seq(key, self.seq)
    }
    pub fn del(&mut self, key: &[u8]) {
        self.buf.push(TxnOp::Del { key: key.to_vec() });
    }

    pub fn commit(self) -> Result<()> {
        // Get a NEW sequence number for this transaction's commit
        // All operations in the transaction will use the same (new) sequence number
        // to ensure atomicity - they all appear to happen at the same instant
        let commit_seq = self.db.next_sequence();
        
        for ops in self.buf {
            match ops {
                TxnOp::Put { key, val } => self.db.put_seq(&key, &val, commit_seq)?,
                TxnOp::Del { key } => self.db.del_seq(&key, commit_seq)?,
            }
        }
        Ok(())
    }

    pub fn abort(&mut self) {
        self.buf.clear();
    }
}
