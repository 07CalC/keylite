use fastrand;
use keylite::db::Db;
use std::time::{Duration, Instant};

const N: usize = 5_000_000;

fn stats(times: &[Duration]) -> (Duration, Duration, Duration, Duration) {
    let mut sorted = times.to_vec();
    sorted.sort();
    let sum: Duration = sorted.iter().copied().sum();

    let avg = sum / (sorted.len() as u32);
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let median = sorted[sorted.len() / 2];

    (avg, min, max, median)
}

fn main() {
    println!("=== KeyLite Benchmark ===");
    println!("Ops: {}", N);

    // OPEN TIME ----------------------------------------------------
    let t = Instant::now();
    let db = Db::open("benchdb").unwrap();
    let open_time = t.elapsed();
    println!("open(): {:?}", open_time);

    // WRITE BENCH --------------------------------------------------
    let mut write_times = Vec::with_capacity(N);

    for i in 0..N {
        let k = format!("key{i}");
        let v = format!("value{i}");
        let t = Instant::now();
        db.put(k.as_bytes(), v.as_bytes()).unwrap();
        write_times.push(t.elapsed());
    }
    // Optional: flush for durability
    // db.flush().unwrap();

    let (w_avg, w_min, w_max, w_med) = stats(&write_times);
    println!(
        "\nwrite x {}:\n  avg: {:?}\n  med: {:?}\n  min: {:?}\n  max: {:?}\n  ops/sec: {:.2}",
        N,
        w_avg,
        w_med,
        w_min,
        w_max,
        N as f64 / write_times.iter().copied().sum::<Duration>().as_secs_f64(),
    );

    // SEQUENTIAL READ BENCH ----------------------------------------
    let mut read_times_seq = Vec::with_capacity(N);

    for i in 0..N {
        let k = format!("key{i}");
        let t = Instant::now();
        let _ = db.get(k.as_bytes()).unwrap();
        read_times_seq.push(t.elapsed());
    }

    let (r_avg, r_min, r_max, r_med) = stats(&read_times_seq);
    println!(
        "\nread(seq) x {}:\n  avg: {:?}\n  med: {:?}\n  min: {:?}\n  max: {:?}\n  ops/sec: {:.2}",
        N,
        r_avg,
        r_med,
        r_min,
        r_max,
        N as f64
            / read_times_seq
                .iter()
                .copied()
                .sum::<Duration>()
                .as_secs_f64(),
    );

    // RANDOM READ BENCH --------------------------------------------
    let mut read_times_rand = Vec::with_capacity(N);
    let mut rng = fastrand::Rng::new();

    for _ in 0..N {
        let i = rng.usize(..N);
        let k = format!("key{i}");
        let t = Instant::now();
        let _ = db.get(k.as_bytes()).unwrap();
        read_times_rand.push(t.elapsed());
    }

    let (rr_avg, rr_min, rr_max, rr_med) = stats(&read_times_rand);
    println!(
        "\nread(rand) x {}:\n  avg: {:?}\n  med: {:?}\n  min: {:?}\n  max: {:?}\n  ops/sec: {:.2}",
        N,
        rr_avg,
        rr_med,
        rr_min,
        rr_max,
        N as f64
            / read_times_rand
                .iter()
                .copied()
                .sum::<Duration>()
                .as_secs_f64(),
    );

    // SUMMARY -------------------------------------------------------
    println!("\n=== Summary ===");
    println!("open(): {:?}", open_time);
    println!(
        "write avg: {:?}    ops/sec: {:.2}",
        w_avg,
        N as f64 / write_times.iter().copied().sum::<Duration>().as_secs_f64()
    );
    println!(
        "seq read avg: {:?} ops/sec: {:.2}",
        r_avg,
        N as f64
            / read_times_seq
                .iter()
                .copied()
                .sum::<Duration>()
                .as_secs_f64()
    );
    println!(
        "rand read avg: {:?} ops/sec: {:.2}",
        rr_avg,
        N as f64
            / read_times_rand
                .iter()
                .copied()
                .sum::<Duration>()
                .as_secs_f64()
    );
}
