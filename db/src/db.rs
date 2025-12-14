use std::path::Path;

use keylite_kv::{core::Db, transaction::Transaction};
use serde_json::Value;

use crate::{
    collection::{CollectionMeta, Index, collection_meta_key, doc_key},
    error::{DocError, Result},
    index::{non_unique_index, prefix_range, unique_index},
    transaction::Txn,
};

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(v) => v.to_string(),
        _ => value.to_string(),
    }
}
pub struct KeyLite {
    kv: Db,
}

impl KeyLite {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let kv = Db::open(path).map_err(DocError::from)?;
        Ok(Self { kv })
    }

    pub fn put(&self, key: &[u8], val: &[u8]) -> Result<()> {
        self.kv.put(key, val).map_err(DocError::from)
    }
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.kv.get(key).map_err(DocError::from)
    }
    pub fn del(&self, key: &[u8]) -> Result<()> {
        self.kv.del(key).map_err(DocError::from)
    }

    pub fn create_collection(&self, name: &str, indexes: Option<Vec<Index>>) -> Result<()> {
        let key = collection_meta_key(name);
        match self.kv.get(&key).map_err(DocError::from)? {
            Some(_) => return Ok(()),
            None => {}
        }

        let meta = CollectionMeta {
            name: name.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            indexes,
        };
        let bytes = rmp_serde::to_vec(&meta)?;
        self.kv.put(&key, &bytes).map_err(DocError::from)
    }

    pub fn create_index(&self, indexes: Vec<Index>, collection: &str) -> Result<()> {
        let key = collection_meta_key(collection);
        let meta_bytes = match self.kv.get(&key).map_err(DocError::from)? {
            Some(meta) => meta,
            None => {
                return Err(DocError::CollectionNotFound(collection.to_string()));
            }
        };
        let mut meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes)?;
        match meta.indexes.as_mut() {
            Some(existing) => {
                existing.extend(indexes);
            }
            None => meta.indexes = Some(indexes),
        }
        let updated_bytes = rmp_serde::to_vec(&meta)?;

        self.kv.put(&key, &updated_bytes).map_err(DocError::from)?;
        Ok(())
    }

    pub fn drop_index(&self, index_field: &str, collection: &str) -> Result<()> {
        let key = collection_meta_key(collection);
        let meta_bytes = match self.kv.get(&key).map_err(DocError::from)? {
            Some(bytes) => bytes,
            None => {
                return Err(DocError::CollectionNotFound(collection.to_string()));
            }
        };

        let mut meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes)?;

        if let Some(ref mut indexes) = meta.indexes {
            indexes.retain(|idx| idx.field != index_field);

            if indexes.is_empty() {
                meta.indexes = None;
            }
        } else {
            return Err(DocError::IndexNotFound(format!(
                "index '{index_field}' does not exist on collection '{collection}'"
            )));
        }

        self.del_by_prefix(&format!("idx:u:{collection}:{index_field}:"))?;
        self.del_by_prefix(&format!("idx:n:{collection}:{index_field}:"))?;
        let updated_bytes = rmp_serde::to_vec(&meta)?;

        self.kv.put(&key, &updated_bytes).map_err(DocError::from)?;

        Ok(())
    }

    pub fn list_index(&self, collection: &str) -> Result<Vec<Index>> {
        let key = collection_meta_key(collection);
        let meta_bytes = match self.kv.get(&key).map_err(DocError::from)? {
            Some(meta) => meta,
            None => {
                return Err(DocError::CollectionNotFound(collection.to_string()));
            }
        };
        let meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes)?;
        Ok(meta.indexes.unwrap_or_default())
    }

    fn del_by_prefix(&self, prefix: &str) -> Result<()> {
        let (start, end) = prefix_range(prefix);
        let iter = self.kv.scan(Some(&start), Some(&end));
        for (k, _) in iter {
            self.kv.del(&k).map_err(DocError::from)?;
        }
        Ok(())
    }

    pub fn drop_collection(&self, name: &str) -> Result<()> {
        let meta = collection_meta_key(&name);
        self.kv.del(&meta).map_err(DocError::from)?;
        self.del_by_prefix(&format!("col:{name}:doc:"))?;
        Ok(())
    }

    pub fn insert(&self, collection: &str, mut doc: Value) -> Result<String> {
        let id = if let Some(id_val) = doc.get("_id") {
            id_val
                .as_str()
                .ok_or_else(|| DocError::InvalidDocumentId("_id must be a string".to_string()))?
                .to_string()
        } else {
            let new_id = uuid::Uuid::new_v4().to_string();
            doc["_id"] = Value::String(new_id.clone());
            new_id
        };

        let doc_bytes = rmp_serde::to_vec(&doc)?;
        let dkey = doc_key(collection, &id);

        self.kv.put(&dkey, &doc_bytes).map_err(DocError::from)?;

        let indexes = self.list_index(collection)?;

        for index in indexes {
            let field = &index.field;
            let field_value = doc.get(field).cloned().unwrap_or(Value::Null);

            if index.unique {
                let ukey = unique_index(collection, field, &field_value);
                if self.kv.get(&ukey).map_err(DocError::from)?.is_some() {
                    return Err(DocError::UniqueConstraintViolation {
                        field: field.clone(),
                        value: value_to_string(&field_value),
                    });
                }
                self.kv.put(&ukey, id.as_bytes()).map_err(DocError::from)?;
            } else {
                let ikey = non_unique_index(collection, field, &field_value, &id);
                self.kv.put(&ikey, &[1]).map_err(DocError::from)?;
            }
        }

        Ok(id)
    }

    pub fn get_doc_by_id(&self, collection: &str, id: &str) -> Result<Option<Value>> {
        let key = doc_key(&collection, &id);

        Ok(match self.kv.get(&key).map_err(DocError::from)? {
            Some(val) => {
                let v = rmp_serde::from_slice::<Value>(&val)?;
                Some(v)
            }
            None => None,
        })
    }

    pub fn delete_doc_by_id(&self, collection: &str, id: &str) -> Result<()> {
        let key = doc_key(collection, id);
        self.kv.del(&key).map_err(DocError::from)
    }

    pub fn scan_collection(&self, collection: &str) -> Result<Vec<Value>> {
        let prefix = format!("col:{collection}:doc:");
        let (start, end) = prefix_range(&prefix);
        let iter = self.kv.scan(Some(&start), Some(&end));
        let mut out = Vec::new();
        for (_k, v) in iter {
            let doc: Value = rmp_serde::from_slice(&v)?;
            out.push(doc);
        }
        Ok(out)
    }

    pub fn get_by_index(&self, collection: &str, field: &str, value: &Value) -> Result<Vec<Value>> {
        let meta_key = collection_meta_key(&collection);
        let meta_bytes = self
            .kv
            .get(&meta_key)
            .map_err(DocError::from)?
            .ok_or_else(|| DocError::CollectionNotFound(collection.to_string()))?;
        let meta: CollectionMeta = rmp_serde::from_slice(&meta_bytes)?;

        let index = meta
            .indexes
            .as_ref()
            .and_then(|indexes| indexes.iter().find(|idx| idx.field == field))
            .ok_or_else(|| DocError::IndexNotFound(field.to_string()))?;
        let mut results = Vec::new();
        if index.unique {
            let ukey = unique_index(collection, field, value);
            if let Some(id_bytes) = self.kv.get(&ukey).map_err(DocError::from)? {
                let id = String::from_utf8(id_bytes)?;
                if let Some(doc) = self.get_doc_by_id(collection, &id)? {
                    results.push(doc);
                }
            }
        } else {
            let prefix = format!("idx:n:{collection}:{field}:{}", value_to_string(value));
            let (start, end) = prefix_range(&prefix);
            let iter = self.kv.scan(Some(&start), Some(&end));

            for (k, _) in iter {
                let key_str = String::from_utf8(k)?;
                if let Some(id) = key_str.split(':').last() {
                    if let Some(doc) = self.get_doc_by_id(collection, id)? {
                        results.push(doc);
                    }
                }
            }
        }

        Ok(results)
    }
    pub fn get_by_field_forced(
        &self,
        collection: &str,
        field: &str,
        value: &Value,
    ) -> Result<Vec<Value>> {
        let prefix = format!("col:{collection}:doc:");
        let (start, end) = prefix_range(&prefix);
        let iter = self.kv.scan(Some(&start), Some(&end));
        let mut result = Vec::new();
        for (_, v) in iter {
            let doc: Value = match rmp_serde::from_slice(&v) {
                Ok(d) => d,
                Err(_) => continue,
            };

            if let Some(field_value) = doc.get(field) {
                if field_value.to_string().to_lowercase() == value.to_string().to_lowercase() {
                    result.push(doc);
                }
            }
        }
        Ok(result)
    }

    pub fn begin(&self) -> Txn {
        let transaction = self.kv.begin();
        Txn::new(&self, transaction)
    }
}
