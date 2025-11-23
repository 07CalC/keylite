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

const THREADS: usize = 25;
const OPS_PER_THREAD: usize = 2000;
const OPS_PER_ONE: usize = 3; // put, get, del

// total latency samples
const TOTAL_SAMPLES: usize = THREADS * OPS_PER_THREAD * OPS_PER_ONE;

// ----- LATENCY HELPERS -----
fn record_latency(buf: &Arc<Vec<AtomicU64>>, index: &AtomicU64, nanos: u64) {
    let i = index.fetch_add(1, Ordering::Relaxed) as usize;

    if i >= buf.len() {
        return; // safeguard (shouldn't normally happen)
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
    println!("=== KeyLite 25-Thread / 2000 Ops Benchmark ===");

    let _ = std::fs::remove_dir_all("bench_db");
    let db = Arc::new(Db::open("bench_db").unwrap());

    // latency buffer sized EXACTLY to avoid panic
    let latencies = Arc::new(
        (0..TOTAL_SAMPLES)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );
    let idx = Arc::new(AtomicU64::new(0));

    let mut handles = vec![];
    let start = Instant::now();

    for id in 0..THREADS {
        let db = Arc::clone(&db);
        let lat = Arc::clone(&latencies);
        let idx = Arc::clone(&idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64((id as u64 + 1) * 9999);

            for _ in 0..OPS_PER_THREAD {
                let key = format!("key_{}_{}", id, rng.random::<u64>());
                let val = format!("val_{}", rng.random::<u64>());

                // put
                let t0 = Instant::now();
                db.put(key.as_bytes(), val.as_bytes()).unwrap();
                record_latency(&lat, &idx, t0.elapsed().as_nanos() as u64);

                // get
                let t0 = Instant::now();
                let _ = db.get(key.as_bytes());
                record_latency(&lat, &idx, t0.elapsed().as_nanos() as u64);

                // delete
                let t0 = Instant::now();
                let _ = db.del(key.as_bytes());
                record_latency(&lat, &idx, t0.elapsed().as_nanos() as u64);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // number of samples actually written
    let used_samples = idx.load(Ordering::Relaxed) as usize;
    let total_time = start.elapsed();
    let ops_per_sec = used_samples as f64 / total_time.as_secs_f64();

    // compute percentiles
    let (p50, p90, p99) = compute_percentiles(&latencies, used_samples);

    // OUTPUT
    println!("\n=== RESULTS ===");
    println!("Total operations recorded: {}", used_samples);
    println!("Total time: {:?}", total_time);
    println!("Ops/sec: {:.2}", ops_per_sec);

    println!("\n--- Latency (ns) ---");
    println!("p50: {}", p50);
    println!("p90: {}", p90);
    println!("p99: {}", p99);

    println!("\n=== Benchmark Complete ===");
}
