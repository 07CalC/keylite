#[allow(non_snake_case)]
use std::path::Path;

use keylite_kv::db::Db;
use keylite_kv::error::DbError;
use serde_json::Value;

use crate::{
    collection::{CollectionMeta, Index, collection_meta_key, doc_key},
    helper::{index_key, index_prefix},
};
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

    pub fn create_collection(&self, name: &str, indexes: Option<Vec<Index>>) -> Result<()> {
        let key = collection_meta_key(name);
        if self.kv.get(&key).is_some() {
            return Ok(());
        };

        let meta = CollectionMeta {
            name: name.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            indexes,
        };
        let bytes = rmp_serde::to_vec(&meta).unwrap();
        self.kv.put(&key, &bytes)
    }

    pub fn drop_collection(&self, name: &str) -> Result<()> {
        let meta_key = collection_meta_key(&name);

        if let Some(meta_bytes) = self.kv.get(&meta_key) {
            let meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes).unwrap();

            if let Some(ref indexes) = meta.indexes {
                for index in indexes {
                    let idx_prefix = format!("_priv:idx:{name}:{}:", index.field);
                    let (start, end) = prefix_range(&idx_prefix);
                    let iter: Vec<_> = self.kv.scan(Some(&start), Some(&end)).collect();
                    for (k, _) in iter {
                        self.kv.del(&k)?;
                    }
                }
            }
        }

        self.kv.del(&meta_key)?;

        let prefix = format!("col:{name}:doc:");
        let (start, end) = prefix_range(&prefix);
        let iter: Vec<_> = self.kv.scan(Some(&start), Some(&end)).collect();
        for (k, _) in iter {
            self.kv.del(&k)?;
        }

        Ok(())
    }

    pub fn insert(&self, collection: &str, mut doc: Value) -> Result<String> {
        let meta_key = collection_meta_key(&collection);
        let meta_bytes = match self.kv.get(&meta_key) {
            Some(bytes) => bytes,
            None => {
                self.create_collection(&collection, None)?;
                self.kv.get(&meta_key).unwrap()
            }
        };
        let meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes).unwrap();

        let id = if let Some(id_val) = doc.get("_id") {
            id_val.as_str().unwrap().to_string()
        } else {
            let new_id = uuid::Uuid::new_v4().to_string();
            doc["_id"] = Value::String(new_id.clone());
            new_id
        };

        if let Some(ref indexes) = meta.indexes {
            for index in indexes {
                let field_value = doc.get(&index.field).cloned().unwrap_or(Value::Null);

                if index.unique {
                    let (start, end) = index_prefix(collection, &index.field, &field_value);
                    let mut scan = self.kv.scan(Some(&start), Some(&end));
                    if scan.next().is_some() {
                        return Err(DbError::Io(std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            format!(
                                "Unique constraint failed on field '{}' for value {}",
                                index.field, field_value
                            ),
                        )));
                    }
                }

                let ikey = index_key(collection, &index.field, &field_value, &id);
                self.kv.put(&ikey, id.as_bytes())?;
            }
        }

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
        let doc_key_bytes = doc_key(collection, id);

        if let Some(doc_bytes) = self.kv.get(&doc_key_bytes) {
            let doc: Value = rmp_serde::from_slice(&doc_bytes).unwrap();

            let meta_key = collection_meta_key(collection);
            if let Some(meta_bytes) = self.kv.get(&meta_key) {
                let meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes).unwrap();

                if let Some(ref indexes) = meta.indexes {
                    for index in indexes {
                        let field_value = doc.get(&index.field).cloned().unwrap_or(Value::Null);
                        let ikey = index_key(collection, &index.field, &field_value, id);
                        self.kv.del(&ikey)?;
                    }
                }
            }
        }

        self.kv.del(&doc_key_bytes)
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

    pub fn get_by_index(&self, collection: &str, field: &str, value: Value) -> Result<Vec<Value>> {
        let (start, end) = index_prefix(collection, field, &value);
        let it = self.kv.scan(Some(&start), Some(&end));

        let mut out = Vec::new();
        for (_, v) in it {
            let id = String::from_utf8_lossy(&v).to_string();
            if let Some(doc) = self.get_doc_by_id(collection, &id)? {
                out.push(doc);
            }
        }
        Ok(out)
    }
}
