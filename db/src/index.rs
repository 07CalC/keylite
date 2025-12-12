use serde_json::Value;

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(v) => v.to_string(),
        _ => value.to_string(),
    }
}

pub fn unique_index(collection: &str, field: &str, value: &Value) -> Vec<u8> {
    let value = value_to_string(&value);
    format!("idx:u:{collection}:{field}:{value}").into_bytes()
}

pub fn non_unique_index(collection: &str, field: &str, value: &Value, id: &str) -> Vec<u8> {
    let value = value_to_string(&value);
    format!("idx:n:{collection}:{field}:{value}:{id}").into_bytes()
}

pub fn prefix_range(prefix: &str) -> (Vec<u8>, Vec<u8>) {
    let start = prefix.as_bytes().to_vec();
    let mut end = start.clone();
    end.push(0xFF);
    (start, end)
}
