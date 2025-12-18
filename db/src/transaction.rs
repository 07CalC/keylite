use keylite_kv::transaction::Transaction;
use serde_json::Value;

use crate::{
    collection::{CollectionMeta, collection_meta_key, doc_key},
    db::{KeyLite, value_to_string},
    error::{DocError, Result},
    index::{non_unique_index, prefix_range, unique_index},
};

pub struct Txn<'a> {
    db: &'a KeyLite,
    txn: Transaction<'a>,
}

impl<'a> Txn<'a> {
    pub fn new(db: &'a KeyLite, txn: Transaction<'a>) -> Self {
        Self { db, txn }
    }

    pub fn insert(&mut self, collection: &str, mut doc: Value) -> Result<String> {
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

        self.txn.put(&dkey, &doc_bytes);

        let indexes = self.db.list_index(collection)?;

        for index in indexes {
            let field = &index.field;
            let field_value = doc.get(field).cloned().unwrap_or(Value::Null);

            if index.unique {
                let ukey = unique_index(collection, field, &field_value);
                if self.txn.get(&ukey).map_err(DocError::from)?.is_some() {
                    return Err(DocError::UniqueConstraintViolation {
                        field: field.clone(),
                        value: value_to_string(&field_value),
                    });
                }
                self.txn.put(&ukey, id.as_bytes());
            } else {
                let ikey = non_unique_index(collection, field, &field_value, &id);
                self.txn.put(&ikey, &[1]);
            }
        }

        Ok(id)
    }

    pub fn get_doc_by_id(&mut self, collection: &str, id: &str) -> Result<Option<Value>> {
        let key = doc_key(&collection, &id);

        Ok(match self.txn.get(&key).map_err(DocError::from)? {
            Some(val) => {
                let v = rmp_serde::from_slice::<Value>(&val)?;
                Some(v)
            }
            None => None,
        })
    }

    pub fn get_by_index(
        &mut self,
        collection: &str,
        field: &str,
        value: &Value,
    ) -> Result<Vec<Value>> {
        let meta_key = collection_meta_key(&collection);
        let meta_bytes = self
            .txn
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
            if let Some(id_bytes) = self.txn.get(&ukey).map_err(DocError::from)? {
                let id = String::from_utf8(id_bytes)?;
                if let Some(doc) = self.get_doc_by_id(collection, &id)? {
                    results.push(doc);
                }
            }
        } else {
            let prefix = format!("idx:n:{collection}:{field}:{}", value_to_string(value));
            let (start, end) = prefix_range(&prefix);
            let iter = self.txn.scan(Some(&start), Some(&end));

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
        let iter = self.txn.scan(Some(&start), Some(&end));
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

    pub fn commit(self) -> Result<()> {
        self.txn.commit().map_err(DocError::from)
    }
}
