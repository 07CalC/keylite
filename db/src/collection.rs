use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionMeta {
    pub name: String,
    pub created_at: i64,
}

pub fn collection_meta_key(name: &str) -> Vec<u8> {
    format!("_priv:collection:{name}").into_bytes()
}

pub fn doc_key(collection: &str, id: &str) -> Vec<u8> {
    format!("col:{collection}:doc:{id}").into_bytes()
}
