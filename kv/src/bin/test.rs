use keylite_kv::db::Db;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

const MAX_SAMPLES: usize = 2_000_000;

// ---- MEMORY MEASUREMENT (LINUX ONLY) ----
fn get_memory_usage_kb() -> u64 {
    // Reads process RSS (resident set size)
    let statm = std::fs::read_to_string("/proc/self/statm").unwrap();
    let parts: Vec<&str> = statm.split_whitespace().collect();
    let rss_pages: u64 = parts[1].parse().unwrap();
    let page_size = 4096u64; // most Linux systems: 4 KB/page
    rss_pages * page_size / 1024
}

// ---- BENCH UTILITIES ----
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

// ---- MAIN BENCH ----
fn main() {
    println!("=== KeyLite Full Concurrency Bench ===\n");

    let _ = std::fs::remove_dir_all("concurrency_test");

    // ---- MEMORY BEFORE OPEN ----
    let mem_before_open = get_memory_usage_kb();
    println!("Memory before DB open: {} KB", mem_before_open);

    // ---- OPEN DB ----
    let t_open = Instant::now();
    let db = Arc::new(Db::open("concurrency_test").unwrap());
    let open_time = t_open.elapsed();

    // ---- MEMORY AFTER OPEN ----
    let mem_after_open = get_memory_usage_kb();
    let mem_used_open = mem_after_open - mem_before_open;

    println!("DB open time: {:?}", open_time);
    println!("Memory after DB open: {} KB", mem_after_open);
    println!("DB used during open: {} KB\n", mem_used_open);

    // ---- BENCH CONFIG ----
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

    let write_idx = Arc::new(AtomicU64::new(0));
    let read_idx = Arc::new(AtomicU64::new(0));
    let del_idx = Arc::new(AtomicU64::new(0));

    // ---- START BENCH ----
    let start = Instant::now();
    let mut handles = vec![];

    // writers
    for id in 0..WRITERS {
        let db = Arc::clone(&db);
        let write_count = Arc::clone(&write_count);
        let write_lat = Arc::clone(&write_lat);
        let write_idx = Arc::clone(&write_idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(id as u64 * 9999 + 1);

            while start.elapsed() < TEST_DURATION {
                let key = format!("key_w{}_{}", id, rng.random::<u64>());
                let val = format!("value_{}", rng.random::<u64>());

                let t0 = Instant::now();
                db.put(key.as_bytes(), val.as_bytes()).unwrap();
                let nanos = t0.elapsed().as_nanos() as u64;

                record_latency(&write_lat, &write_idx, nanos);
                write_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // readers
    for id in 0..READERS {
        let db = Arc::clone(&db);
        let read_count = Arc::clone(&read_count);
        let read_lat = Arc::clone(&read_lat);
        let read_idx = Arc::clone(&read_idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(id as u64 * 5555 + 2);

            while start.elapsed() < TEST_DURATION {
                let key = format!("key_w{}_{}", rng.random_range(0..WRITERS), rng.random::<u64>());

                let t0 = Instant::now();
                let _ = db.get(key.as_bytes());
                let nanos = t0.elapsed().as_nanos() as u64;

                record_latency(&read_lat, &read_idx, nanos);
                read_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // deleters
    for id in 0..DELETERS {
        let db = Arc::clone(&db);
        let del_count = Arc::clone(&del_count);
        let del_lat = Arc::clone(&del_lat);
        let del_idx = Arc::clone(&del_idx);

        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(id as u64 * 1234 + 3);

            while start.elapsed() < TEST_DURATION {
                let key = format!("key_w{}_{}", rng.random_range(0..WRITERS), rng.random::<u64>());

                let t0 = Instant::now();
                let _ = db.del(key.as_bytes());
                let nanos = t0.elapsed().as_nanos() as u64;

                record_latency(&del_lat, &del_idx, nanos);
                del_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // wait for threads
    for h in handles {
        h.join().unwrap();
    }

    let wc = write_count.load(Ordering::Relaxed);
    let rc = read_count.load(Ordering::Relaxed);
    let dc = del_count.load(Ordering::Relaxed);

    // ---- MEMORY AFTER TEST ----
    let mem_after_test = get_memory_usage_kb();
    let mem_used_total = mem_after_test - mem_before_open;

    // ---- RESULTS ----
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

    let (w50, w90, w99) = compute_percentiles(&write_lat, wc);
    let (r50, r90, r99) = compute_percentiles(&read_lat, rc);
    let (d50, d90, d99) = compute_percentiles(&del_lat, dc);

    println!("\n--- Write Latency (ns) ---");
    println!("p50: {} ns,  p90: {} ns,  p99: {} ns", w50, w90, w99);

    println!("\n--- Read Latency (ns) ---");
    println!("p50: {} ns,  p90: {} ns,  p99: {} ns", r50, r90, r99);

    println!("\n--- Delete Latency (ns) ---");
    println!("p50: {} ns,  p90: {} ns,  p99: {} ns", d50, d90, d99);

    println!("\n--- MEMORY USAGE ---");
    println!("Memory before DB open:  {} KB", mem_before_open);
    println!("Memory after DB open:   {} KB", mem_after_open);
    println!("Memory after benchmark: {} KB", mem_after_test);
    println!("Total DB memory used:   {} KB", mem_used_total);

    println!("\n=== Test Complete ===");
}
