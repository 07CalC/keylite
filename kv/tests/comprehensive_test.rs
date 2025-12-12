use keylite_kv::core::Db;
use std::fs;
use std::sync::Arc;
use std::thread;

#[test]
fn test_versioning_multiple_updates_same_key() {
    let test_dir = "/tmp/test_versioning_updates";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"versioned_key", b"v1").unwrap();
    db.put(b"versioned_key", b"v2").unwrap();
    db.put(b"versioned_key", b"v3").unwrap();
    db.put(b"versioned_key", b"v4").unwrap();
    db.put(b"versioned_key", b"v5").unwrap();

    assert_eq!(db.get(b"versioned_key").unwrap(), Some(b"v5".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_versioning_delete_and_recreate() {
    let test_dir = "/tmp/test_versioning_delete_recreate";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key", b"value1").unwrap();
    db.del(b"key").unwrap();
    assert_eq!(db.get(b"key").unwrap(), None);

    db.put(b"key", b"value2").unwrap();
    assert_eq!(db.get(b"key").unwrap(), Some(b"value2".to_vec()));

    db.del(b"key").unwrap();
    assert_eq!(db.get(b"key").unwrap(), None);

    db.put(b"key", b"value3").unwrap();
    assert_eq!(db.get(b"key").unwrap(), Some(b"value3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_versioning_interleaved_keys() {
    let test_dir = "/tmp/test_versioning_interleaved";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"v1_1").unwrap();
    db.put(b"key2", b"v2_1").unwrap();
    db.put(b"key1", b"v1_2").unwrap();
    db.put(b"key3", b"v3_1").unwrap();
    db.put(b"key2", b"v2_2").unwrap();
    db.put(b"key1", b"v1_3").unwrap();

    assert_eq!(db.get(b"key1").unwrap(), Some(b"v1_3".to_vec()));
    assert_eq!(db.get(b"key2").unwrap(), Some(b"v2_2".to_vec()));
    assert_eq!(db.get(b"key3").unwrap(), Some(b"v3_1".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_versioning_with_transactions() {
    let test_dir = "/tmp/test_versioning_with_txn";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key", b"v1").unwrap();
    let txn1 = db.begin();

    db.put(b"key", b"v2").unwrap();
    let txn2 = db.begin();

    db.put(b"key", b"v3").unwrap();
    let txn3 = db.begin();

    assert_eq!(txn1.get(b"key").unwrap(), Some(b"v1".to_vec()));
    assert_eq!(txn2.get(b"key").unwrap(), Some(b"v2".to_vec()));
    assert_eq!(txn3.get(b"key").unwrap(), Some(b"v3".to_vec()));
    assert_eq!(db.get(b"key").unwrap(), Some(b"v3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_recovery_transaction_after_restart() {
    let test_dir = "/tmp/test_recovery_txn";
    let _ = fs::remove_dir_all(test_dir);

    {
        let db = Db::open(test_dir).unwrap();
        let mut txn = db.begin();
        txn.put(b"key1", b"value1");
        txn.put(b"key2", b"value2");
        txn.commit().unwrap();
    }

    thread::sleep(std::time::Duration::from_millis(100));

    {
        let db = Db::open(test_dir).unwrap();
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
        assert_eq!(db.get(b"key2").unwrap(), Some(b"value2".to_vec()));
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_recovery_multiple_versions() {
    let test_dir = "/tmp/test_recovery_versions";
    let _ = fs::remove_dir_all(test_dir);

    {
        let db = Db::open(test_dir).unwrap();
        db.put(b"key", b"v1").unwrap();
        db.put(b"key", b"v2").unwrap();
        db.put(b"key", b"v3").unwrap();
    }

    thread::sleep(std::time::Duration::from_millis(100));

    {
        let db = Db::open(test_dir).unwrap();
        assert_eq!(db.get(b"key").unwrap(), Some(b"v3".to_vec()));
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_recovery_after_deletes() {
    let test_dir = "/tmp/test_recovery_deletes";
    let _ = fs::remove_dir_all(test_dir);

    {
        let db = Db::open(test_dir).unwrap();
        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();
        db.del(b"key1").unwrap();
    }

    thread::sleep(std::time::Duration::from_millis(100));

    {
        let db = Db::open(test_dir).unwrap();
        assert_eq!(db.get(b"key1").unwrap(), None);
        assert_eq!(db.get(b"key2").unwrap(), Some(b"value2".to_vec()));
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_recovery_large_dataset() {
    let test_dir = "/tmp/test_recovery_large";
    let _ = fs::remove_dir_all(test_dir);

    {
        let db = Db::open(test_dir).unwrap();
        for i in 0..1000 {
            let key = format!("key_{:05}", i);
            let value = format!("value_{}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
    }

    thread::sleep(std::time::Duration::from_millis(500));

    {
        let db = Db::open(test_dir).unwrap();
        for i in 0..1000 {
            let key = format!("key_{:05}", i);
            let expected = format!("value_{}", i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected.as_bytes().to_vec())
            );
        }
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_recovery_transaction_sequence_numbers() {
    let test_dir = "/tmp/test_recovery_txn_seq";
    let _ = fs::remove_dir_all(test_dir);

    {
        let db = Db::open(test_dir).unwrap();

        db.put(b"before", b"v1").unwrap();

        let mut txn = db.begin();
        txn.put(b"txn_key", b"txn_value");
        txn.commit().unwrap();

        db.put(b"after", b"v2").unwrap();
    }

    thread::sleep(std::time::Duration::from_millis(100));

    {
        let db = Db::open(test_dir).unwrap();

        let txn = db.begin();

        db.put(b"new_key", b"new_value").unwrap();

        assert_eq!(txn.get(b"new_key").unwrap(), None);
        assert_eq!(txn.get(b"before").unwrap(), Some(b"v1".to_vec()));
        assert_eq!(txn.get(b"txn_key").unwrap(), Some(b"txn_value".to_vec()));
        assert_eq!(txn.get(b"after").unwrap(), Some(b"v2".to_vec()));
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_memtable_flush_preserves_versions() {
    let test_dir = "/tmp/test_flush_versions";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    for i in 0..10000 {
        let key = format!("key_{:05}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    for i in 0..100 {
        let key = format!("key_{:05}", i);
        let value = format!("updated_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    thread::sleep(std::time::Duration::from_millis(500));

    for i in 0..100 {
        let key = format!("key_{:05}", i);
        let expected = format!("updated_{}", i);
        assert_eq!(
            db.get(key.as_bytes()).unwrap(),
            Some(expected.as_bytes().to_vec())
        );
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_across_flush() {
    let test_dir = "/tmp/test_txn_across_flush";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"value1").unwrap();
    let txn = db.begin();

    for i in 0..10000 {
        let key = format!("flush_key_{:05}", i);
        db.put(key.as_bytes(), b"data").unwrap();
    }

    thread::sleep(std::time::Duration::from_millis(500));

    assert_eq!(txn.get(b"key1").unwrap(), Some(b"value1".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_concurrent_reads() {
    let test_dir = "/tmp/test_concurrent_reads";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());

    for i in 0..100 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    let mut handles = vec![];

    for thread_id in 0..10 {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                for i in 0..100 {
                    let key = format!("key{}", i);
                    let expected = format!("value{}", i);
                    let result = db.get(key.as_bytes()).unwrap();
                    assert_eq!(
                        result,
                        Some(expected.as_bytes().to_vec()),
                        "Thread {} failed on key {}",
                        thread_id,
                        i
                    );
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_concurrent_writes_different_keys() {
    let test_dir = "/tmp/test_concurrent_writes_diff";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());
    let mut handles = vec![];

    for thread_id in 0..10 {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let key = format!("thread{}_key{}", thread_id, i);
                let value = format!("value{}", i);
                db.put(key.as_bytes(), value.as_bytes()).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for thread_id in 0..10 {
        for i in 0..100 {
            let key = format!("thread{}_key{}", thread_id, i);
            let expected = format!("value{}", i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected.as_bytes().to_vec())
            );
        }
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_concurrent_writes_same_keys() {
    let test_dir = "/tmp/test_concurrent_writes_same";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());
    let mut handles = vec![];

    for thread_id in 0..10 {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let key = format!("shared_key{}", i % 10);
                let value = format!("thread{}_value{}", thread_id, i);
                db.put(key.as_bytes(), value.as_bytes()).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for i in 0..10 {
        let key = format!("shared_key{}", i);
        let result = db.get(key.as_bytes()).unwrap();
        assert!(result.is_some());
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_concurrent_mixed_operations() {
    let test_dir = "/tmp/test_concurrent_mixed";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());

    for i in 0..50 {
        let key = format!("key{}", i);
        db.put(key.as_bytes(), b"initial").unwrap();
    }

    let mut handles = vec![];

    for thread_id in 0..5 {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let key = format!("key{}", i % 50);

                match i % 3 {
                    0 => {
                        let value = format!("t{}_{}", thread_id, i);
                        db.put(key.as_bytes(), value.as_bytes()).unwrap();
                    }
                    1 => {
                        db.del(key.as_bytes()).unwrap();
                    }
                    _ => {
                        let _ = db.get(key.as_bytes());
                    }
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Database should be in consistent state (no panics)
    for i in 0..50 {
        let key = format!("key{}", i);
        let _ = db.get(key.as_bytes()).unwrap();
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_very_long_keys() {
    let test_dir = "/tmp/test_long_keys";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let long_key = vec![b'k'; 1000];
    let value = b"value_for_long_key";

    db.put(&long_key, value).unwrap();
    assert_eq!(db.get(&long_key).unwrap(), Some(value.to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_many_small_operations() {
    let test_dir = "/tmp/test_many_small_ops";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    for i in 0..10000 {
        let key = format!("{}", i);
        let value = format!("{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    for i in 0..10000 {
        let key = format!("{}", i);
        let expected = format!("{}", i);
        assert_eq!(
            db.get(key.as_bytes()).unwrap(),
            Some(expected.as_bytes().to_vec())
        );
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_alternating_put_delete() {
    let test_dir = "/tmp/test_alternating";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    for i in 0..100 {
        db.put(b"key", format!("value{}", i).as_bytes()).unwrap();
        if i % 2 == 0 {
            db.del(b"key").unwrap();
        }
    }

    assert_eq!(db.get(b"key").unwrap(), Some(b"value99".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_scan_with_versions() {
    let test_dir = "/tmp/test_scan_versions";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"v1_1").unwrap();
    db.put(b"key2", b"v2_1").unwrap();
    db.put(b"key1", b"v1_2").unwrap();
    db.put(b"key3", b"v3_1").unwrap();
    db.put(b"key2", b"v2_2").unwrap();

    let mut results = vec![];
    let mut iter = db.scan(None, None);
    while let Some((key, value)) = iter.next() {
        results.push((key, value));
    }

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], (b"key1".to_vec(), b"v1_2".to_vec()));
    assert_eq!(results[1], (b"key2".to_vec(), b"v2_2".to_vec()));
    assert_eq!(results[2], (b"key3".to_vec(), b"v3_1".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_get_after_many_versions() {
    let test_dir = "/tmp/test_many_versions";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    for i in 0..1000 {
        let value = format!("version_{}", i);
        db.put(b"versioned_key", value.as_bytes()).unwrap();
    }

    assert_eq!(
        db.get(b"versioned_key").unwrap(),
        Some(b"version_999".to_vec())
    );

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_with_many_versions_in_db() {
    let test_dir = "/tmp/test_txn_many_versions";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key", b"v0").unwrap();

    let txn = db.begin();

    for i in 1..100 {
        let value = format!("v{}", i);
        db.put(b"key", value.as_bytes()).unwrap();
    }

    assert_eq!(txn.get(b"key").unwrap(), Some(b"v0".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_empty_database_operations() {
    let test_dir = "/tmp/test_empty_db";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    assert_eq!(db.get(b"nonexistent").unwrap(), None);

    db.del(b"nonexistent").unwrap();

    let mut iter = db.scan(None, None);
    assert!(iter.next().is_none());

    let txn = db.begin();
    assert_eq!(txn.get(b"nonexistent").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}
