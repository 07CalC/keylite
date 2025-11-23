use keylite_db::{collection::Index, db::KeyLite};
use std::time::Instant;

fn main() {
    println!("=== Detailed Performance Profiling ===\n");
    
    // Clean up
    let _ = std::fs::remove_dir_all("profile_db");
    
    let db = KeyLite::open("profile_db").unwrap();
    
    // Test 1: Collection creation overhead
    let start = Instant::now();
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
    println!("Collection creation: {:?}\n", start.elapsed());
    
    // Test 2: Single insert with indexes
    let doc = serde_json::json!({
        "name": "User0",
        "email": "user0@test.com",
        "age": 20,
        "status": "active"
    });
    
    let start = Instant::now();
    db.insert("users", doc).unwrap();
    let single_insert = start.elapsed();
    println!("Single insert (first): {:?}", single_insert);
    
    // Test 3: Insert with existing unique value check
    let doc = serde_json::json!({
        "name": "User0",
        "email": "user1@test.com",
        "age": 21,
    });
    
    let start = Instant::now();
    db.insert("users", doc).unwrap();
    println!("Single insert (subsequent): {:?}\n", start.elapsed());
    
    // Test 4: Batch insert performance
    println!("Inserting 100 documents...");
    let start = Instant::now();
    for i in 2..102 {
        let doc = serde_json::json!({
            "name": format!("User{}", i % 10),
            "email": format!("user{}@test.com", i),
            "age": 20 + (i % 50),
        });
        db.insert("users", doc).unwrap();
    }
    let batch_time = start.elapsed();
    println!("100 inserts: {:?} ({:.2} ms per insert)\n", batch_time, batch_time.as_secs_f64() * 1000.0 / 100.0);
    
    // Test 5: Query by non-unique index
    let start = Instant::now();
    let results = db.get_by_index("users", "name", "User5".into()).unwrap();
    println!("Query by non-unique index: {:?} (found {} docs)", start.elapsed(), results.len());
    
    // Test 6: Query by unique index
    let start = Instant::now();
    let results = db.get_by_index("users", "email", "user50@test.com".into()).unwrap();
    println!("Query by unique index: {:?} (found {} docs)", start.elapsed(), results.len());
    
    // Test 7: Repeated queries (caching test)
    let start = Instant::now();
    for i in 0..100 {
        let name = format!("User{}", i % 10);
        let _ = db.get_by_index("users", "name", name.into()).unwrap();
    }
    println!("100 queries (mixed): {:?} ({:.2} ms per query)\n", start.elapsed(), start.elapsed().as_secs_f64() * 1000.0 / 100.0);
    
    // Test 8: Full scan
    let start = Instant::now();
    let all = db.scan_collection("users").unwrap();
    println!("Full collection scan: {:?} ({} docs)", start.elapsed(), all.len());
    
    // Clean up
    let _ = std::fs::remove_dir_all("profile_db");
}
