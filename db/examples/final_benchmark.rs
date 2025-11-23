use keylite_db::{collection::Index, db::KeyLite};
use std::time::Instant;

fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║     KEYLITE PERFORMANCE BENCHMARK - FINAL RESULTS         ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    
    // Clean up
    let _ = std::fs::remove_dir_all("final_bench");
    
    let db = KeyLite::open("final_bench").unwrap();
    
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
    
    // Test 1: Small batch (1K)
    println!("📊 Test 1: Insert 1,000 documents (with 2 indexes)");
    let start = Instant::now();
    for i in 0..1000 {
        let doc = serde_json::json!({
            "name": format!("User{}", i % 100),
            "email": format!("user{}@test.com", i),
            "age": 20 + (i % 50),
            "city": if i % 3 == 0 { "NYC" } else { "SF" },
            "score": i as f64 * 1.234,
        });
        db.insert("users", doc).unwrap();
    }
    let time_1k = start.elapsed();
    println!("   ⏱️  Time: {:?}", time_1k);
    println!("   🚀 Throughput: {:.0} docs/sec\n", 1000.0 / time_1k.as_secs_f64());
    
    // Test 2: Query performance
    println!("📊 Test 2: Query by non-unique index");
    let start = Instant::now();
    let results = db.get_by_index("users", "name", "User50".into()).unwrap();
    let query_time = start.elapsed();
    println!("   ⏱️  Time: {:?}", query_time);
    println!("   📦 Found {} documents\n", results.len());
    
    println!("📊 Test 3: Query by unique index");
    let start = Instant::now();
    let results = db.get_by_index("users", "email", "user500@test.com".into()).unwrap();
    let unique_query_time = start.elapsed();
    println!("   ⏱️  Time: {:?}", unique_query_time);
    println!("   📦 Found {} documents\n", results.len());
    
    // Test 4: Full scan
    println!("📊 Test 4: Full collection scan");
    let start = Instant::now();
    let all = db.scan_collection("users").unwrap();
    let scan_time = start.elapsed();
    println!("   ⏱️  Time: {:?}", scan_time);
    println!("   📦 Total: {} documents", all.len());
    println!("   🚀 Throughput: {:.0} docs/sec\n", all.len() as f64 / scan_time.as_secs_f64());
    
    // Test 5: Large batch
    println!("📊 Test 5: Insert 9,000 more documents (total: 10,000)");
    let start = Instant::now();
    for i in 1000..10000 {
        let doc = serde_json::json!({
            "name": format!("User{}", i % 100),
            "email": format!("user{}@test.com", i),
            "age": 20 + (i % 50),
            "city": if i % 3 == 0 { "NYC" } else { "SF" },
            "score": i as f64 * 1.234,
        });
        db.insert("users", doc).unwrap();
    }
    let time_9k = start.elapsed();
    println!("   ⏱️  Time: {:?}", time_9k);
    println!("   🚀 Throughput: {:.0} docs/sec\n", 9000.0 / time_9k.as_secs_f64());
    
    // Test 6: Query performance at scale
    println!("📊 Test 6: 100 queries at scale (10K docs)");
    let start = Instant::now();
    for i in 0..100 {
        let name = format!("User{}", i % 100);
        let _ = db.get_by_index("users", "name", name.into()).unwrap();
    }
    let queries_100 = start.elapsed();
    println!("   ⏱️  Time: {:?}", queries_100);
    println!("   📈 Per query: {:?}", queries_100 / 100);
    println!("   🚀 Throughput: {:.0} queries/sec\n", 100.0 / queries_100.as_secs_f64());
    
    // Summary
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                      SUMMARY                              ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("📝 Total documents inserted: 10,000");
    println!("📝 Indexes: 2 (1 unique, 1 non-unique)");
    println!("📝 Average insert throughput: {:.0} docs/sec", 10000.0 / (time_1k + time_9k).as_secs_f64());
    println!("📝 Query latency (non-unique): {:?}", query_time);
    println!("📝 Query latency (unique): {:?}", unique_query_time);
    println!("📝 Scan throughput: {:.0} docs/sec", 1000.0 / scan_time.as_secs_f64());
    
    // Clean up
    let _ = std::fs::remove_dir_all("final_bench");
}
