use keylite_kv::core::Db;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_stress_sequential_operations() {
    let test_dir = "/tmp/test_stress_sequential";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    for i in 0..1000 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    for i in 0..1000 {
        let key = format!("key{}", i);
        let expected = format!("value{}", i);
        assert_eq!(
            db.get(key.as_bytes()).unwrap(),
            Some(expected.as_bytes().to_vec())
        );
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_stress_rapid_version_updates() {
    let test_dir = "/tmp/test_stress_versions";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());
    let mut handles = vec![];

    for thread_id in 0..5 {
        let db = Arc::clone(&db);

        handles.push(thread::spawn(move || {
            for i in 0..200 {
                let value = format!("t{}_v{}", thread_id, i);
                db.put(b"hot_key", value.as_bytes()).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert!(db.get(b"hot_key").unwrap().is_some());

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_stress_write_amplification() {
    let test_dir = "/tmp/test_stress_amplification";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    for round in 0..10 {
        for i in 0..1000 {
            let key = format!("key{}", i);
            let value = format!("round{}_value{}", round, i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
    }

    for i in 0..1000 {
        let key = format!("key{}", i);
        let expected = format!("round9_value{}", i);
        assert_eq!(
            db.get(key.as_bytes()).unwrap(),
            Some(expected.as_bytes().to_vec())
        );
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_stress_transaction_abort_rate() {
    let test_dir = "/tmp/test_stress_abort";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());
    let mut handles = vec![];

    for thread_id in 0..5 {
        let db = Arc::clone(&db);

        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let mut txn = db.begin();
                let key = format!("key_t{}_i{}", thread_id, i);
                txn.put(key.as_bytes(), b"value");

                // Abort half the transactions
                if i % 2 == 0 {
                    txn.abort();
                } else {
                    txn.commit().unwrap();
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut count = 0;
    for thread_id in 0..5 {
        for i in 0..100 {
            let key = format!("key_t{}_i{}", thread_id, i);
            if db.get(key.as_bytes()).unwrap().is_some() {
                count += 1;
            }
        }
    }

    assert!(count >= 240 && count <= 260, "Expected ~250, got {}", count);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_stress_persistence_under_load() {
    let test_dir = "/tmp/test_stress_persistence";
    let _ = fs::remove_dir_all(test_dir);

    let expected = Arc::new(Mutex::new(HashMap::new()));

    {
        let db = Arc::new(Db::open(test_dir).unwrap());
        let mut handles = vec![];

        for thread_id in 0..5 {
            let db = Arc::clone(&db);
            let expected = Arc::clone(&expected);

            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    let key = format!("persist_t{}_k{}", thread_id, i);
                    let value = format!("value_{}", i);

                    db.put(key.as_bytes(), value.as_bytes()).unwrap();

                    let mut exp = expected.lock().unwrap();
                    exp.insert(key.clone(), value.clone());
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    thread::sleep(std::time::Duration::from_millis(200));

    {
        let db = Db::open(test_dir).unwrap();
        let exp = expected.lock().unwrap();

        for (key, value) in exp.iter() {
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(value.as_bytes().to_vec()),
                "Key {} mismatch after recovery",
                key
            );
        }
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_isolation_read_committed() {
    let test_dir = "/tmp/test_isolation_read_committed";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());

    db.put(b"account_a", b"100").unwrap();
    db.put(b"account_b", b"100").unwrap();

    let txn = db.begin();

    db.put(b"account_a", b"50").unwrap();
    db.put(b"account_b", b"150").unwrap();

    assert_eq!(txn.get(b"account_a").unwrap(), Some(b"100".to_vec()));
    assert_eq!(txn.get(b"account_b").unwrap(), Some(b"100".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_isolation_phantom_reads_prevented() {
    let test_dir = "/tmp/test_isolation_phantom";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"item1", b"exists").unwrap();
    db.put(b"item2", b"exists").unwrap();

    let txn = db.begin();

    let read1_item3 = txn.get(b"item3").unwrap();

    db.put(b"item3", b"new_item").unwrap();

    let read2_item3 = txn.get(b"item3").unwrap();

    assert_eq!(read1_item3, None);
    assert_eq!(read2_item3, None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_isolation_non_repeatable_reads_prevented() {
    let test_dir = "/tmp/test_isolation_non_repeatable";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"balance", b"1000").unwrap();

    let txn = db.begin();
    let read1 = txn.get(b"balance").unwrap();

    db.put(b"balance", b"500").unwrap();
    let read2 = txn.get(b"balance").unwrap();

    assert_eq!(read1, Some(b"1000".to_vec()));
    assert_eq!(read2, Some(b"1000".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}
