use keylite_kv::core::Db;
use std::fs;

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
