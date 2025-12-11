use keylite_kv::db::Db;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn create_test_db(test_name: &str) -> Db {
    let path = format!("test_data/{}", test_name);
    let _ = std::fs::remove_dir_all(&path);
    Db::open(&path).unwrap()
}

fn cleanup_test_db(test_name: &str) {
    let path = format!("test_data/{}", test_name);
    let _ = std::fs::remove_dir_all(&path);
}

#[test]
fn test_basic_put_get() {
    let db = create_test_db("basic_put_get");

    db.put(b"key1", b"value1").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));

    db.put(b"key2", b"value2").unwrap();
    db.put(b"key3", b"value3").unwrap();

    assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));
    assert_eq!(db.get(b"key2").unwrap(), Some(b"value2".to_vec()));
    assert_eq!(db.get(b"key3").unwrap(), Some(b"value3".to_vec()));

    assert_eq!(db.get(b"nonexistent").unwrap(), None);

    drop(db);
    cleanup_test_db("basic_put_get");
}

#[test]
fn test_overwrite() {
    let db = create_test_db("overwrite");

    db.put(b"key1", b"value1").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));

    db.put(b"key1", b"value2").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), Some(b"value2".to_vec()));

    db.put(b"key1", b"value3").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), Some(b"value3".to_vec()));

    drop(db);
    cleanup_test_db("overwrite");
}

#[test]
fn test_delete() {
    let db = create_test_db("delete");

    db.put(b"key1", b"value1").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));

    db.del(b"key1").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), None);

    db.del(b"nonexistent").unwrap();

    db.put(b"key1", b"new_value").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), Some(b"new_value".to_vec()));

    drop(db);
    cleanup_test_db("delete");
}

#[test]
fn test_empty_key_value() {
    let db = create_test_db("empty_key_value");

    db.put(b"key1", b"").unwrap();
    assert_eq!(db.get(b"key1").unwrap(), None);

    db.put(b"", b"value").unwrap();
    assert_eq!(db.get(b"").unwrap(), Some(b"value".to_vec()));

    drop(db);
    cleanup_test_db("empty_key_value");
}

#[test]
fn test_large_values() {
    let db = create_test_db("large_values");

    let large_value = vec![b'x'; 1024 * 1024];
    db.put(b"large_key", &large_value).unwrap();
    assert_eq!(db.get(b"large_key").unwrap(), Some(large_value.clone()));

    let very_large_value = vec![b'y'; 10 * 1024 * 1024];
    db.put(b"very_large_key", &very_large_value).unwrap();
    assert_eq!(db.get(b"very_large_key").unwrap(), Some(very_large_value));

    drop(db);
    cleanup_test_db("large_values");
}

#[test]
fn test_many_keys() {
    let db = create_test_db("many_keys");

    let num_keys = 1000;

    for i in 0..num_keys {
        let key = format!("key_{:05}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    for i in 0..num_keys {
        let key = format!("key_{:05}", i);
        let expected_value = format!("value_{}", i);
        assert_eq!(
            db.get(key.as_bytes()).unwrap(),
            Some(expected_value.into_bytes())
        );
    }

    drop(db);
    cleanup_test_db("many_keys");
}

#[test]
fn test_persistence() {
    let test_name = "persistence";
    let path = format!("test_data/{}", test_name);
    let _ = std::fs::remove_dir_all(&path);

    {
        let db = Db::open(&path).unwrap();
        for i in 0..100 {
            let key = format!("persistent_key_{:05}", i);
            let value = format!("persistent_value_{}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
    }

    thread::sleep(std::time::Duration::from_secs(2));

    {
        let db = Db::open(&path).unwrap();
        for i in 0..100 {
            let key = format!("persistent_key_{:05}", i);
            let expected_value = format!("persistent_value_{}", i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected_value.into_bytes()),
                "Mismatch for key: {}",
                key
            );
        }
    }

    cleanup_test_db(test_name);
}

#[test]
fn test_persistence_with_updates() {
    let test_name = "persistence_updates";
    let path = format!("test_data/{}", test_name);
    let _ = std::fs::remove_dir_all(&path);

    {
        let db = Db::open(&path).unwrap();
        for i in 0..5000 {
            let key = format!("key_{:05}", i);
            let value = format!("value_{}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
    }

    thread::sleep(std::time::Duration::from_secs(2));

    {
        let db = Db::open(&path).unwrap();
        for i in 0..2500 {
            let key = format!("key_{:05}", i);
            let value = format!("updated_value_{}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
        for i in 5000..7500 {
            let key = format!("key_{:05}", i);
            let value = format!("value_{}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
    }

    thread::sleep(std::time::Duration::from_secs(2));

    {
        let db = Db::open(&path).unwrap();
        for i in 0..2500 {
            let key = format!("key_{:05}", i);
            let expected_value = format!("updated_value_{}", i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected_value.into_bytes())
            );
        }
        for i in 2500..5000 {
            let key = format!("key_{:05}", i);
            let expected_value = format!("value_{}", i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected_value.into_bytes())
            );
        }
        for i in 5000..7500 {
            let key = format!("key_{:05}", i);
            let expected_value = format!("value_{}", i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected_value.into_bytes())
            );
        }
    }

    cleanup_test_db(test_name);
}

#[test]
fn test_persistence_with_deletes() {
    let test_name = "persistence_deletes";
    let path = format!("test_data/{}", test_name);
    let _ = std::fs::remove_dir_all(&path);

    {
        let db = Db::open(&path).unwrap();
        for i in 0..10000 {
            let key = format!("key_{:05}", i);
            let value = format!("value_{}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
        for i in (0..10000).step_by(2) {
            let key = format!("key_{:05}", i);
            db.del(key.as_bytes()).unwrap();
        }
    }

    thread::sleep(std::time::Duration::from_secs(2));

    {
        let db = Db::open(&path).unwrap();
        for i in 0..10000 {
            let key = format!("key_{:05}", i);
            if i % 2 == 0 {
                assert_eq!(db.get(key.as_bytes()).unwrap(), None);
            } else {
                let expected_value = format!("value_{}", i);
                assert_eq!(
                    db.get(key.as_bytes()).unwrap(),
                    Some(expected_value.into_bytes())
                );
            }
        }
    }

    cleanup_test_db(test_name);
}

#[test]
fn test_concurrent_writes() {
    let db = Arc::new(create_test_db("concurrent_writes"));
    let num_threads = 10;
    let writes_per_thread = 100;

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for i in 0..writes_per_thread {
                let key = format!("thread_{}_key_{}", thread_id, i);
                let value = format!("thread_{}_value_{}", thread_id, i);
                db.put(key.as_bytes(), value.as_bytes()).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for thread_id in 0..num_threads {
        for i in 0..writes_per_thread {
            let key = format!("thread_{}_key_{}", thread_id, i);
            let expected_value = format!("thread_{}_value_{}", thread_id, i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected_value.into_bytes())
            );
        }
    }

    cleanup_test_db("concurrent_writes");
}

#[test]
fn test_concurrent_reads_writes() {
    let db = Arc::new(create_test_db("concurrent_reads_writes"));

    for i in 0..100 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    let num_readers = 5;
    let num_writers = 5;
    let operations = 100;

    let mut handles = vec![];

    for reader_id in 0..num_readers {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(reader_id as u64);
            for _ in 0..operations {
                let key = format!("key_{}", rng.random_range(0..100));
                let _ = db.get(key.as_bytes());
            }
        }));
    }

    for writer_id in 0..num_writers {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for i in 0..operations {
                let key = format!("writer_{}_key_{}", writer_id, i);
                let value = format!("writer_{}_value_{}", writer_id, i);
                db.put(key.as_bytes(), value.as_bytes()).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    for i in 0..100 {
        let key = format!("key_{}", i);
        let expected_value = format!("value_{}", i);
        assert_eq!(
            db.get(key.as_bytes()).unwrap(),
            Some(expected_value.into_bytes())
        );
    }

    for writer_id in 0..num_writers {
        for i in 0..operations {
            let key = format!("writer_{}_key_{}", writer_id, i);
            let expected_value = format!("writer_{}_value_{}", writer_id, i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected_value.into_bytes())
            );
        }
    }

    cleanup_test_db("concurrent_reads_writes");
}

#[test]
fn test_concurrent_same_key() {
    let db = Arc::new(create_test_db("concurrent_same_key"));
    let num_threads = 10;
    let iterations = 100;

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let db = Arc::clone(&db);
        handles.push(thread::spawn(move || {
            for i in 0..iterations {
                let value = format!("thread_{}_iter_{}", thread_id, i);
                db.put(b"shared_key", value.as_bytes()).unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Key should have some value (last write wins)
    let result = db.get(b"shared_key").unwrap();
    assert!(result.is_some());

    cleanup_test_db("concurrent_same_key");
}

#[test]
fn test_sequential_consistency() {
    let db = create_test_db("sequential_consistency");

    let mut expected_state: HashMap<String, Option<Vec<u8>>> = HashMap::new();
    let mut rng = StdRng::seed_from_u64(12345);

    for i in 0..500 {
        let key = format!("key_{}", rng.random_range(0..50));

        let op = rng.random_range(0..3);
        match op {
            0 => {
                let value = format!("value_{}", i);
                db.put(key.as_bytes(), value.as_bytes()).unwrap();
                expected_state.insert(key.clone(), Some(value.into_bytes()));
            }
            1 => {
                db.del(key.as_bytes()).unwrap();
                expected_state.insert(key.clone(), None);
            }
            2 => {
                let actual = db.get(key.as_bytes()).unwrap();
                let expected = expected_state.get(&key).cloned().unwrap_or(None);
                assert_eq!(actual, expected, "Mismatch for key: {}", key);
            }
            _ => unreachable!(),
        }
    }

    for (key, expected_value) in expected_state.iter() {
        let actual = db.get(key.as_bytes()).unwrap();
        assert_eq!(&actual, expected_value, "Final mismatch for key: {}", key);
    }

    drop(db);
    cleanup_test_db("sequential_consistency");
}

#[test]
fn test_force_flush_and_compaction() {
    let db = create_test_db("flush_compaction");

    // big number to get multiple flushed
    let num_entries = 100_000;

    for i in 0..num_entries {
        let key = format!("key_{:08}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    thread::sleep(std::time::Duration::from_secs(2));

    for i in 0..num_entries {
        let key = format!("key_{:08}", i);
        let expected_value = format!("value_{}", i);
        let actual = db.get(key.as_bytes()).unwrap();
        assert_eq!(
            actual,
            Some(expected_value.into_bytes()),
            "Mismatch for key: {}",
            key
        );
    }

    drop(db);
    cleanup_test_db("flush_compaction");
}

#[test]
fn test_recovery_after_flush() {
    let test_name = "recovery_flush";
    let path = format!("test_data/{}", test_name);
    let _ = std::fs::remove_dir_all(&path);

    {
        let db = Db::open(&path).unwrap();
        for i in 0..50_000 {
            let key = format!("key_{:08}", i);
            let value = format!("value_{}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }

    {
        let db = Db::open(&path).unwrap();
        for i in 0..50_000 {
            let key = format!("key_{:08}", i);
            let expected_value = format!("value_{}", i);
            assert_eq!(
                db.get(key.as_bytes()).unwrap(),
                Some(expected_value.into_bytes())
            );
        }
    }

    cleanup_test_db(test_name);
}

#[test]
fn test_binary_keys_and_values() {
    let db = create_test_db("binary_data");

    let binary_key1 = vec![0u8, 1, 2, 3, 255, 254, 253];
    let binary_value1 = vec![255u8, 128, 64, 32, 16, 8, 4, 2, 1, 0];

    db.put(&binary_key1, &binary_value1).unwrap();
    assert_eq!(db.get(&binary_key1).unwrap(), Some(binary_value1.clone()));

    let key_with_nulls = b"key\0with\0nulls";
    let value_with_nulls = b"value\0with\0nulls";

    db.put(key_with_nulls, value_with_nulls).unwrap();
    assert_eq!(
        db.get(key_with_nulls).unwrap(),
        Some(value_with_nulls.to_vec())
    );

    drop(db);
    cleanup_test_db("binary_data");
}

#[test]
#[ignore]
fn test_stress_concurrent_mixed_ops() {
    let db = Arc::new(create_test_db("stress_mixed"));
    let num_threads = 8;
    let operations_per_thread = 1000;

    let expected_state: Arc<Mutex<HashMap<Vec<u8>, Option<Vec<u8>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let db = Arc::clone(&db);
        let expected = Arc::clone(&expected_state);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(thread_id as u64 * 999);

            for i in 0..operations_per_thread {
                let key_id = rng.random_range(0..100);
                let key = format!("key_{}", key_id);
                let key_bytes = key.as_bytes().to_vec();

                let op = rng.random_range(0..10);

                if op < 6 {
                    let value = format!("t{}_i{}_v{}", thread_id, i, rng.random::<u32>());
                    let value_bytes = value.as_bytes().to_vec();
                    db.put(&key_bytes, &value_bytes).unwrap();

                    let mut state = expected.lock().unwrap();
                    state.insert(key_bytes, Some(value_bytes));
                } else if op < 8 {
                    db.del(&key_bytes).unwrap();

                    let mut state = expected.lock().unwrap();
                    state.insert(key_bytes, None);
                } else {
                    let _ = db.get(&key_bytes);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    thread::sleep(std::time::Duration::from_millis(2000));

    let final_state = expected_state.lock().unwrap();
    for (key, expected_value) in final_state.iter() {
        let actual = db.get(key).unwrap();
        assert_eq!(
            &actual,
            expected_value,
            "Mismatch for key: {:?}",
            String::from_utf8_lossy(key)
        );
    }

    cleanup_test_db("stress_mixed");
}

#[test]
fn test_scan_all() {
    let db = create_test_db("scan_all");

    db.put(b"key3", b"value3").unwrap();
    db.put(b"key1", b"value1").unwrap();
    db.put(b"key5", b"value5").unwrap();
    db.put(b"key2", b"value2").unwrap();
    db.put(b"key4", b"value4").unwrap();

    let iter = db.scan(None, None);
    let results: Vec<(Vec<u8>, Vec<u8>)> = iter.collect();

    assert_eq!(results.len(), 5);
    assert_eq!(results[0], (b"key1".to_vec(), b"value1".to_vec()));
    assert_eq!(results[1], (b"key2".to_vec(), b"value2".to_vec()));
    assert_eq!(results[2], (b"key3".to_vec(), b"value3".to_vec()));
    assert_eq!(results[3], (b"key4".to_vec(), b"value4".to_vec()));
    assert_eq!(results[4], (b"key5".to_vec(), b"value5".to_vec()));

    drop(db);
    cleanup_test_db("scan_all");
}

#[test]
fn test_scan_range() {
    let db = create_test_db("scan_range");

    for i in 0..10 {
        let key = format!("key_{:02}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    let iter = db.scan(Some(b"key_03"), Some(b"key_07"));
    let results: Vec<(Vec<u8>, Vec<u8>)> = iter.collect();

    assert_eq!(results.len(), 4);
    assert_eq!(results[0], (b"key_03".to_vec(), b"value_3".to_vec()));
    assert_eq!(results[1], (b"key_04".to_vec(), b"value_4".to_vec()));
    assert_eq!(results[2], (b"key_05".to_vec(), b"value_5".to_vec()));
    assert_eq!(results[3], (b"key_06".to_vec(), b"value_6".to_vec()));

    drop(db);
    cleanup_test_db("scan_range");
}

#[test]
fn test_scan_with_start_only() {
    let db = create_test_db("scan_start");

    for i in 0..10 {
        let key = format!("key_{:02}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    let iter = db.scan(Some(b"key_05"), None);
    let results: Vec<(Vec<u8>, Vec<u8>)> = iter.collect();

    assert_eq!(results.len(), 5);
    assert_eq!(results[0], (b"key_05".to_vec(), b"value_5".to_vec()));
    assert_eq!(results[4], (b"key_09".to_vec(), b"value_9".to_vec()));

    drop(db);
    cleanup_test_db("scan_start");
}

#[test]
fn test_scan_with_end_only() {
    let db = create_test_db("scan_end");

    for i in 0..10 {
        let key = format!("key_{:02}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    let iter = db.scan(None, Some(b"key_05"));
    let results: Vec<(Vec<u8>, Vec<u8>)> = iter.collect();

    assert_eq!(results.len(), 5);
    assert_eq!(results[0], (b"key_00".to_vec(), b"value_0".to_vec()));
    assert_eq!(results[4], (b"key_04".to_vec(), b"value_4".to_vec()));

    drop(db);
    cleanup_test_db("scan_end");
}

#[test]
fn test_scan_skips_tombstones() {
    let db = create_test_db("scan_tombstones");

    for i in 0..10 {
        let key = format!("key_{:02}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    // Delete every other key
    for i in (0..10).step_by(2) {
        let key = format!("key_{:02}", i);
        db.del(key.as_bytes()).unwrap();
    }

    // Scan should skip deleted keys
    let iter = db.scan(None, None);
    let results: Vec<(Vec<u8>, Vec<u8>)> = iter.collect();

    assert_eq!(results.len(), 5);
    assert_eq!(results[0], (b"key_01".to_vec(), b"value_1".to_vec()));
    assert_eq!(results[1], (b"key_03".to_vec(), b"value_3".to_vec()));
    assert_eq!(results[2], (b"key_05".to_vec(), b"value_5".to_vec()));
    assert_eq!(results[3], (b"key_07".to_vec(), b"value_7".to_vec()));
    assert_eq!(results[4], (b"key_09".to_vec(), b"value_9".to_vec()));

    drop(db);
    cleanup_test_db("scan_tombstones");
}

#[test]
fn test_scan_empty_db() {
    let db = create_test_db("scan_empty");

    let iter = db.scan(None, None);
    let results: Vec<(Vec<u8>, Vec<u8>)> = iter.collect();

    assert_eq!(results.len(), 0);

    drop(db);
    cleanup_test_db("scan_empty");
}

#[test]
fn test_scan_with_large_dataset() {
    let db = create_test_db("scan_large");

    for i in 0..100000 {
        let key = format!("key_{:05}", i);
        let value = format!("value_{}", i);
        db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    let iter = db.scan(Some(b"key_00100"), Some(b"key_00200"));
    let results: Vec<(Vec<u8>, Vec<u8>)> = iter.collect();
    println!("{:?}", results);

    assert_eq!(results.len(), 100);
    assert_eq!(results[0], (b"key_00100".to_vec(), b"value_100".to_vec()));
    assert_eq!(results[99], (b"key_00199".to_vec(), b"value_199".to_vec()));

    drop(db);
    cleanup_test_db("scan_large");
}
