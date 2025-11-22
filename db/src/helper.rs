use serde_json::Value;

#[inline]
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}

pub(crate) fn index_prefix(col: &str, index: &str, value: &Value) -> (Vec<u8>, Vec<u8>) {
    let val = value_to_string(value);
    let start = format!("_priv:idx:{col}:{index}:{val}").into_bytes();
    let mut end = start.clone();
    end.push(0xFF);
    (start, end)
}

pub(crate) fn index_key(col: &str, index: &str, val: &Value, id: &str) -> Vec<u8> {
    let val = value_to_string(val);
    format!("_priv:idx:{col}:{index}:{val}:{id}").into_bytes()
}
