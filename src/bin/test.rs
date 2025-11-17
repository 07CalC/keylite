use keylite::db::Db;
use std::time::Instant;

fn main() {
    println!("=== KeyLite Performance Test ===\n");

    // Clean up
    let _ = std::fs::remove_dir_all("perftest");

    // Open database
    let t = Instant::now();
    let db = Db::open("perftest").unwrap();
    println!("Database open: {:?}", t.elapsed());

    // Write test
    let n = 100_000;
    println!("\n--- Write Test ({} ops) ---", n);
    let t = Instant::now();
    for i in 0..n {
        let key = format!("key{:08}", i);
        let val = format!("value{:08}_some_data_here", i);
        db.put(key.as_bytes(), val.as_bytes()).unwrap();
    }
    let elapsed = t.elapsed();
    println!("Total time: {:?}", elapsed);
    println!(
        "Throughput: {:.2} ops/sec",
        n as f64 / elapsed.as_secs_f64()
    );
    println!("Avg latency: {:?}", elapsed / n);

    // Sequential read test
    println!("\n--- Sequential Read Test ({} ops) ---", n);
    let t = Instant::now();
    for i in 0..n {
        let key = format!("key{:08}", i);
        let _ = db.get(key.as_bytes()).unwrap();
    }
    let elapsed = t.elapsed();
    println!("Total time: {:?}", elapsed);
    println!(
        "Throughput: {:.2} ops/sec",
        n as f64 / elapsed.as_secs_f64()
    );
    println!("Avg latency: {:?}", elapsed / n);

    // Random read test
    println!("\n--- Random Read Test ({} ops) ---", n);
    let t = Instant::now();
    for i in 0..n {
        let idx = (i * 7919) % n; // pseudo-random
        let key = format!("key{:08}", idx);
        let _ = db.get(key.as_bytes()).unwrap();
    }
    let elapsed = t.elapsed();
    println!("Total time: {:?}", elapsed);
    println!(
        "Throughput: {:.2} ops/sec",
        n as f64 / elapsed.as_secs_f64()
    );
    println!("Avg latency: {:?}", elapsed / n);

    // Negative lookup test (bloom filter efficiency)
    println!("\n--- Negative Lookup Test (bloom filter) ---");
    let t = Instant::now();
    for i in 0..10000 {
        let key = format!("nonexistent{:08}", i);
        let _ = db.get(key.as_bytes());
    }
    let elapsed = t.elapsed();
    println!("Total time: {:?}", elapsed);
    println!("Throughput: {:.2} ops/sec", 10000.0 / elapsed.as_secs_f64());

    println!("\n=== Test Complete ===");
}
