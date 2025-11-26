use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Instant;

use keylite_kv::db::Db; // <-- YOUR DB
use keylite_kv::error::DbError;

// Number of operations per test
const OPS: usize = 1_000_000;

// Helpers to compute percentiles
fn percentile(latencies: &mut [u128], p: f64) -> u128 {
    let idx = ((latencies.len() as f64) * p).ceil() as usize - 1;
    latencies[idx]
}

fn bench_put(db: &Db, rng: &mut StdRng) -> (u128, Vec<u128>) {
    let mut lat = Vec::with_capacity(OPS);

    for _ in 0..OPS {
        let key = rng.random::<u64>().to_be_bytes();
        let val = rng.random::<u64>().to_be_bytes();

        let t = Instant::now();
        db.put(&key, &val).unwrap();
        lat.push(t.elapsed().as_micros());
    }

    let total: u128 = lat.iter().sum();
    (total, lat)
}

fn bench_get(db: &Db, keys: &[Vec<u8>]) -> (u128, Vec<u128>) {
    let mut lat = Vec::with_capacity(OPS);

    for key in keys.iter() {
        let t = Instant::now();
        let _ = db.get(key).unwrap();
        lat.push(t.elapsed().as_micros());
    }

    let total: u128 = lat.iter().sum();
    (total, lat)
}

fn bench_del(db: &Db, keys: &[Vec<u8>]) -> (u128, Vec<u128>) {
    let mut lat = Vec::with_capacity(OPS);

    for key in keys.iter() {
        let t = Instant::now();
        let _ = db.del(key).unwrap();
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

fn main() -> Result<(), DbError> {
    std::fs::remove_dir_all("benchdb").ok();
    let db = Db::open("benchdb")?;

    let mut rng = StdRng::seed_from_u64(42);

    // Pre-generate keys for GET + DEL benchmark
    let mut keys = Vec::with_capacity(OPS);
    for _ in 0..OPS {
        keys.push(rng.random::<u64>().to_be_bytes().to_vec());
    }

    println!("\n╔════════════════════════════════════════════╗");
    println!("║          KeyLite Benchmark Suite           ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!("Benchmarking PUT...");
    let (put_total, mut put_lat) = bench_put(&db, &mut rng);

    println!("Benchmarking GET...");
    let (get_total, mut get_lat) = bench_get(&db, &keys);

    println!("Benchmarking DEL...");
    let (del_total, mut del_lat) = bench_del(&db, &keys);

    let total_ops = (OPS * 3) as f64;
    let total_time = (put_total + get_total + del_total) as f64;

    println!("\n╔════════════════════════════════════════════╗");
    println!("║               BENCHMARK RESULTS            ║");
    println!("╚════════════════════════════════════════════╝\n");

    print_results("PUT", put_total, &mut put_lat);
    print_results("GET", get_total, &mut get_lat);
    print_results("DEL", del_total, &mut del_lat);

    println!(
        "Overall throughput: {:.2} K ops/sec",
        (total_ops * 1_000_000.0 / total_time) / 1000.0,
    );

    Ok(())
}
