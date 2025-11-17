use keylite::db::Db;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

/// Reserve large fixed-size buckets to store individual operation latencies.
/// Each latency is stored in nanoseconds. Stats are extracted at end.
const MAX_SAMPLES: usize = 2_000_000;

fn record_latency(buf: &Arc<Vec<AtomicU64>>, index: &AtomicU64, nanos: u64) {
    let i = index.fetch_add(1, Ordering::Relaxed);
    if (i as usize) < MAX_SAMPLES {
        buf[i as usize].store(nanos, Ordering::Relaxed);
    }
}

fn compute_percentiles(buf: &Arc<Vec<AtomicU64>>, count: u64) -> (u64, u64, u64) {
    let mut v = Vec::with_capacity(count as usize);
    for i in 0..count {
        let ns = buf[i as usize].load(Ordering::Relaxed);
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

fn main() {
    println!("=== KeyLite Full Concurrency Bench ===\n");

    // Clean DB directory
    let _ = std::fs::remove_dir_all("concurrency_test");

    // Measure DB open latency
    let t_open = Instant::now();
    let db = Arc::new(Db::open("concurrency_test").unwrap());
    let open_time = t_open.elapsed();
    println!("DB open time: {:?}\n", open_time);

    const WRITERS: usize = 4;
    const READERS: usize = 4;
    const DELETERS: usize = 2;
    const TEST_DURATION: Duration = Duration::from_secs(10);

    let write_count = Arc::new(AtomicU64::new(0));
    let read_count = Arc::new(AtomicU64::new(0));
    let del_count = Arc::new(AtomicU64::new(0));

    let write_lat = Arc::new(
        (0..MAX_SAMPLES)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );
    let read_lat = Arc::new(
        (0..MAX_SAMPLES)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );
    let del_lat = Arc::new(
        (0..MAX_SAMPLES)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );

    // Index counters
    let write_idx = Arc::new(AtomicU64::new(0));
    let read_idx = Arc::new(AtomicU64::new(0));
    let del_idx = Arc::new(AtomicU64::new(0));

    let start = Instant::now();
    let mut handles = vec![];

    // ---------------- Writers -----------------
    for id in 0..WRITERS {
        let db = Arc::clone(&db);
        let write_count = Arc::clone(&write_count);
        let write_lat = Arc::clone(&write_lat);
        let write_idx = Arc::clone(&write_idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(id as u64 * 9999 + 1);

            while start.elapsed() < TEST_DURATION {
                let key = format!("key_w{}_{}", id, rng.gen::<u64>());
                let val = format!("value_{}", rng.gen::<u64>());

                let t0 = Instant::now();
                db.put(key.as_bytes(), val.as_bytes()).unwrap();
                let nanos = t0.elapsed().as_nanos() as u64;

                record_latency(&write_lat, &write_idx, nanos);
                write_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // ---------------- Readers -----------------
    for id in 0..READERS {
        let db = Arc::clone(&db);
        let read_count = Arc::clone(&read_count);
        let read_lat = Arc::clone(&read_lat);
        let read_idx = Arc::clone(&read_idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(id as u64 * 5555 + 2);

            while start.elapsed() < TEST_DURATION {
                let key = format!("key_w{}_{}", rng.gen_range(0..WRITERS), rng.gen::<u64>());

                let t0 = Instant::now();
                let _ = db.get(key.as_bytes());
                let nanos = t0.elapsed().as_nanos() as u64;

                record_latency(&read_lat, &read_idx, nanos);
                read_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // ---------------- Deleters -----------------
    for id in 0..DELETERS {
        let db = Arc::clone(&db);
        let del_count = Arc::clone(&del_count);
        let del_lat = Arc::clone(&del_lat);
        let del_idx = Arc::clone(&del_idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(id as u64 * 1234 + 3);

            while start.elapsed() < TEST_DURATION {
                let key = format!("key_w{}_{}", rng.gen_range(0..WRITERS), rng.gen::<u64>());

                let t0 = Instant::now();
                let _ = db.del(key.as_bytes());
                let nanos = t0.elapsed().as_nanos() as u64;

                record_latency(&del_lat, &del_idx, nanos);
                del_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // Wait all threads
    for h in handles {
        h.join().unwrap();
    }

    // Final tallies
    let wc = write_count.load(Ordering::Relaxed);
    let rc = read_count.load(Ordering::Relaxed);
    let dc = del_count.load(Ordering::Relaxed);

    println!("\n=== RESULTS ===\n");

    println!("DB open latency: {:?}", open_time);

    println!(
        "Writes:  {} ops  ({:.2} ops/sec)",
        wc,
        wc as f64 / TEST_DURATION.as_secs_f64()
    );
    println!(
        "Reads:   {} ops  ({:.2} ops/sec)",
        rc,
        rc as f64 / TEST_DURATION.as_secs_f64()
    );
    println!(
        "Deletes: {} ops  ({:.2} ops/sec)",
        dc,
        dc as f64 / TEST_DURATION.as_secs_f64()
    );

    // Percentiles
    let (w50, w90, w99) = compute_percentiles(&write_lat, wc);
    let (r50, r90, r99) = compute_percentiles(&read_lat, rc);
    let (d50, d90, d99) = compute_percentiles(&del_lat, dc);

    println!("\n--- Write Latency (ns) ---");
    println!("p50: {} ns,  p90: {} ns,  p99: {} ns", w50, w90, w99);

    println!("\n--- Read Latency (ns) ---");
    println!("p50: {} ns,  p90: {} ns,  p99: {} ns", r50, r90, r99);

    println!("\n--- Delete Latency (ns) ---");
    println!("p50: {} ns,  p90: {} ns,  p99: {} ns", d50, d90, d99);

    println!("\n=== Test Complete ===");
}
