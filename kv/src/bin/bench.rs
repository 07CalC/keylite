use keylite_kv::db::Db;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

const THREADS: usize = 1;
const OPS_PER_THREAD: usize = 150000;

// TOTAL ops for each category (write / read / delete)
const TOTAL_OPS: usize = THREADS * OPS_PER_THREAD;

// ----- LATENCY HELPERS -----
fn record_latency(buf: &Arc<Vec<AtomicU64>>, index: &AtomicU64, nanos: u64) {
    let i = index.fetch_add(1, Ordering::Relaxed) as usize;
    if i >= buf.len() {
        return;
    }
    buf[i].store(nanos, Ordering::Relaxed);
}

fn compute_percentiles(buf: &Arc<Vec<AtomicU64>>, count: usize) -> (u64, u64, u64) {
    let mut v = Vec::with_capacity(count);
    for i in 0..count {
        let ns = buf[i].load(Ordering::Relaxed);
        if ns > 0 {
            v.push(ns);
        }
    }
    if v.is_empty() {
        return (0, 0, 0);
    }
    v.sort_unstable();
    let p50 = v[(v.len() as f64 * 0.50) as usize];
    let p90 = v[(v.len() as f64 * 0.90) as usize];
    let p99 = v[(v.len() as f64 * 0.99) as usize];
    (p50, p90, p99)
}

// ----- MAIN -----
fn main() {
    println!("=== KeyLite 50-Thread / 15000 Ops Benchmark ===");

    let _ = std::fs::remove_dir_all("bench_db");
    let db = Arc::new(Db::open("bench_db").unwrap());

    // Separate latency storage
    let write_lat = Arc::new(
        (0..TOTAL_OPS)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );
    let read_lat = Arc::new(
        (0..TOTAL_OPS)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );
    let del_lat = Arc::new(
        (0..TOTAL_OPS)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );

    let write_idx = Arc::new(AtomicU64::new(0));
    let read_idx = Arc::new(AtomicU64::new(0));
    let del_idx = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];
    let start = Instant::now();

    for id in 0..THREADS {
        let db = Arc::clone(&db);
        let write_lat = Arc::clone(&write_lat);
        let read_lat = Arc::clone(&read_lat);
        let del_lat = Arc::clone(&del_lat);

        let write_idx = Arc::clone(&write_idx);
        let read_idx = Arc::clone(&read_idx);
        let del_idx = Arc::clone(&del_idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64((id as u64 + 1) * 9999);

            for _ in 0..OPS_PER_THREAD {
                let key = format!("key_{}_{}", id, rng.random::<u64>());
                let val = format!("val_{}", rng.random::<u64>());

                // write
                let t0 = Instant::now();
                db.put(key.as_bytes(), val.as_bytes()).unwrap();
                record_latency(&write_lat, &write_idx, t0.elapsed().as_nanos() as u64);

                // read
                let t0 = Instant::now();
                let _ = db.get(key.as_bytes());
                record_latency(&read_lat, &read_idx, t0.elapsed().as_nanos() as u64);

                // delete
                let t0 = Instant::now();
                let _ = db.del(key.as_bytes());
                record_latency(&del_lat, &del_idx, t0.elapsed().as_nanos() as u64);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let total_time = start.elapsed();

    let writes_done = write_idx.load(Ordering::Relaxed) as usize;
    let reads_done = read_idx.load(Ordering::Relaxed) as usize;
    let dels_done = del_idx.load(Ordering::Relaxed) as usize;

    // Percentiles
    let (w50, w90, w99) = compute_percentiles(&write_lat, writes_done);
    let (r50, r90, r99) = compute_percentiles(&read_lat, reads_done);
    let (d50, d90, d99) = compute_percentiles(&del_lat, dels_done);

    println!("\n=== RESULTS ===");
    println!("Total time: {:?}", total_time);

    println!(
        "\nWrites:  {} ops  ({:.2} ops/sec)",
        writes_done,
        writes_done as f64 / total_time.as_secs_f64()
    );
    println!(
        "Reads:   {} ops  ({:.2} ops/sec)",
        reads_done,
        reads_done as f64 / total_time.as_secs_f64()
    );
    println!(
        "Deletes: {} ops  ({:.2} ops/sec)",
        dels_done,
        dels_done as f64 / total_time.as_secs_f64()
    );

    println!("\n--- Write Latency (ns) ---");
    println!("p50: {},  p90: {},  p99: {}", w50, w90, w99);

    println!("\n--- Read Latency (ns) ---");
    println!("p50: {},  p90: {},  p99: {}", r50, r90, r99);

    println!("\n--- Delete Latency (ns) ---");
    println!("p50: {},  p90: {},  p99: {}", d50, d90, d99);

    println!("\n=== Benchmark Complete ===");
}
