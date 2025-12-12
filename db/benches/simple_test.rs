use keylite_db::collection::Index;
use keylite_db::db::KeyLite;
use serde_json::json;

fn main() {
    std::fs::remove_dir_all("/tmp/test_db_issue").ok();
    let db = KeyLite::open("/tmp/test_db_issue").unwrap();
    
    println!("Creating collection without index...");
    db.create_collection("col_no_idx", None).unwrap();
    println!("Created col_no_idx");
    
    println!("Creating collection with index...");
    db.create_collection(
        "col_with_idx",
        Some(vec![Index {
            field: "email".to_string(),
            unique: true,
        }]),
    ).unwrap();
    println!("Created col_with_idx");
    
    println!("Verifying col_with_idx indexes...");
    let indexes = db.list_index("col_with_idx").unwrap();
    println!("Indexes: {:?}", indexes);
    
    println!("\nInserting 5000 docs into col_no_idx...");
    for i in 0..5000 {
        let doc = json!({
            "name": format!("user_{}", i),
            "value": i,
        });
        db.insert("col_no_idx", doc).unwrap();
        if i % 1000 == 0 {
            println!("  Inserted {} docs...", i);
        }
    }
    println!("Done inserting into col_no_idx");
    
    println!("\nVerifying col_with_idx still exists...");
    match db.list_index("col_with_idx") {
        Ok(indexes) => println!("Indexes: {:?}", indexes),
        Err(e) => println!("ERROR listing indexes: {:?}", e),
    }
    
    println!("\nInserting 1 doc into col_with_idx...");
    let doc = json!({
        "name": "test",
        "email": "test@example.com",
    });
    match db.insert("col_with_idx", doc) {
        Ok(id) => println!("Successfully inserted doc with id: {}", id),
        Err(e) => println!("ERROR inserting: {:?}", e),
    }
    
    std::fs::remove_dir_all("/tmp/test_db_issue").ok();
}
