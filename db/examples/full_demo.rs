use keylite_db::{collection::Index, db::KeyLite};

fn main() {
    println!("=== Secondary Index Full Demo ===\n");
    
    let db = KeyLite::open("demo_db").unwrap();
    
    println!("1. Creating collection with indexes:");
    let indexes = vec![
        Index {
            field: "name".to_string(),
            unique: false,
        },
        Index {
            field: "email".to_string(),
            unique: true,
        },
        Index {
            field: "status".to_string(),
            unique: false,
        },
    ];
    db.create_collection("users", Some(indexes)).unwrap();
    println!("   Created collection 'users' with 3 indexes\n");
    
    println!("2. Inserting documents:");
    let users = vec![
        serde_json::json!({"name": "Alice", "email": "alice@test.com", "status": "active", "age": 30}),
        serde_json::json!({"name": "Bob", "email": "bob@test.com", "status": "active", "age": 25}),
        serde_json::json!({"name": "Alice", "email": "alice2@test.com", "status": "inactive", "age": 28}),
        serde_json::json!({"name": "Charlie", "email": "charlie@test.com", "status": "active", "age": 35}),
    ];
    
    for user in users {
        let id = db.insert("users", user).unwrap();
        println!("   Inserted user with id: {}", id);
    }
    
    println!("\n3. Query by name (non-unique index):");
    let results = db.get_by_index("users", "name", "Alice".into()).unwrap();
    println!("   Found {} users named Alice:", results.len());
    for doc in &results {
        println!("     - email: {}, status: {}", 
            doc.get("email").unwrap().as_str().unwrap(),
            doc.get("status").unwrap().as_str().unwrap()
        );
    }
    
    println!("\n4. Query by email (unique index):");
    let results = db.get_by_index("users", "email", "bob@test.com".into()).unwrap();
    println!("   Found {} user with email bob@test.com:", results.len());
    for doc in &results {
        println!("     - name: {}, age: {}", 
            doc.get("name").unwrap().as_str().unwrap(),
            doc.get("age").unwrap().as_u64().unwrap()
        );
    }
    
    println!("\n5. Query by status:");
    let results = db.get_by_index("users", "status", "active".into()).unwrap();
    println!("   Found {} active users:", results.len());
    for doc in &results {
        println!("     - name: {}", doc.get("name").unwrap().as_str().unwrap());
    }
    
    println!("\n6. Testing unique constraint:");
    let duplicate = serde_json::json!({"name": "Test", "email": "alice@test.com", "status": "active"});
    match db.insert("users", duplicate) {
        Ok(_) => println!("   ERROR: Should have failed!"),
        Err(e) => println!("   ✓ Correctly rejected: {}", e),
    }
    
    println!("\n7. Testing delete with index cleanup:");
    let alice_results = db.get_by_index("users", "email", "alice@test.com".into()).unwrap();
    if !alice_results.is_empty() {
        let id = alice_results[0].get("_id").unwrap().as_str().unwrap();
        println!("   Deleting user with id: {}", id);
        db.delete_doc_by_id("users", id).unwrap();
        
        let after = db.get_by_index("users", "email", "alice@test.com".into()).unwrap();
        println!("   ✓ After delete: {} docs found (should be 0)", after.len());
        
        let by_name = db.get_by_index("users", "name", "Alice".into()).unwrap();
        println!("   ✓ Other Alice doc still exists: {} docs", by_name.len());
    }
    
    println!("\n8. Full scan to see all remaining docs:");
    let all = db.scan_collection("users").unwrap();
    println!("   Total documents: {}", all.len());
    for doc in &all {
        println!("     - {}: {}", 
            doc.get("name").unwrap().as_str().unwrap(),
            doc.get("email").unwrap().as_str().unwrap()
        );
    }
    
    println!("\n✓ All operations completed successfully!");
}
