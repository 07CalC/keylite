use std::time::Instant;

use keylite_db::{collection::Index, db::KeyLite};

fn main() {
    let db = KeyLite::open("testdb").unwrap();
    
    let mut indexes = Vec::new();
    indexes.push(Index {
        field: "name".to_string(),
        unique: false,
    });
    indexes.push(Index {
        field: "email".to_string(),
        unique: true,
    });
    
    let t1 = Instant::now();
    db.create_collection("users", Some(indexes)).unwrap();
    println!("time taken to create_collection: {:?}", t1.elapsed());

    let t2 = Instant::now();
    for i in 0..1000 {
        let doc = serde_json::json!({
            "name": format!("user_{}", i % 100),
            "email": format!("user_{}@example.com", i),
            "class": "low class",
            "age": 10 + (i % 50)
        });
        let _ = db.insert("users", doc).unwrap();
    }
    println!("time taken to insert 1000 docs: {:?}", t2.elapsed());

    let t3 = Instant::now();
    let val = db.scan_collection("users").unwrap();
    println!(
        "time taken to scan {:?} users: {:?}",
        val.len(),
        t3.elapsed()
    );

    let t4 = Instant::now();
    let results = db.get_by_index("users", "name", "user_50".into()).unwrap();
    println!(
        "time taken to get docs by name index: {:?}, found: {} docs",
        t4.elapsed(),
        results.len()
    );

    let t5 = Instant::now();
    let results = db.get_by_index("users", "email", "user_500@example.com".into()).unwrap();
    println!(
        "time taken to get doc by unique email index: {:?}, found: {} doc",
        t5.elapsed(),
        results.len()
    );

    println!("\nTesting unique constraint...");
    let duplicate_doc = serde_json::json!({
        "name": "test_user",
        "email": "user_500@example.com",
        "age": 25
    });
    match db.insert("users", duplicate_doc) {
        Ok(_) => println!("ERROR: Should have failed unique constraint!"),
        Err(e) => println!("Correctly rejected duplicate email: {}", e),
    }

    println!("\nTesting delete with index cleanup...");
    let first_result = db.get_by_index("users", "email", "user_100@example.com".into()).unwrap();
    if !first_result.is_empty() {
        let id = first_result[0].get("_id").unwrap().as_str().unwrap();
        db.delete_doc_by_id("users", id).unwrap();
        println!("Deleted doc with id: {}", id);
        
        let after_delete = db.get_by_index("users", "email", "user_100@example.com".into()).unwrap();
        println!("After delete, found {} docs (should be 0)", after_delete.len());
    }
}
