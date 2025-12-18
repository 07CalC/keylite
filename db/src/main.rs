use std::{fs, time::Instant};

use keylite_db::{collection::Index, db::KeyLite, filter::Filter};
use serde_json::json;

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

    let users: Vec<_> = (1..=50000)
        .map(|i| {
            json!({
                "name": format!("User{}", i),
                "email": format!("user{}@example.com", i),
                "age": 18 + (i % 40)  // ages 18–57
            })
        })
        .collect();
    for user in users {
        db.insert("users", user.into()).unwrap();
    }

    let user = db.get_by_index("users", "email", &"rachel@example.com".into());
    println!("{:?}", user.unwrap().len());

    let t4 = Instant::now();
    let user3 = db.get_by_field_forced("users", "name", &"violet".into());
    println!(
        "{:?}\n time taken to get by force db {:?}",
        t4.elapsed(),
        user3.unwrap().len()
    );

    println!("\n=== Testing Query API ===");

    let t5 = Instant::now();
    let query_result = db
        .query("users")
        .filter(Filter::Gt {
            field: "age".to_string(),
            value: json!(30),
        })
        .execute()
        .unwrap();
    println!(
        "Users with age > 30: {} (took {:?})",
        query_result.len(),
        t5.elapsed()
    );
    for user in &query_result[..3.min(query_result.len())] {
        println!("  - {}: age {}", user["name"], user["age"]);
    }

    // Test 2: Filter with sorting
    let t6 = Instant::now();
    let sorted_result = db
        .query("users")
        .filter(Filter::Gt {
            field: "age".to_string(),
            value: json!(25),
        })
        .sort("age", true)
        .limit(5)
        .execute()
        .unwrap();
    println!(
        "\nTop 5 youngest users with age > 25 (took {:?}):",
        t6.elapsed()
    );
    for user in sorted_result {
        println!("  - {}: age {}", user["name"], user["age"]);
    }

    // Test 3: Filter with skip and limit
    let t7 = Instant::now();
    let paginated_result = db
        .query("users")
        .sort("name", true)
        .skip(5)
        .limit(3)
        .execute()
        .unwrap();
    println!("\nUsers 6-8 (sorted by name) (took {:?}):", t7.elapsed());
    for user in paginated_result {
        println!("  - {}", user["name"]);
    }

    let t8 = Instant::now();
    let multi_filter_result = db
        .query("users")
        .filter(Filter::Gt {
            field: "age".to_string(),
            value: json!(25),
        })
        .filter(Filter::Lt {
            field: "age".to_string(),
            value: json!(30),
        })
        .filter(Filter::Exists {
            field: "email".to_string(),
        })
        .sort("age", false)
        .execute()
        .unwrap();
    println!(
        "\nUsers with age between 25 and 30 (sorted desc) (took {:?}):",
        t8.elapsed()
    );
    for user in multi_filter_result {
        println!("  - {}: age {}", user["name"], user["age"]);
    }

    let t9 = Instant::now();
    let in_filter_result = db
        .query("users")
        .filter(Filter::In {
            field: "age".to_string(),
            values: vec![json!(25), json!(30), json!(35)],
        })
        .sort("age", true)
        .execute()
        .unwrap();
    println!(
        "\nUsers with age in [25, 30, 35] (took {:?}):",
        t9.elapsed()
    );
    for user in in_filter_result {
        println!("  - {}: age {}", user["name"], user["age"]);
    }
    fs::remove_dir("testdb");
}
