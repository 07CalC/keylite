use std::path::Path;

use keylite_kv::db::Db;
use keylite_kv::error::DbError;
use serde_json::Value;

use crate::collection::{CollectionMeta, collection_meta_key, doc_key};
pub struct KeyLite {
    kv: Db,
}

pub type Result<T> = std::result::Result<T, DbError>;

fn prefix_range(prefix: &str) -> (Vec<u8>, Vec<u8>) {
    let start = prefix.as_bytes().to_vec();
    let mut end = start.clone();
    end.push(0xFF);
    (start, end)
}

impl KeyLite {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let kv = Db::open(path)?;
        Ok(Self { kv })
    }

    pub fn put(&self, key: &[u8], val: &[u8]) -> Result<()> {
        self.kv.put(key, val)
    }
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.kv.get(key)
    }
    pub fn del(&self, key: &[u8]) -> Result<()> {
        self.kv.del(key)
    }

    pub fn create_collection(&self, name: &str) -> Result<()> {
        let key = collection_meta_key(name);
        if self.kv.get(&key).is_some() {
            return Ok(());
        };

        let meta = CollectionMeta {
            name: name.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        };
        let bytes = serde_json::to_vec(&meta).unwrap();
        self.kv.put(&key, &bytes)
    }

    pub fn drop_collection(&self, name: &str) -> Result<()> {
        let meta = collection_meta_key(&name);
        self.kv.del(&meta)?;

        let prefix = format!("col:{name}:doc:");
        let (start, end) = prefix_range(&prefix);
        let iter = self.kv.scan(Some(&start), Some(&end));
        for (k, _) in iter {
            self.kv.del(&k)?;
        }

        Ok(())
    }

    pub fn insert(&self, collection: &str, mut doc: Value) -> Result<String> {
        self.create_collection(&collection)?;
        let id = if let Some(id_val) = doc.get("_id") {
            id_val.as_str().unwrap().to_string()
        } else {
            let new_id = uuid::Uuid::new_v4().to_string();
            doc["_id"] = Value::String(new_id.clone());
            new_id
        };

        let key = doc_key(&collection, &id);
        let bytes = rmp_serde::to_vec(&doc).unwrap();
        self.kv.put(&key, &bytes)?;
        Ok(id)
    }

    pub fn get_doc_by_id(&self, collection: &str, id: &str) -> Result<Option<Value>> {
        let key = doc_key(&collection, &id);
        Ok(match self.kv.get(&key) {
            Some(bytes) => {
                let v = rmp_serde::from_slice::<Value>(&bytes).unwrap();
                Some(v)
            }
            None => None,
        })
    }

    pub fn delete_doc_by_id(&self, collection: &str, id: &str) -> Result<()> {
        let key = doc_key(collection, id);
        self.kv.del(&key)
    }

    pub fn scan_collection(&self, collection: &str) -> Result<Vec<Value>> {
        let prefix = format!("col:{collection}:doc:");
        let (start, end) = prefix_range(&prefix);
        let iter = self.kv.scan(Some(&start), Some(&end));
        let mut out = Vec::new();
        for (_k, v) in iter {
            let doc: Value = rmp_serde::from_slice(&v).unwrap();
            out.push(doc);
        }
        Ok(out)
    }
}
