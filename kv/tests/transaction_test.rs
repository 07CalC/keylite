use keylite_kv::core::Db;
use std::{fs, string};

#[test]
fn test_basic_transaction_commit() {
    let test_dir = "/tmp/test_txn_basic";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"key1", b"value1");
    txn.put(b"key2", b"value2");
    txn.commit().unwrap();

    assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
    assert_eq!(db.get(b"key2").unwrap(), Some(b"value2".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_abort() {
    let test_dir = "/tmp/test_txn_abort";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"key3", b"value3");
    txn.abort();

    assert_eq!(db.get(b"key3").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_snapshot_isolation() {
    let test_dir = "/tmp/test_txn_snapshot";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();
    db.put(b"snap1", b"v1").unwrap();

    let txn1 = db.begin();
    let val_before = txn1.get(b"snap1").unwrap();

    db.put(b"snap1", b"v2").unwrap();

    let val_after = txn1.get(b"snap1").unwrap();

    assert_eq!(val_before, Some(b"v1".to_vec()));
    assert_eq!(val_after, Some(b"v1".to_vec()));
    assert_eq!(db.get(b"snap1").unwrap(), Some(b"v2".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_delete() {
    let test_dir = "/tmp/test_txn_delete";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();
    db.put(b"del_key", b"del_val").unwrap();

    let mut txn = db.begin();
    txn.del(b"del_key");
    txn.commit().unwrap();

    assert_eq!(db.get(b"del_key").unwrap(), None);

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_multiple_ops_same_key() {
    let test_dir = "/tmp/test_txn_multi";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    let mut txn = db.begin();
    txn.put(b"multi", b"v1");
    txn.put(b"multi", b"v2");
    txn.put(b"multi", b"v3");
    txn.commit().unwrap();

    assert_eq!(db.get(b"multi").unwrap(), Some(b"v3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_scan_all() {
    let test_dir = "/tmp/test_txn_scan_all";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"a1", b"db_a1").unwrap();
    db.put(b"b2", b"db_b2").unwrap();
    db.put(b"c3", b"db_c3").unwrap();

    let mut txn = db.begin();
    txn.put(b"a2", b"txn_a2");
    txn.put(b"b1", b"txn_b1");
    txn.put(b"d1", b"txn_d1");

    let results: Vec<_> = txn.scan(None, None).collect();
    for res in results.iter() {
        println!("{:?}", String::from_utf8(res.1.clone()));
    }
    println!("{:?}", results);

    assert_eq!(results.len(), 6);
    assert_eq!(results[0], (b"a1".to_vec(), b"db_a1".to_vec()));
    assert_eq!(results[1], (b"a2".to_vec(), b"txn_a2".to_vec()));
    assert_eq!(results[2], (b"b1".to_vec(), b"txn_b1".to_vec()));
    assert_eq!(results[3], (b"b2".to_vec(), b"db_b2".to_vec()));
    assert_eq!(results[4], (b"c3".to_vec(), b"db_c3".to_vec()));
    assert_eq!(results[5], (b"d1".to_vec(), b"txn_d1".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_scan_with_range() {
    let test_dir = "/tmp/test_txn_scan_range";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"a1", b"db_a1").unwrap();
    db.put(b"b2", b"db_b2").unwrap();
    db.put(b"c3", b"db_c3").unwrap();
    db.put(b"d4", b"db_d4").unwrap();

    let mut txn = db.begin();
    txn.put(b"b1", b"txn_b1");
    txn.put(b"c1", b"txn_c1");

    let results: Vec<_> = txn.scan(Some(b"b"), Some(b"d")).collect();
    println!("{:?}", results);

    assert_eq!(results.len(), 4);
    assert_eq!(results[0], (b"b1".to_vec(), b"txn_b1".to_vec()));
    assert_eq!(results[1], (b"b2".to_vec(), b"db_b2".to_vec()));
    assert_eq!(results[2], (b"c1".to_vec(), b"txn_c1".to_vec()));
    assert_eq!(results[3], (b"c3".to_vec(), b"db_c3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_scan_with_overwrites() {
    let test_dir = "/tmp/test_txn_scan_overwrite";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    // Put some data in DB
    db.put(b"key1", b"db_value1").unwrap();
    db.put(b"key2", b"db_value2").unwrap();
    db.put(b"key3", b"db_value3").unwrap();

    // Create transaction that overwrites some keys
    let mut txn = db.begin();
    txn.put(b"key2", b"txn_value2");

    // Scan all - should see transaction value for key2
    let results: Vec<_> = txn.scan(None, None).collect();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0], (b"key1".to_vec(), b"db_value1".to_vec()));
    assert_eq!(results[1], (b"key2".to_vec(), b"txn_value2".to_vec()));
    assert_eq!(results[2], (b"key3".to_vec(), b"db_value3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_scan_with_deletes() {
    let test_dir = "/tmp/test_txn_scan_delete";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    // Put some data in DB
    db.put(b"key1", b"db_value1").unwrap();
    db.put(b"key2", b"db_value2").unwrap();
    db.put(b"key3", b"db_value3").unwrap();

    // Create transaction that deletes a key
    let mut txn = db.begin();
    txn.del(b"key2");

    // Scan all - should not see key2
    let results: Vec<_> = txn.scan(None, None).collect();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (b"key1".to_vec(), b"db_value1".to_vec()));
    assert_eq!(results[1], (b"key3".to_vec(), b"db_value3".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_scan_start_only() {
    let test_dir = "/tmp/test_txn_scan_start";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"a", b"val_a").unwrap();
    db.put(b"b", b"val_b").unwrap();
    db.put(b"c", b"val_c").unwrap();
    db.put(b"d", b"val_d").unwrap();

    let txn = db.begin();

    // Scan from 'c' onwards
    let results: Vec<_> = txn.scan(Some(b"c"), None).collect();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (b"c".to_vec(), b"val_c".to_vec()));
    assert_eq!(results[1], (b"d".to_vec(), b"val_d".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_transaction_scan_end_only() {
    let test_dir = "/tmp/test_txn_scan_end";
    let _ = fs::remove_dir_all(test_dir);

    let db = Db::open(test_dir).unwrap();

    db.put(b"a", b"val_a").unwrap();
    db.put(b"b", b"val_b").unwrap();
    db.put(b"c", b"val_c").unwrap();
    db.put(b"d", b"val_d").unwrap();

    let txn = db.begin();

    // Scan up to 'c' (exclusive)
    let results: Vec<_> = txn.scan(None, Some(b"c")).collect();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], (b"a".to_vec(), b"val_a".to_vec()));
    assert_eq!(results[1], (b"b".to_vec(), b"val_b".to_vec()));

    let _ = fs::remove_dir_all(test_dir);
}
