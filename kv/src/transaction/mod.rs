use crossbeam_skiplist::SkipMap;

use crate::{
    core::{Db, DbIterator},
    error::Result,
};

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

    pub fn scan(&self, start: Option<&[u8]>, end: Option<&[u8]>) -> TransactionIterator {
        // Get underlying DB iterator with snapshot isolation at transaction's sequence
        let db_iter = self.db.scan_seq(start, end, self.seq);

        // Collect transaction buffer entries within the range
        let mut txn_entries = Vec::new();
        for entry in self.buf.iter() {
            let key = entry.key();

            // Check if key is within range
            if let Some(s) = start {
                if key.as_slice() < s {
                    continue;
                }
            }
            if let Some(e) = end {
                if key.as_slice() >= e {
                    continue;
                }
            }

            txn_entries.push((key.clone(), entry.value().clone()));
        }

        TransactionIterator {
            db_iter,
            txn_entries,
            txn_pos: 0,
            last_key: None,
            peeked_db_entry: None,
        }
    }
}

pub struct TransactionIterator {
    db_iter: DbIterator,
    txn_entries: Vec<(Vec<u8>, Vec<u8>)>,
    txn_pos: usize,
    last_key: Option<Vec<u8>>,
    peeked_db_entry: Option<(Vec<u8>, Vec<u8>)>, // Store peeked DB entry
}

impl Iterator for TransactionIterator {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // peek at both sources first
            let txn_entry = if self.txn_pos < self.txn_entries.len() {
                Some(&self.txn_entries[self.txn_pos])
            } else {
                None
            };

            // get or peek DB entry
            if self.peeked_db_entry.is_none() {
                self.peeked_db_entry = self.db_iter.next();
            }

            // determine which entry to be returned
            let (key, val) = match (txn_entry, &self.peeked_db_entry) {
                (Some((tk, tv)), Some((dk, _))) => {
                    // Both have entries, pick the smaller key
                    // Transaction entries take precedence on equal keys
                    if tk <= dk {
                        self.txn_pos += 1;
                        (tk.clone(), tv.clone())
                    } else {
                        let entry = self.peeked_db_entry.take().unwrap();
                        entry
                    }
                }
                (Some((tk, tv)), None) => {
                    // only transaction has entry
                    self.txn_pos += 1;
                    (tk.clone(), tv.clone())
                }
                (None, Some(_)) => {
                    // only DB has entry
                    let entry = self.peeked_db_entry.take().unwrap();
                    entry
                }
                (None, None) => {
                    // none has a entry
                    return None;
                }
            };

            // skip duplicates
            if let Some(ref last) = self.last_key {
                if &key == last {
                    continue;
                }
            }
            self.last_key = Some(key.clone());

            // skip tombstones, i.e. empty values or deleted values
            if val.is_empty() {
                continue;
            }

            return Some((key, val));
        }
    }
}
