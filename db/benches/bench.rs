use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde_json::{Value, json};
use std::time::Instant;

use keylite_db::collection::Index;
use keylite_db::db::KeyLite;

// Number of operations per test
const OPS: usize = 10_000;

// Helpers to compute percentiles
fn percentile(latencies: &mut [u128], p: f64) -> u128 {
    let idx = ((latencies.len() as f64) * p).ceil() as usize - 1;
    latencies[idx]
}

fn bench_insert(db: &KeyLite, collection: &str, rng: &mut StdRng) -> (u128, Vec<u128>) {
    println!("benchmarking inserts for col: {:?}", collection);
    let mut lat = Vec::with_capacity(OPS);

    for _ in 0..OPS {
        let user_id: u32 = rng.r#gen();
        let email_id: u32 = rng.r#gen();
        let doc = json!({
            "name": format!("user_{}", user_id),
            "age": rng.gen_range(18..80),
            "email": format!("user_{}@example.com", email_id),
            "score": rng.gen_range(0..1000),
        });

        let t = Instant::now();
        db.insert(collection, doc).unwrap();
        lat.push(t.elapsed().as_micros());
    }

    let total: u128 = lat.iter().sum();
    println!("done");
    (total, lat)
}

fn bench_get_by_id(db: &KeyLite, collection: &str, ids: &[String]) -> (u128, Vec<u128>) {
    let mut lat = Vec::with_capacity(ids.len());

    for id in ids.iter() {
        let t = Instant::now();
        let _ = db.get_doc_by_id(collection, id).unwrap();
        lat.push(t.elapsed().as_micros());
    }

    let total: u128 = lat.iter().sum();
    (total, lat)
}

fn bench_get_by_index(
    db: &KeyLite,
    collection: &str,
    field: &str,
    values: &[Value],
) -> (u128, Vec<u128>) {
    let mut lat = Vec::with_capacity(values.len());

    for value in values.iter() {
        let t = Instant::now();
        let _ = db.get_by_index(collection, field, value).unwrap();
        lat.push(t.elapsed().as_micros());
    }

    let total: u128 = lat.iter().sum();
    (total, lat)
}

fn bench_scan_collection(db: &KeyLite, collection: &str) -> u128 {
    let t = Instant::now();
    let _ = db.scan_collection(collection).unwrap();
    t.elapsed().as_micros()
}

fn bench_delete_by_id(db: &KeyLite, collection: &str, ids: &[String]) -> (u128, Vec<u128>) {
    let mut lat = Vec::with_capacity(ids.len());

    for id in ids.iter() {
        let t = Instant::now();
        let _ = db.delete_doc_by_id(collection, id).unwrap();
        lat.push(t.elapsed().as_micros());
    }

    let total: u128 = lat.iter().sum();
    (total, lat)
}

fn print_results(name: &str, total_time_us: u128, lat: &mut [u128]) {
    lat.sort_unstable();

    let p50 = percentile(lat, 0.50);
    let p90 = percentile(lat, 0.90);
    let p99 = percentile(lat, 0.99);

    let throughput = (lat.len() as f64 * 1_000_000.0) / total_time_us as f64;

    println!("┌──────────────────────────────────────");
    println!("│  {name}");
    println!("├──────────────────────────────────────");
    println!("│  ops:              {}", lat.len());
    println!("│  throughput:       {:.2} K ops/sec", throughput / 1000.0);
    println!("│  latency p50:      {} µs", p50);
    println!("│  latency p90:      {} µs", p90);
    println!("│  latency p99:      {} µs", p99);
    println!("└──────────────────────────────────────\n");
}

fn main() {
    std::fs::remove_dir_all("benchdb").ok();
    let db = KeyLite::open("benchdb").unwrap();

    let mut rng = StdRng::seed_from_u64(42);

    println!("\n╔════════════════════════════════════════════╗");
    println!("║       KeyLite DB Benchmark Suite          ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Setup: Create collections with and without indexes
    println!("Setting up collections...");
    db.create_collection("users_no_index", None).unwrap();

    // Try to read it back
    println!("Verifying users_no_index...");
    println!("{:?}", db.list_index("users_no_index").unwrap());

    db.create_collection(
        "users_with_index",
        Some(vec![
            Index {
                field: "email".to_string(),
                unique: true,
            },
            Index {
                field: "age".to_string(),
                unique: false,
            },
        ]),
    )
    .unwrap();

    // Verify collections exist
    println!("Verifying users_with_index...");
    println!("{:?}", db.list_index("users_with_index").unwrap());

    // Benchmark 1: Insert without indexes
    println!("\nBenchmarking INSERT (no indexes)...");
    let (insert_no_idx_total, mut insert_no_idx_lat) =
        bench_insert(&db, "users_no_index", &mut rng);

    // Collect IDs for later benchmarks
    let docs_no_idx = db.scan_collection("users_no_index").unwrap();
    let ids_no_idx: Vec<String> = docs_no_idx
        .iter()
        .map(|doc| doc.get("_id").unwrap().as_str().unwrap().to_string())
        .collect();

    // Benchmark 2: Insert with indexes
    println!("Benchmarking INSERT (with unique + non-unique indexes)...");
    let (insert_with_idx_total, mut insert_with_idx_lat) =
        bench_insert(&db, "users_with_index", &mut rng);

    // Collect IDs and values for later benchmarks
    let docs_with_idx = db.scan_collection("users_with_index").unwrap();
    let ids_with_idx: Vec<String> = docs_with_idx
        .iter()
        .map(|doc| doc.get("_id").unwrap().as_str().unwrap().to_string())
        .collect();

    // Sample age values for index queries
    let age_values: Vec<Value> = docs_with_idx
        .iter()
        .take(1000)
        .map(|doc| doc.get("age").cloned().unwrap())
        .collect();

    // Benchmark 3: Get by ID (no indexes)
    println!("Benchmarking GET BY ID (no indexes)...");
    let sample_ids_no_idx: Vec<String> = ids_no_idx.iter().take(10000).cloned().collect();
    let (get_id_no_idx_total, mut get_id_no_idx_lat) =
        bench_get_by_id(&db, "users_no_index", &sample_ids_no_idx);

    // Benchmark 4: Get by ID (with indexes)
    println!("Benchmarking GET BY ID (with indexes)...");
    let sample_ids_with_idx: Vec<String> = ids_with_idx.iter().take(10000).cloned().collect();
    let (get_id_with_idx_total, mut get_id_with_idx_lat) =
        bench_get_by_id(&db, "users_with_index", &sample_ids_with_idx);

    // Benchmark 5: Get by index (non-unique)
    println!("Benchmarking GET BY INDEX (non-unique: age)...");
    let (get_idx_total, mut get_idx_lat) =
        bench_get_by_index(&db, "users_with_index", "age", &age_values);

    // Benchmark 6: Scan collection (no indexes)
    println!("Benchmarking SCAN COLLECTION (no indexes)...");
    let scan_no_idx_time = bench_scan_collection(&db, "users_no_index");

    // Benchmark 7: Scan collection (with indexes)
    println!("Benchmarking SCAN COLLECTION (with indexes)...");
    let scan_with_idx_time = bench_scan_collection(&db, "users_with_index");

    // Benchmark 8: Delete by ID (no indexes)
    println!("Benchmarking DELETE BY ID (no indexes)...");
    let delete_ids_no_idx: Vec<String> = ids_no_idx.iter().take(10000).cloned().collect();
    let (del_no_idx_total, mut del_no_idx_lat) =
        bench_delete_by_id(&db, "users_no_index", &delete_ids_no_idx);

    // Benchmark 9: Delete by ID (with indexes)
    println!("Benchmarking DELETE BY ID (with indexes)...");
    let delete_ids_with_idx: Vec<String> = ids_with_idx.iter().take(10000).cloned().collect();
    let (del_with_idx_total, mut del_with_idx_lat) =
        bench_delete_by_id(&db, "users_with_index", &delete_ids_with_idx);

    // Print Results
    println!("\n╔════════════════════════════════════════════╗");
    println!("║               BENCHMARK RESULTS            ║");
    println!("╚════════════════════════════════════════════╝\n");

    print_results(
        "INSERT (no indexes)",
        insert_no_idx_total,
        &mut insert_no_idx_lat,
    );
    print_results(
        "INSERT (with indexes)",
        insert_with_idx_total,
        &mut insert_with_idx_lat,
    );

    println!("┌──────────────────────────────────────");
    println!("│  Index Overhead");
    println!("├──────────────────────────────────────");
    let overhead_pct = ((insert_with_idx_total as f64 - insert_no_idx_total as f64)
        / insert_no_idx_total as f64)
        * 100.0;
    println!("│  overhead:         {:.2}%", overhead_pct);
    println!("└──────────────────────────────────────\n");

    print_results(
        "GET BY ID (no indexes)",
        get_id_no_idx_total,
        &mut get_id_no_idx_lat,
    );
    print_results(
        "GET BY ID (with indexes)",
        get_id_with_idx_total,
        &mut get_id_with_idx_lat,
    );

    print_results("GET BY INDEX (non-unique)", get_idx_total, &mut get_idx_lat);

    println!("┌──────────────────────────────────────");
    println!("│  SCAN COLLECTION (no indexes)");
    println!("├──────────────────────────────────────");
    println!("│  total docs:       {}", docs_no_idx.len());
    println!("│  total time:       {} µs", scan_no_idx_time);
    println!(
        "│  throughput:       {:.2} K docs/sec",
        (docs_no_idx.len() as f64 * 1_000_000.0 / scan_no_idx_time as f64) / 1000.0
    );
    println!("└──────────────────────────────────────\n");

    println!("┌──────────────────────────────────────");
    println!("│  SCAN COLLECTION (with indexes)");
    println!("├──────────────────────────────────────");
    println!("│  total docs:       {}", docs_with_idx.len());
    println!("│  total time:       {} µs", scan_with_idx_time);
    println!(
        "│  throughput:       {:.2} K docs/sec",
        (docs_with_idx.len() as f64 * 1_000_000.0 / scan_with_idx_time as f64) / 1000.0
    );
    println!("└──────────────────────────────────────\n");

    print_results(
        "DELETE BY ID (no indexes)",
        del_no_idx_total,
        &mut del_no_idx_lat,
    );
    print_results(
        "DELETE BY ID (with indexes)",
        del_with_idx_total,
        &mut del_with_idx_lat,
    );

    // Calculate overall throughput
    let total_ops = (OPS * 2)
        + (sample_ids_no_idx.len() * 2)
        + age_values.len()
        + (delete_ids_no_idx.len() * 2)
        + 2; // +2 for scans
    let total_time = insert_no_idx_total
        + insert_with_idx_total
        + get_id_no_idx_total
        + get_id_with_idx_total
        + get_idx_total
        + scan_no_idx_time
        + scan_with_idx_time
        + del_no_idx_total
        + del_with_idx_total;

    println!(
        "Overall throughput: {:.2} K ops/sec",
        (total_ops as f64 * 1_000_000.0 / total_time as f64) / 1000.0,
    );

    drop(db);
    // std::fs::remove_dir_all("benchdb").ok();
}
