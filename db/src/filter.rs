use serde_json::Value;

use crate::get_field;

pub enum Filter {
    Eq { field: String, value: Value },
    Gt { field: String, value: Value },
    Lt { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    Exists { field: String },
}

impl Filter {
    pub fn matches(&self, doc: &Value) -> bool {
        match self {
            Filter::Eq { field, value } => {
                get_field(&doc, field).map(|v| v == value).unwrap_or(false)
            }

            Filter::Gt { field, value } => match (get_field(&doc, field), value) {
                (Some(Value::Number(a)), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
                    (Some(a_f), Some(b_f)) => a_f > b_f,
                    _ => false,
                },
                (Some(Value::String(a)), Value::String(b)) => a > b,
                _ => false,
            },
            Filter::Lt { field, value } => match (get_field(&doc, field), value) {
                (Some(Value::Number(a)), Value::Number(b)) => match (a.as_f64(), b.as_f64()) {
                    (Some(a_f), Some(b_f)) => a_f < b_f,
                    _ => false,
                },
                (Some(Value::String(a)), Value::String(b)) => a < b,
                _ => false,
            },
            Filter::In { field, values } => get_field(&doc, field)
                .map(|v| values.contains(v))
                .unwrap_or(false),
            Filter::Exists { field } => get_field(&doc, field).is_some(),
        }
    }
}
