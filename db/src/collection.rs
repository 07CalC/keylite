use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub field: String,
    pub unique: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionMeta {
    pub name: String,
    pub created_at: i64,
    pub indexes: Option<Vec<Index>>,
}

pub fn collection_meta_key(name: &str) -> Vec<u8> {
    format!("_priv:collection:{name}").into_bytes()
}

pub fn doc_key(collection: &str, id: &str) -> Vec<u8> {
    format!("col:{collection}:doc:{id}").into_bytes()
}

// Document metadata fields (all prefixed with underscore)
pub const FIELD_ID: &str = "_id";
pub const FIELD_VERSION: &str = "_version";
pub const FIELD_CREATED_AT: &str = "_created_at";
pub const FIELD_UPDATED_AT: &str = "_updated_at";

/// Add metadata fields to a document
pub fn add_document_metadata(doc: &mut Value, id: String, version: u64, now: i64) {
    if let Value::Object(map) = doc {
        map.insert(FIELD_ID.to_string(), Value::String(id));
        map.insert(FIELD_VERSION.to_string(), Value::Number(version.into()));
        map.insert(FIELD_CREATED_AT.to_string(), Value::Number(now.into()));
        map.insert(FIELD_UPDATED_AT.to_string(), Value::Number(now.into()));
    }
}

/// Update document metadata (version and updated_at)
pub fn update_document_metadata(doc: &mut Value, version: u64, now: i64) {
    if let Value::Object(map) = doc {
        map.insert(FIELD_VERSION.to_string(), Value::Number(version.into()));
        map.insert(FIELD_UPDATED_AT.to_string(), Value::Number(now.into()));
    }
}

/// Get document version (returns 0 if not found)
pub fn get_document_version(doc: &Value) -> u64 {
    doc.get(FIELD_VERSION)
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

/// Get document ID (returns None if not found)
pub fn get_document_id(doc: &Value) -> Option<String> {
    doc.get(FIELD_ID)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Check if a field name is a reserved system field (starts with underscore)
pub fn is_system_field(field: &str) -> bool {
    field.starts_with('_')
}

/// Strip system fields from user input to prevent overwrites
pub fn strip_system_fields(doc: &mut Value) {
    if let Value::Object(map) = doc {
        map.retain(|k, _| !is_system_field(k));
    }
}
