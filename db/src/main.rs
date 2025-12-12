use std::time::Instant;

use keylite_db::{collection::Index, db::KeyLite};
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
    db.create_collection("users", None).unwrap();
    println!("time taken to create_collection: {:?}", t1.elapsed());

    let users = vec![
        json!({"name": "Alice", "email": "alice@example.com", "age": 25}),
        json!({"name": "Bob", "email": "bob@example.com", "age": 30}),
        json!({"name": "Charlie", "email": "charlie@example.com", "age": 22}),
        json!({"name": "David", "email": "david@example.com", "age": 28}),
        json!({"name": "Eve", "email": "eve@example.com", "age": 26}),
        json!({"name": "Frank", "email": "frank@example.com", "age": 33}),
        json!({"name": "Grace", "email": "grace@example.com", "age": 24}),
        json!({"name": "Heidi", "email": "heidi@example.com", "age": 27}),
        json!({"name": "Ivan", "email": "ivan@example.com", "age": 35}),
        json!({"name": "Judy", "email": "judy@example.com", "age": 29}),
        json!({"name": "Karl", "email": "karl@example.com", "age": 31}),
        json!({"name": "Laura", "email": "laura@example.com", "age": 23}),
        json!({"name": "Mallory", "email": "mallory@example.com", "age": 34}),
        json!({"name": "Niaj", "email": "niaj@example.com", "age": 21}),
        json!({"name": "Olivia", "email": "olivia@example.com", "age": 26}),
        json!({"name": "Peggy", "email": "peggy@example.com", "age": 28}),
        json!({"name": "Quentin", "email": "quentin@example.com", "age": 32}),
        json!({"name": "Ruth", "email": "ruth@example.com", "age": 27}),
        json!({"name": "Sybil", "email": "sybil@example.com", "age": 24}),
        json!({"name": "Trent", "email": "trent@example.com", "age": 30}),
        json!({"name": "Uma", "email": "uma@example.com", "age": 22}),
        json!({"name": "Victor", "email": "victor@example.com", "age": 33}),
        json!({"name": "Wendy", "email": "wendy@example.com", "age": 25}),
        json!({"name": "Xavier", "email": "xavier@example.com", "age": 29}),
        json!({"name": "Yvonne", "email": "yvonne@example.com", "age": 31}),
        json!({"name": "Zack", "email": "zack@example.com", "age": 23}),
        json!({"name": "Aaron", "email": "aaron@example.com", "age": 34}),
        json!({"name": "Bella", "email": "bella@example.com", "age": 28}),
        json!({"name": "Carter", "email": "carter@example.com", "age": 27}),
        json!({"name": "Diana", "email": "diana@example.com", "age": 22}),
        json!({"name": "Ethan", "email": "ethan@example.com", "age": 26}),
        json!({"name": "Fiona", "email": "fiona@example.com", "age": 30}),
        json!({"name": "George", "email": "george@example.com", "age": 35}),
        json!({"name": "Hannah", "email": "hannah@example.com", "age": 24}),
        json!({"name": "Isaac", "email": "isaac@example.com", "age": 28}),
        json!({"name": "Jessica", "email": "jessica@example.com", "age": 29}),
        json!({"name": "Kevin", "email": "kevin@example.com", "age": 33}),
        json!({"name": "Lily", "email": "lily@example.com", "age": 25}),
        json!({"name": "Mason", "email": "mason@example.com", "age": 27}),
        json!({"name": "Nora", "email": "nora@example.com", "age": 26}),
        json!({"name": "Owen", "email": "owen@example.com", "age": 32}),
        json!({"name": "Paula", "email": "paula@example.com", "age": 23}),
        json!({"name": "Quincy", "email": "quincy@example.com", "age": 34}),
        json!({"name": "Rachel", "email": "rachel@example.com", "age": 28}),
        json!({"name": "Samuel", "email": "samuel@example.com", "age": 30}),
        json!({"name": "Tina", "email": "tina@example.com", "age": 21}),
        json!({"name": "Umar", "email": "umar@example.com", "age": 24}),
        json!({"name": "Violet", "email": "violet@example.com", "age": 29}),
        json!({"name": "Wyatt", "email": "wyatt@example.com", "age": 31}),
        json!({"name": "Zara", "email": "zara@example.com", "age": 22}),
    ];

    // let t2 = Instant::now();
    // for user in users {
    //     db.insert("users", user).unwrap();
    // }
    // println!("Inserted 50 users in {:?}\n", t2.elapsed());
    //
    // let t3 = Instant::now();
    // let result = db.get_by_field_forced("users", "email", &"violet@emample.com".into());
    // if result.is_ok() {
    //     println!("{:?}", result.unwrap());
    // }
    // println!("got in {:?} via force", t3.elapsed());
    //
    // let t4 = Instant::now();
    // let result2 = db.get_by_index("users", "email", &"violet@example.com".into());
    // if result2.is_ok() {
    //     println!("{:?}", result2);
    // }
    // println!("got in {:?} via index", t4.elapsed());
    //
    let indexes = db.list_index("users");

    let t2 = Instant::now();
    println!("{:?}", indexes.unwrap());
    println!("time taken: {:?}", t2.elapsed());
}
