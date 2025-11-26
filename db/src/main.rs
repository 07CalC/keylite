use std::time::Instant;

use keylite_db::{collection::Index, db::KeyLite};

fn main() {
    let db = KeyLite::open("testdb").unwrap();

    let indexes = vec![
        Index {
            field: "email".to_string(),
            unique: true,
        },
        Index {
            field: "age".to_string(),
            unique: false,
        },
    ];

    let t1 = Instant::now();
    db.create_collection("users", Some(indexes)).unwrap();
    println!("time taken to create_collection: {:?}", t1.elapsed());

    let users = vec![
        serde_json::json!({
            "name": "Alice",
            "email": "alice@example.com",
            "age": 25
        }),
        serde_json::json!({
            "name": "Bob",
            "email": "bob@example.com",
            "age": 30
        }),
        serde_json::json!({
            "name": "Charlie",
            "email": "charlie@example.com",
            "age": 25
        }),
    ];

    for user in users {
        db.insert("users", user).unwrap();
    }
    println!("Inserted 3 users\n");

    println!("=== Testing Unique Index (email) ===");
    let t2 = Instant::now();
    let results = db
        .get_by_index("users", "email", &serde_json::json!("alice@example.com"))
        .unwrap();
    println!("time taken to query by unique index: {:?}", t2.elapsed());
    println!("Found {} document(s):", results.len());
    for doc in results {
        println!("  {:?}", doc);
    }

    println!("\n=== Testing Non-Unique Index (age) ===");
    let t3 = Instant::now();
    let results = db
        .get_by_index("users", "age", &serde_json::json!(25))
        .unwrap();
    println!(
        "time taken to query by non-unique index: {:?}",
        t3.elapsed()
    );
    println!("Found {} document(s) with age=25:", results.len());
    for doc in results {
        println!("  {:?}", doc);
    }

    println!("\n=== All Users ===");
    let all_users = db.scan_collection("users").unwrap();
    println!("Total users: {}", all_users.len());
    for user in all_users {
        println!("  {:?}", user);
    }
}
