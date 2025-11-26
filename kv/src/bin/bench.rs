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
const OPS_PER_THREAD: usize = 5_00_000;

const TOTAL_OPS: usize = THREADS * OPS_PER_THREAD;

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

fn format_throughput(ops: usize, seconds: f64) -> String {
    let ops_per_sec = ops as f64 / seconds;
    if ops_per_sec >= 1_000_000.0 {
        format!("{:.2} M ops/sec", ops_per_sec / 1_000_000.0)
    } else if ops_per_sec >= 1_000.0 {
        format!("{:.2} K ops/sec", ops_per_sec / 1_000.0)
    } else {
        format!("{:.2} ops/sec", ops_per_sec)
    }
}

fn format_latency(nanos: u64) -> String {
    if nanos >= 1_000_000_000 {
        format!("{:.2} s", nanos as f64 / 1_000_000_000.0)
    } else if nanos >= 1_000_000 {
        format!("{:.2} ms", nanos as f64 / 1_000_000.0)
    } else if nanos >= 1_000 {
        format!("{:.2} µs", nanos as f64 / 1_000.0)
    } else {
        format!("{} ns", nanos)
    }
}

// ----- MAIN -----
fn main() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          KeyLite KV Store Benchmark Suite                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    println!("Configuration:");
    println!("  Threads:         {}", THREADS);
    println!("  Ops per thread:  {}", OPS_PER_THREAD);
    println!("  Total ops:       {} (per operation type)\n", TOTAL_OPS);

    let _ = std::fs::remove_dir_all("bench_db");
    let db = Arc::new(Db::open("bench_db").unwrap());

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

                let t0 = Instant::now();
                db.put(key.as_bytes(), val.as_bytes()).unwrap();
                record_latency(&write_lat, &write_idx, t0.elapsed().as_nanos() as u64);

                let t0 = Instant::now();
                let _ = db.get(key.as_bytes());
                record_latency(&read_lat, &read_idx, t0.elapsed().as_nanos() as u64);

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

    let seconds = total_time.as_secs_f64();

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                    BENCHMARK RESULTS                     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    println!("Total execution time: {:.2} seconds\n", seconds);

    // ===== THROUGHPUT =====
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│                       THROUGHPUT                        │");
    println!("├─────────────────────────────────────────────────────────┤");

    let write_throughput = format_throughput(writes_done, seconds);
    let read_throughput = format_throughput(reads_done, seconds);
    let delete_throughput = format_throughput(dels_done, seconds);

    println!(
        "│  Write:   {:>8} ops  │  {:>20} │",
        writes_done, write_throughput
    );
    println!(
        "│  Read:    {:>8} ops  │  {:>20} │",
        reads_done, read_throughput
    );
    println!(
        "│  Delete:  {:>8} ops  │  {:>20} │",
        dels_done, delete_throughput
    );
    println!("└─────────────────────────────────────────────────────────┘\n");

    // ===== LATENCY PERCENTILES =====
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│                   LATENCY PERCENTILES                   │");
    println!("├──────────┬──────────────┬──────────────┬────────────────┤");
    println!("│ Operation│     p50      │     p90      │      p99       │");
    println!("├──────────┼──────────────┼──────────────┼────────────────┤");
    println!(
        "│  Write   │ {:>12} │ {:>12} │ {:>13} │",
        format_latency(w50),
        format_latency(w90),
        format_latency(w99)
    );
    println!(
        "│  Read    │ {:>12} │ {:>12} │ {:>13} │",
        format_latency(r50),
        format_latency(r90),
        format_latency(r99)
    );
    println!(
        "│  Delete  │ {:>12} │ {:>12} │ {:>13} │",
        format_latency(d50),
        format_latency(d90),
        format_latency(d99)
    );
    println!("└──────────┴──────────────┴──────────────┴───────────────┘\n");

    let total_ops = writes_done + reads_done + dels_done;
    let overall_throughput = format_throughput(total_ops, seconds);

    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│                      SUMMARY                            │");
    println!("├─────────────────────────────────────────────────────────┤");
    println!(
        "│  Total operations:    {:>8}                         │",
        total_ops
    );
    println!(
        "│  Overall throughput:  {:>20}                 │",
        overall_throughput
    );
    println!(
        "│  Average op latency:  {:>12}                     │",
        format_latency((total_time.as_nanos() as u64) / (total_ops as u64))
    );
    println!("└─────────────────────────────────────────────────────────┘\n");

    println!("✓ Benchmark complete!\n");
}
