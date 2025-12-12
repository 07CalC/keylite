use keylite_kv::core::Db;
use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_simple_snapshot_isolation() {
    let test_dir = "/tmp/test_simple_snapshot";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());

    db.put(b"key", b"v0").unwrap();

    let txn = db.begin();

    let read1 = txn.get(b"key").unwrap();
    println!("Read 1: {:?}", read1);

    let db_clone = Arc::clone(&db);
    let writer = thread::spawn(move || {
        for i in 1..10 {
            let value = format!("v{}", i);
            db_clone.put(b"key", value.as_bytes()).unwrap();
            thread::sleep(Duration::from_millis(1));
        }
    });

    for _ in 0..50 {
        thread::sleep(Duration::from_millis(1));
        let read = txn.get(b"key").unwrap();
        println!("Read: {:?}", read);
        assert_eq!(read, read1, "Snapshot isolation violated!");
    }

    writer.join().unwrap();

    let _ = fs::remove_dir_all(test_dir);
}
