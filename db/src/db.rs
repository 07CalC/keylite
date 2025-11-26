use std::path::Path;

use keylite_kv::db::Db;
use keylite_kv::error::DbError;
use serde_json::Value;

use crate::{
    collection::{CollectionMeta, Index, collection_meta_key, doc_key},
    index::{non_unique_index, unique_index},
};

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(v) => v.to_string(),
        _ => value.to_string(),
    }
}
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
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.kv.get(key)
    }
    pub fn del(&self, key: &[u8]) -> Result<()> {
        self.kv.del(key)
    }

    pub fn create_collection(&self, name: &str, indexes: Option<Vec<Index>>) -> Result<()> {
        let key = collection_meta_key(name);
        match self.kv.get(&key) {
            Ok(some) => {
                if some.is_some() {
                    return Ok(());
                }
            }
            Err(e) => return Err(e),
        }

        let meta = CollectionMeta {
            name: name.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            indexes,
        };
        let bytes = rmp_serde::to_vec(&meta).unwrap();
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
        let meta_key = collection_meta_key(collection);
        let meta_bytes = self
            .kv
            .get(&meta_key)?
            .ok_or_else(|| DbError::Other(format!("collection `{}` does not exist", collection)))?;

        let meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes).unwrap();

        let id = if let Some(id_val) = doc.get("_id") {
            id_val.as_str().unwrap().to_string()
        } else {
            let new_id = uuid::Uuid::new_v4().to_string();
            doc["_id"] = Value::String(new_id.clone());
            new_id
        };

        let doc_bytes = rmp_serde::to_vec(&doc).unwrap();
        let dkey = doc_key(collection, &id);

        self.kv.put(&dkey, &doc_bytes)?;

        if let Some(indexes) = &meta.indexes {
            for index in indexes {
                let field = &index.field;
                let field_value = doc.get(field).cloned().unwrap_or(Value::Null);

                if index.unique {
                    let ukey = unique_index(collection, field, &field_value);
                    if self.kv.get(&ukey)?.is_some() {
                        return Err(DbError::Other(format!(
                            "unique constraint: {}={}",
                            field, field_value
                        )));
                    }
                    self.kv.put(&ukey, id.as_bytes())?;
                } else {
                    let ikey = non_unique_index(collection, field, &field_value, &id);
                    self.kv.put(&ikey, &[1])?;
                }
            }
        }

        Ok(id)
    }

    pub fn get_doc_by_id(&self, collection: &str, id: &str) -> Result<Option<Value>> {
        let key = doc_key(&collection, &id);

        Ok(match self.kv.get(&key) {
            Ok(some) => match some {
                Some(val) => {
                    let v = rmp_serde::from_slice::<Value>(&val).unwrap();
                    Some(v)
                }
                None => None,
            },
            Err(e) => return Err(e),
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

    pub fn get_by_index(&self, collection: &str, field: &str, value: &Value) -> Result<Vec<Value>> {
        let meta_key = collection_meta_key(&collection);
        let meta_bytes = self
            .kv
            .get(&meta_key)?
            .ok_or_else(|| DbError::Other(format!("collection `{collection}` does not exist")))?;
        let meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes).unwrap();

        let index = meta
            .indexes
            .as_ref()
            .and_then(|indexes| indexes.iter().find(|idx| idx.field == field))
            .ok_or_else(|| DbError::Other(format!("no index on field `{field}`")))?;
        let mut results = Vec::new();
        if index.unique {
            let ukey = unique_index(collection, field, value);
            if let Some(id_bytes) = self.kv.get(&ukey)? {
                let id = String::from_utf8(id_bytes).unwrap();
                if let Some(doc) = self.get_doc_by_id(collection, &id)? {
                    results.push(doc);
                }
            }
        } else {
            let prefix = format!("idx:n:{collection}:{field}:{}", value_to_string(value));
            let (start, end) = prefix_range(&prefix);
            let iter = self.kv.scan(Some(&start), Some(&end));

            for (k, _) in iter {
                let key_str = String::from_utf8(k).unwrap();
                if let Some(id) = key_str.split(':').last() {
                    if let Some(doc) = self.get_doc_by_id(collection, id)? {
                        results.push(doc);
                    }
                }
            }
        }

        Ok(results)
    }
}
