use serde_json::Value;

pub mod collection;
pub mod db;
pub mod error;
pub mod filter;
mod index;
pub mod query;
pub mod transaction;

pub fn get_field<'a>(doc: &'a Value, field: &str) -> Option<&'a Value> {
    doc.as_object()?.get(field)
}
