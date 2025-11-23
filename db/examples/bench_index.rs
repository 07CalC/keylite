use keylite_db::{collection::Index, db::KeyLite};
use std::time::Instant;

fn main() {
    println!("=== Index Performance Benchmark ===\n");
    
    // Clean up any existing test db
    let _ = std::fs::remove_dir_all("bench_db");
    
    let db = KeyLite::open("bench_db").unwrap();
    
    // Create collection with 2 indexes (1 unique, 1 non-unique)
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
    println!("Created collection with 2 indexes (name: non-unique, email: unique)\n");
    
    // Benchmark: Insert 1000 documents
    let n = 1000;
    println!("Benchmark 1: Inserting {} documents", n);
    let start = Instant::now();
    
    for i in 0..n {
        let doc = serde_json::json!({
            "name": format!("User{}", i % 100),  // 100 unique names, 10 docs per name
            "email": format!("user{}@test.com", i),  // All unique emails
            "age": 20 + (i % 50),
            "status": if i % 3 == 0 { "active" } else { "inactive" }
        });
        db.insert("users", doc).unwrap();
    }
    
    let insert_time = start.elapsed();
    println!("  Total time: {:?}", insert_time);
    println!("  Per document: {:?}", insert_time / n);
    println!("  Throughput: {:.0} docs/sec\n", n as f64 / insert_time.as_secs_f64());
    
    // Benchmark: get_by_index on non-unique index (returns 10 docs)
    println!("Benchmark 2: Query by non-unique index (name = 'User50')");
    let start = Instant::now();
    let results = db.get_by_index("users", "name", "User50".into()).unwrap();
    let query_time = start.elapsed();
    println!("  Found {} documents", results.len());
    println!("  Query time: {:?}\n", query_time);
    
    // Benchmark: get_by_index on unique index (returns 1 doc)
    println!("Benchmark 3: Query by unique index (email = 'user500@test.com')");
    let start = Instant::now();
    let results = db.get_by_index("users", "email", "user500@test.com".into()).unwrap();
    let query_time = start.elapsed();
    println!("  Found {} documents", results.len());
    println!("  Query time: {:?}\n", query_time);
    
    // Benchmark: Full collection scan
    println!("Benchmark 4: Full collection scan");
    let start = Instant::now();
    let all = db.scan_collection("users").unwrap();
    let scan_time = start.elapsed();
    println!("  Total documents: {}", all.len());
    println!("  Scan time: {:?}", scan_time);
    println!("  Throughput: {:.0} docs/sec\n", all.len() as f64 / scan_time.as_secs_f64());
    
    // Benchmark: Multiple queries
    println!("Benchmark 5: 100 random queries by index");
    let start = Instant::now();
    for i in 0..100 {
        let name = format!("User{}", i % 100);
        let _ = db.get_by_index("users", "name", name.into()).unwrap();
    }
    let multi_query_time = start.elapsed();
    println!("  Total time: {:?}", multi_query_time);
    println!("  Per query: {:?}", multi_query_time / 100);
    println!("  Throughput: {:.0} queries/sec\n", 100.0 / multi_query_time.as_secs_f64());
    
    println!("✓ Benchmark completed!");
    
    // Clean up
    let _ = std::fs::remove_dir_all("bench_db");
}
