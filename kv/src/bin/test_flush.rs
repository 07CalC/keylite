use keylite_kv::core::Db;
use std::sync::Arc;

fn main() {
    println!("Testing immutable memtable tracking...\n");

    let _ = std::fs::remove_dir_all("test_flush_db");
    let db = Arc::new(Db::open("test_flush_db").unwrap());

    println!("Writing 100,000 entries with 500 byte values...");
    for i in 0..100_000 {
        let key = format!("key_{:010}", i);
        let val = format!("{:0>500}", i); // 500 byte value
        db.put(key.as_bytes(), val.as_bytes()).unwrap();

        if i % 10000 == 0 {
            println!("Written {} entries", i);
        }
    }

    println!("\nAll writes completed. DB will flush on drop...");
    drop(db);
    println!("\nTest complete!");
}
