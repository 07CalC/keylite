use keylite_kv::db::Db;
use std::time::Instant;

fn main() {
    println!("=== KeyLite Scan Functionality Test ===\n");

    // Clean up any previous test data
    let _ = std::fs::remove_dir_all("testdb");

    // Open the database
    let t0 = Instant::now();
    let db = Db::open("testdb").unwrap();
    let open_time = t0.elapsed();
    println!("✓ Database opened in {:?}\n", open_time);

    // Test 1: Insert some sample data
    println!("--- Test 1: Inserting Sample Data ---");
    let t1 = Instant::now();

    db.put(b"user:1001", b"Alice").unwrap();
    db.put(b"user:1002", b"Bob").unwrap();
    db.put(b"user:1003", b"Charlie").unwrap();
    db.put(b"user:1004", b"Diana").unwrap();
    db.put(b"user:1005", b"Eve").unwrap();

    db.put(b"post:2001", b"Hello World").unwrap();
    db.put(b"post:2002", b"Goodbye World").unwrap();

    db.put(b"score:3001", b"95").unwrap();
    db.put(b"score:3002", b"87").unwrap();
    db.put(b"score:3003", b"92").unwrap();

    let insert_time = t1.elapsed();
    println!("✓ Inserted 10 keys in {:?}\n", insert_time);

    // Test 2: Scan all keys
    println!("--- Test 2: Scan All Keys ---");
    let t2 = Instant::now();
    let iter = db.scan(None, None);
    let results: Vec<_> = iter.collect();
    let scan_all_time = t2.elapsed();

    println!("Found {} keys in {:?}", results.len(), scan_all_time);
    for (key, value) in &results {
        println!(
            "  {} => {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }
    println!();

    // Test 3: Scan with prefix (all users)
    println!("--- Test 3: Scan Users Only (user:) ---");
    let t3 = Instant::now();
    let iter = db.scan(Some(b"user:"), Some(b"user;"));
    let user_results: Vec<_> = iter.collect();
    let scan_users_time = t3.elapsed();

    println!(
        "Found {} users in {:?}",
        user_results.len(),
        scan_users_time
    );
    for (key, value) in &user_results {
        println!(
            "  {} => {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }
    println!();

    // Test 4: Scan with range
    println!("--- Test 4: Scan Range (user:1002 to user:1004) ---");
    let t4 = Instant::now();
    let iter = db.scan(Some(b"user:1002"), Some(b"user:1004"));
    let range_results: Vec<_> = iter.collect();
    let scan_range_time = t4.elapsed();

    println!(
        "Found {} keys in range in {:?}",
        range_results.len(),
        scan_range_time
    );
    for (key, value) in &range_results {
        println!(
            "  {} => {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }
    println!();

    // Test 5: Update some values and scan again
    println!("--- Test 5: Update Values and Rescan ---");
    db.put(b"user:1002", b"Bob Smith").unwrap();
    db.put(b"user:1003", b"Charlie Brown").unwrap();

    let t5 = Instant::now();
    let iter = db.scan(Some(b"user:"), Some(b"user;"));
    let updated_results: Vec<_> = iter.collect();
    let scan_updated_time = t5.elapsed();

    println!(
        "Found {} users after updates in {:?}",
        updated_results.len(),
        scan_updated_time
    );
    for (key, value) in &updated_results {
        println!(
            "  {} => {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }
    println!();

    // Test 6: Delete a key and scan
    println!("--- Test 6: Delete Key and Rescan ---");
    db.del(b"user:1003").unwrap();

    let t6 = Instant::now();
    let iter = db.scan(Some(b"user:"), Some(b"user;"));
    let after_delete_results: Vec<_> = iter.collect();
    let scan_after_delete_time = t6.elapsed();

    println!(
        "Found {} users after deletion in {:?}",
        after_delete_results.len(),
        scan_after_delete_time
    );
    for (key, value) in &after_delete_results {
        println!(
            "  {} => {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }
    println!();

    // Test 7: Large dataset scan
    println!("--- Test 7: Large Dataset Scan ---");
    let t7 = Instant::now();

    for i in 0..1000 {
        let key = format!("item:{:05}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }
    let insert_large_time = t7.elapsed();
    println!("✓ Inserted 1000 items in {:?}", insert_large_time);

    // Scan a subset
    let t8 = Instant::now();
    let iter = db.scan(Some(b"item:00100"), Some(b"item:00200"));
    let large_results: Vec<_> = iter.collect();
    let scan_large_time = t8.elapsed();

    println!(
        "Found {} items in range [item:00100, item:00200) in {:?}",
        large_results.len(),
        scan_large_time
    );
    println!("First 5 items:");
    for (key, value) in large_results.iter().take(5) {
        println!(
            "  {} => {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }
    println!("Last 5 items:");
    for (key, value) in large_results.iter().rev().take(5).rev() {
        println!(
            "  {} => {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }
    println!();

    // Test 8: Scan all items (full scan performance)
    println!("--- Test 8: Full Scan Performance ---");
    let t9 = Instant::now();
    let iter = db.scan(None, None);
    let all_results: Vec<_> = iter.collect();
    let full_scan_time = t9.elapsed();

    println!(
        "Full scan returned {} total keys in {:?}",
        all_results.len(),
        full_scan_time
    );
    println!(
        "Average time per key: {:?}",
        full_scan_time / all_results.len() as u32
    );
    println!();

    // Test 9: Empty range scan
    println!("--- Test 9: Empty Range Scan ---");
    let t10 = Instant::now();
    let iter = db.scan(Some(b"zzz:"), Some(b"zzz;"));
    let empty_results: Vec<_> = iter.collect();
    let empty_scan_time = t10.elapsed();

    println!(
        "Empty range scan returned {} keys in {:?}",
        empty_results.len(),
        empty_scan_time
    );
    println!();

    // Summary
    println!("=== Performance Summary ===");
    println!("Database open:        {:?}", open_time);
    println!("Insert 10 keys:       {:?}", insert_time);
    println!("Scan all (10 keys):   {:?}", scan_all_time);
    println!("Scan prefix:          {:?}", scan_users_time);
    println!("Scan range:           {:?}", scan_range_time);
    println!("Insert 1000 keys:     {:?}", insert_large_time);
    println!("Scan subset (100):    {:?}", scan_large_time);
    println!("Full scan (1010):     {:?}", full_scan_time);
    println!("Empty scan:           {:?}", empty_scan_time);
    println!();

    // Clean up
    drop(db);
    println!("✓ Database closed successfully");
    println!("\n=== Test Complete ===");
}
