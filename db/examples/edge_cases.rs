use keylite_db::{collection::Index, db::KeyLite};

fn main() {
    let db = KeyLite::open("test_edge_cases").unwrap();
    
    println!("Test 1: Collection without indexes");
    db.create_collection("no_index", None).unwrap();
    db.insert("no_index", serde_json::json!({"name": "test"})).unwrap();
    let results = db.get_by_index("no_index", "name", "test".into()).unwrap();
    println!("Found {} docs (expected 0 since no index defined)\n", results.len());
    
    println!("Test 2: Query with no matches");
    let indexes = vec![Index { field: "status".to_string(), unique: false }];
    db.create_collection("orders", Some(indexes)).unwrap();
    db.insert("orders", serde_json::json!({"status": "pending", "amount": 100})).unwrap();
    let results = db.get_by_index("orders", "status", "completed".into()).unwrap();
    println!("Found {} docs (expected 0)\n", results.len());
    
    println!("Test 3: Multiple docs with same index value");
    let idx = vec![Index { field: "category".to_string(), unique: false }];
    db.create_collection("products", Some(idx)).unwrap();
    for i in 0..5 {
        db.insert("products", serde_json::json!({"category": "electronics", "id": i})).unwrap();
    }
    let results = db.get_by_index("products", "category", "electronics".into()).unwrap();
    println!("Found {} electronics products\n", results.len());
    
    println!("Test 4: Doc without indexed field gets null index");
    db.insert("products", serde_json::json!({"id": 99})).unwrap();
    let results = db.get_by_index("products", "category", serde_json::Value::Null).unwrap();
    println!("Found {} products with null category\n", results.len());
    
    println!("All tests completed!");
}
