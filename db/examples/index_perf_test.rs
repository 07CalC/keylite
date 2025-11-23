use keylite_db::{collection::Index, db::KeyLite};
use std::time::Instant;

fn main() {
    println!("=== Insert Performance: Unique vs Non-Unique Indexes ===\n");
    
    let n = 5000;
    
    // Test 1: Only non-unique indexes
    println!("Test 1: Non-unique indexes only ({} docs)", n);
    let _ = std::fs::remove_dir_all("test_nonunique");
    let db = KeyLite::open("test_nonunique").unwrap();
    
    let indexes = vec![
        Index {
            field: "name".to_string(),
            unique: false,
        },
        Index {
            field: "status".to_string(),
            unique: false,
        },
    ];
    db.create_collection("users", Some(indexes)).unwrap();
    
    let start = Instant::now();
    for i in 0..n {
        let doc = serde_json::json!({
            "name": format!("User{}", i % 100),
            "status": if i % 3 == 0 { "active" } else { "inactive" },
            "age": 20 + (i % 50),
        });
        db.insert("users", doc).unwrap();
    }
    let nonunique_time = start.elapsed();
    println!("  Time: {:?}", nonunique_time);
    println!("  Throughput: {:.0} docs/sec\n", n as f64 / nonunique_time.as_secs_f64());
    
    // Test 2: With unique index
    println!("Test 2: With 1 unique index ({} docs)", n);
    let _ = std::fs::remove_dir_all("test_unique");
    let db = KeyLite::open("test_unique").unwrap();
    
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
    
    let start = Instant::now();
    for i in 0..n {
        let doc = serde_json::json!({
            "name": format!("User{}", i % 100),
            "email": format!("user{}@test.com", i),
            "age": 20 + (i % 50),
        });
        db.insert("users", doc).unwrap();
    }
    let unique_time = start.elapsed();
    println!("  Time: {:?}", unique_time);
    println!("  Throughput: {:.0} docs/sec\n", n as f64 / unique_time.as_secs_f64());
    
    // Test 3: No indexes
    println!("Test 3: No indexes ({} docs)", n);
    let _ = std::fs::remove_dir_all("test_noindex");
    let db = KeyLite::open("test_noindex").unwrap();
    db.create_collection("users", None).unwrap();
    
    let start = Instant::now();
    for i in 0..n {
        let doc = serde_json::json!({
            "name": format!("User{}", i % 100),
            "email": format!("user{}@test.com", i),
            "age": 20 + (i % 50),
        });
        db.insert("users", doc).unwrap();
    }
    let noindex_time = start.elapsed();
    println!("  Time: {:?}", noindex_time);
    println!("  Throughput: {:.0} docs/sec\n", n as f64 / noindex_time.as_secs_f64());
    
    println!("Slowdown from indexes:");
    println!("  Unique index: {:.2}x slower", unique_time.as_secs_f64() / noindex_time.as_secs_f64());
    println!("  Non-unique indexes: {:.2}x slower", nonunique_time.as_secs_f64() / noindex_time.as_secs_f64());
    
    // Cleanup
    let _ = std::fs::remove_dir_all("test_nonunique");
    let _ = std::fs::remove_dir_all("test_unique");
    let _ = std::fs::remove_dir_all("test_noindex");
}
