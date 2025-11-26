use std::time::Instant;

use keylite_db::db::KeyLite;

fn main() {
    let db = KeyLite::open("testdb").unwrap();
    let t1 = Instant::now();
    db.create_collection("users").unwrap();
    println!("time taken to create_collection: {:?}", t1.elapsed());
    let doc = serde_json::json!({
        "name": "tanish",
        "class": "low class",
        "age": 10
    });

    let t2 = Instant::now();
    // for _ in 0..1000 {
    //     let _ = db.insert("users", doc.clone()).unwrap();
    // }
    println!("time taken to insert doc: {:?}", t2.elapsed());
    let t3 = Instant::now();
    // let res = db.get_doc_by_id("users", &id);
    // println!("time taken to get doc: {:?}", t3.elapsed());
    // let val = serde_json::json!(res.unwrap());
    // println!("{:?}", val);

    let val = db.scan_collection("users").unwrap();
    println!(
        "time taken to scan {:?} users: {:?}",
        val.len(),
        t3.elapsed()
    );
    // for r in val.iter() {
    //     println!("{:?}", r);
    // }
}
