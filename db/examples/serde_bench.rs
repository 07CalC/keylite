use std::time::Instant;
use serde_json::Value;

fn main() {
    println!("=== Serialization Benchmark ===\n");
    
    let doc = serde_json::json!({
        "name": "Alice Smith",
        "email": "alice@example.com",
        "age": 30,
        "address": {
            "street": "123 Main St",
            "city": "Springfield",
            "zip": "12345"
        },
        "hobbies": ["reading", "coding", "gaming"],
        "active": true
    });
    
    let n = 100_000;
    
    // Benchmark rmp_serde
    let start = Instant::now();
    for _ in 0..n {
        let bytes = rmp_serde::to_vec(&doc).unwrap();
        let _: Value = rmp_serde::from_slice(&bytes).unwrap();
    }
    let rmp_time = start.elapsed();
    
    // Benchmark serde_json
    let start = Instant::now();
    for _ in 0..n {
        let bytes = serde_json::to_vec(&doc).unwrap();
        let _: Value = serde_json::from_slice(&bytes).unwrap();
    }
    let json_time = start.elapsed();
    
    // Size comparison
    let rmp_bytes = rmp_serde::to_vec(&doc).unwrap();
    let json_bytes = serde_json::to_vec(&doc).unwrap();
    
    println!("Iterations: {}", n);
    println!("\nrmp_serde (MessagePack):");
    println!("  Time: {:?}", rmp_time);
    println!("  Per operation: {:.2} µs", rmp_time.as_micros() as f64 / n as f64);
    println!("  Size: {} bytes", rmp_bytes.len());
    
    println!("\nserde_json (JSON):");
    println!("  Time: {:?}", json_time);
    println!("  Per operation: {:.2} µs", json_time.as_micros() as f64 / n as f64);
    println!("  Size: {} bytes", json_bytes.len());
    
    println!("\nSpeedup: {:.2}x", json_time.as_secs_f64() / rmp_time.as_secs_f64());
    println!("Space savings: {:.1}%", (1.0 - rmp_bytes.len() as f64 / json_bytes.len() as f64) * 100.0);
}
