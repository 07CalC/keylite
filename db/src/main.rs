use std::time::Instant;

use keylite_db::db::KeyLite;

fn main() {
    let db = KeyLite::open("testdb").unwrap();
    let t1 = Instant::now();
    db.create_collection("users").unwrap();
    println!("time taken to create_collection: {:?}", t1.elapsed());
    let doc = serde_json::json!({
        "name": "vin",
        "class": "low class",
        "age": 10
    });

    let t2 = Instant::now();
    let id = db.insert("users", doc).unwrap();
    println!("time taken to insert doc: {:?}", t2.elapsed());
    let t3 = Instant::now();
    let res = db.get_doc_by_id("users", &id);
    println!("time taken to get doc: {:?}", t3.elapsed());
    let val = serde_json::json!(res.unwrap());
    println!("{:?}", val);
}
