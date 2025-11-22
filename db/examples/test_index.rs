use keylite_db::{collection::Index, db::KeyLite};

fn main() {
    let db = KeyLite::open("test_index_db").unwrap();
    
    let indexes = vec![
        Index {
            field: "name".to_string(),
            unique: false,
        },
        Index {
            field: "email".to_string(),
            unique: true,
        },
    ];
    
    db.create_collection("users", Some(indexes)).unwrap();
    
    let doc1 = serde_json::json!({
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30
    });
    
    let doc2 = serde_json::json!({
        "name": "Bob",
        "email": "bob@example.com",
        "age": 25
    });
    
    let doc3 = serde_json::json!({
        "name": "Alice",
        "email": "alice2@example.com",
        "age": 28
    });
    
    println!("Inserting docs...");
    let id1 = db.insert("users", doc1).unwrap();
    let id2 = db.insert("users", doc2).unwrap();
    let id3 = db.insert("users", doc3).unwrap();
    println!("Inserted: {}, {}, {}", id1, id2, id3);
    
    println!("\nQuerying by name = 'Alice':");
    let results = db.get_by_index("users", "name", "Alice".into()).unwrap();
    println!("Found {} docs", results.len());
    for doc in &results {
        println!("  - {}", doc);
    }
    
    println!("\nQuerying by email = 'bob@example.com':");
    let results = db.get_by_index("users", "email", "bob@example.com".into()).unwrap();
    println!("Found {} docs", results.len());
    for doc in &results {
        println!("  - {}", doc);
    }
    
    println!("\nTrying to insert duplicate email...");
    let duplicate = serde_json::json!({
        "name": "Charlie",
        "email": "alice@example.com",
        "age": 35
    });
    match db.insert("users", duplicate) {
        Ok(_) => println!("ERROR: Should have failed!"),
        Err(e) => println!("Correctly rejected: {}", e),
    }
}
