use keylite_kv::db::Db;
use std::time::Instant;

// Linux-only: reads RSS from /proc/self/statm
fn get_memory_usage_kb() -> u64 {
    let statm = std::fs::read_to_string("/proc/self/statm").unwrap();
    let parts: Vec<&str> = statm.split_whitespace().collect();
    let rss_pages: u64 = parts[1].parse().unwrap();
    let page_size = 4096u64;
    rss_pages * page_size / 1024
}

fn main() {
    println!("=== KeyLite Memory Footprint Test ===");

    // 1. Clean previous directory
    let _ = std::fs::remove_dir_all("memtest_db");

    // 2. Capture memory before DB open
    let mem_before_open = get_memory_usage_kb();
    println!("Memory before open: {} KB", mem_before_open);

    // 3. Open DB
    let t0 = Instant::now();
    let db = Db::open("concurrency_test").unwrap();
    println!("DB open time: {:?}", t0.elapsed());

    // 4. Memory after DB open
    let mem_after_open = get_memory_usage_kb();
    println!("Memory after open: {} KB", mem_after_open);
    println!("DB allocated: {} KB", mem_after_open - mem_before_open);

    // 5. Drop DB explicitly
    drop(db);

    // 6. Force cleanup by GC / allocator
    std::thread::sleep(std::time::Duration::from_millis(200));

    // 7. Now measure memory after everything is dropped
    let mem_after_drop = get_memory_usage_kb();
    println!("Memory after drop: {} KB", mem_after_drop);

    println!(
        "Net DB memory retained after drop (should be ~0): {} KB",
        mem_after_drop - mem_before_open
    );

    println!("=== Done ===");
}
