use keylite_kv::db::Db;
use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn test_transaction_read_own_writes() {
    let test_dir = "/tmp/test_txn_read_own_writes";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"key1", b"value1");

    let result = txn.get(b"key1").unwrap();
    assert_eq!(result, Some(b"value1".to_vec()));

    txn.commit().unwrap();

    assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
}

#[test]
fn test_transaction_isolation_multiple_readers() {
    let test_dir = "/tmp/test_txn_multi_readers";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());
    db.put(b"key1", b"v0").unwrap();

    let txn1 = db.begin();
    let txn2 = db.begin();
    let txn3 = db.begin();
    let txn4 = db.begin();
    let txn5 = db.begin();

    db.put(b"key1", b"v1").unwrap();
    db.put(b"key1", b"v2").unwrap();
    db.put(b"key1", b"v3").unwrap();

    assert_eq!(txn1.get(b"key1").unwrap(), Some(b"v0".to_vec()));
    assert_eq!(txn2.get(b"key1").unwrap(), Some(b"v0".to_vec()));
    assert_eq!(txn3.get(b"key1").unwrap(), Some(b"v0".to_vec()));
    assert_eq!(txn4.get(b"key1").unwrap(), Some(b"v0".to_vec()));
    assert_eq!(txn5.get(b"key1").unwrap(), Some(b"v0".to_vec()));

    assert_eq!(db.get(b"key1").unwrap(), Some(b"v3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_isolation_staggered_snapshots() {
    let test_dir = "/tmp/test_txn_staggered";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"v1").unwrap();
    let txn1 = db.begin();

    db.put(b"key1", b"v2").unwrap();
    let txn2 = db.begin();

    db.put(b"key1", b"v3").unwrap();
    let txn3 = db.begin();

    db.put(b"key1", b"v4").unwrap();

    assert_eq!(txn1.get(b"key1").unwrap(), Some(b"v1".to_vec()));
    assert_eq!(txn2.get(b"key1").unwrap(), Some(b"v2".to_vec()));
    assert_eq!(txn3.get(b"key1").unwrap(), Some(b"v3".to_vec()));
    assert_eq!(db.get(b"key1").unwrap(), Some(b"v4".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_sees_deletes_at_snapshot() {
    let test_dir = "/tmp/test_txn_sees_deletes";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"value1").unwrap();
    db.put(b"key2", b"value2").unwrap();

    let txn = db.begin();

    db.del(b"key1").unwrap();

    assert_eq!(txn.get(b"key1").unwrap(), Some(b"value1".to_vec()));

    assert_eq!(db.get(b"key1").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_multiple_deletes_in_transaction() {
    let test_dir = "/tmp/test_txn_multi_deletes";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"value1").unwrap();
    db.put(b"key2", b"value2").unwrap();
    db.put(b"key3", b"value3").unwrap();

    let mut txn = db.begin();
    txn.del(b"key1");
    txn.del(b"key2");
    txn.del(b"key3");
    txn.commit().unwrap();

    assert_eq!(db.get(b"key1").unwrap(), None);
    assert_eq!(db.get(b"key2").unwrap(), None);
    assert_eq!(db.get(b"key3").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_commit_atomicity() {
    let test_dir = "/tmp/test_txn_atomicity";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    for i in 0..100 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        txn.put(key.as_bytes(), value.as_bytes());
    }
    txn.commit().unwrap();

    for i in 0..100 {
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
fn test_transaction_abort_discards_all_writes() {
    let test_dir = "/tmp/test_txn_abort_all";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    for i in 0..50 {
        let key = format!("key{}", i);
        let value = format!("value{}", i);
        txn.put(key.as_bytes(), value.as_bytes());
    }
    txn.abort();

    for i in 0..50 {
        let key = format!("key{}", i);
        assert_eq!(db.get(key.as_bytes()).unwrap(), None);
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_overwrite_in_same_transaction() {
    let test_dir = "/tmp/test_txn_overwrite_same";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"key1", b"v1");
    txn.put(b"key1", b"v2");
    txn.put(b"key1", b"v3");
    txn.del(b"key1");
    txn.put(b"key1", b"v4");
    txn.commit().unwrap();

    assert_eq!(db.get(b"key1").unwrap(), Some(b"v4".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_empty_commit() {
    let test_dir = "/tmp/test_txn_empty";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let txn = db.begin();
    txn.commit().unwrap();

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_abort_after_commit_prepare() {
    let test_dir = "/tmp/test_txn_abort_after_writes";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"key1", b"value1");
    txn.put(b"key2", b"value2");
    txn.abort();

    assert_eq!(db.get(b"key1").unwrap(), None);
    assert_eq!(db.get(b"key2").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_update_existing_keys() {
    let test_dir = "/tmp/test_txn_update_existing";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"old1").unwrap();
    db.put(b"key2", b"old2").unwrap();
    db.put(b"key3", b"old3").unwrap();

    let mut txn = db.begin();
    txn.put(b"key1", b"new1");
    txn.put(b"key2", b"new2");
    txn.commit().unwrap();

    assert_eq!(db.get(b"key1").unwrap(), Some(b"new1".to_vec()));
    assert_eq!(db.get(b"key2").unwrap(), Some(b"new2".to_vec()));
    assert_eq!(db.get(b"key3").unwrap(), Some(b"old3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_delete_existing_keys() {
    let test_dir = "/tmp/test_txn_delete_existing";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"value1").unwrap();
    db.put(b"key2", b"value2").unwrap();

    let txn1 = db.begin();

    let mut txn2 = db.begin();
    txn2.del(b"key1");
    txn2.commit().unwrap();

    assert_eq!(txn1.get(b"key1").unwrap(), Some(b"value1".to_vec()));

    assert_eq!(db.get(b"key1").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_read_after_external_update() {
    let test_dir = "/tmp/test_txn_read_after_update";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"counter", b"0").unwrap();

    let txn1 = db.begin();

    db.put(b"counter", b"1").unwrap();
    db.put(b"counter", b"2").unwrap();
    db.put(b"counter", b"3").unwrap();

    // Transaction should still see original value
    assert_eq!(txn1.get(b"counter").unwrap(), Some(b"0".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_concurrent_transactions_no_conflict() {
    let test_dir = "/tmp/test_concurrent_txn_no_conflict";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());
    let barrier = Arc::new(Barrier::new(5));

    let mut handles = vec![];

    for i in 0..5 {
        let db = Arc::clone(&db);
        let barrier = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            barrier.wait();

            let mut txn = db.begin();
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            txn.put(key.as_bytes(), value.as_bytes());
            txn.commit().unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for i in 0..5 {
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
fn test_concurrent_transactions_same_key_last_writer_wins() {
    let test_dir = "/tmp/test_concurrent_txn_same_key";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());
    let barrier = Arc::new(Barrier::new(10));

    let mut handles = vec![];

    for i in 0..10 {
        let db = Arc::clone(&db);
        let barrier = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            barrier.wait();

            let mut txn = db.begin();
            let value = format!("value{}", i);
            txn.put(b"shared_key", value.as_bytes());
            txn.commit().unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let result = db.get(b"shared_key").unwrap();
    assert!(result.is_some());

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_read_consistency_under_concurrent_writes() {
    let test_dir = "/tmp/test_txn_consistency";
    let _ = fs::remove_dir_all(test_dir);

    let db = Arc::new(Db::open(test_dir).unwrap());

    db.put(b"a", b"1").unwrap();
    db.put(b"b", b"2").unwrap();
    db.put(b"c", b"3").unwrap();

    let txn = db.begin();

    let db_clone = Arc::clone(&db);
    let writer = thread::spawn(move || {
        for _ in 0..100 {
            db_clone.put(b"a", b"100").unwrap();
            db_clone.put(b"b", b"200").unwrap();
            db_clone.put(b"c", b"300").unwrap();
        }
    });

    for _ in 0..100 {
        assert_eq!(txn.get(b"a").unwrap(), Some(b"1".to_vec()));
        assert_eq!(txn.get(b"b").unwrap(), Some(b"2".to_vec()));
        assert_eq!(txn.get(b"c").unwrap(), Some(b"3".to_vec()));
    }

    writer.join().unwrap();

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_empty_key() {
    let test_dir = "/tmp/test_txn_empty_key";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"", b"empty_key_value");
    txn.commit().unwrap();

    assert_eq!(db.get(b"").unwrap(), Some(b"empty_key_value".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_empty_value() {
    let test_dir = "/tmp/test_txn_empty_value";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"key_with_empty_value", b"");
    txn.commit().unwrap();

    let result = db.get(b"key_with_empty_value").unwrap();
    assert_eq!(result, None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_large_number_of_operations() {
    let test_dir = "/tmp/test_txn_large_ops";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    for i in 0..1000 {
        let key = format!("key_{:05}", i);
        let value = format!("value_{}", i);
        txn.put(key.as_bytes(), value.as_bytes());
    }
    txn.commit().unwrap();

    for i in 0..1000 {
        let key = format!("key_{:05}", i);
        let expected = format!("value_{}", i);
        assert_eq!(
            db.get(key.as_bytes()).unwrap(),
            Some(expected.as_bytes().to_vec())
        );
    }

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_large_values() {
    let test_dir = "/tmp/test_txn_large_values";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let large_value = vec![b'X'; 1024 * 1024]; // 1MB

    let mut txn = db.begin();
    txn.put(b"large_key", &large_value);
    txn.commit().unwrap();

    assert_eq!(db.get(b"large_key").unwrap(), Some(large_value));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_binary_data() {
    let test_dir = "/tmp/test_txn_binary";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let binary_key = vec![0u8, 1, 2, 255, 254, 253];
    let binary_value = vec![255u8, 128, 64, 32, 16, 8, 4, 2, 1, 0];

    let mut txn = db.begin();
    txn.put(&binary_key, &binary_value);
    txn.commit().unwrap();

    assert_eq!(db.get(&binary_key).unwrap(), Some(binary_value));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_special_characters() {
    let test_dir = "/tmp/test_txn_special_chars";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put("key\0with\nnull".as_bytes(), b"value1");
    txn.put("key\t\r\n".as_bytes(), b"value2");
    txn.put("键".as_bytes(), "值".as_bytes());
    txn.commit().unwrap();

    assert_eq!(
        db.get("key\0with\nnull".as_bytes()).unwrap(),
        Some(b"value1".to_vec())
    );
    assert_eq!(
        db.get("key\t\r\n".as_bytes()).unwrap(),
        Some(b"value2".to_vec())
    );
    assert_eq!(
        db.get("键".as_bytes()).unwrap(),
        Some("值".as_bytes().to_vec())
    );

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_delete_nonexistent_key() {
    let test_dir = "/tmp/test_txn_delete_nonexistent";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.del(b"nonexistent_key");
    txn.commit().unwrap();

    assert_eq!(db.get(b"nonexistent_key").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_mixed_operations_order() {
    let test_dir = "/tmp/test_txn_mixed_order";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key1", b"old").unwrap();

    let mut txn = db.begin();
    txn.put(b"key1", b"new1");
    txn.del(b"key1");
    txn.put(b"key1", b"new2");
    txn.put(b"key2", b"value2");
    txn.del(b"key2");
    txn.put(b"key3", b"value3");
    txn.commit().unwrap();

    assert_eq!(db.get(b"key1").unwrap(), Some(b"new2".to_vec()));
    assert_eq!(db.get(b"key2").unwrap(), None);
    assert_eq!(db.get(b"key3").unwrap(), Some(b"value3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_sequence_isolation() {
    let test_dir = "/tmp/test_txn_seq_isolation";
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

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_commit_gets_new_sequence() {
    let test_dir = "/tmp/test_txn_new_seq";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"key", b"v1").unwrap();

    let txn1 = db.begin();
    let txn2 = db.begin();

    let mut txn1 = txn1;
    txn1.put(b"result", b"txn1");
    txn1.commit().unwrap();

    let mut txn2 = txn2;
    txn2.put(b"result", b"txn2");
    txn2.commit().unwrap();

    assert_eq!(db.get(b"result").unwrap(), Some(b"txn2".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}
