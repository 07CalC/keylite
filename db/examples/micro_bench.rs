use std::time::Instant;

// Old format! based version
#[inline(never)]
fn doc_key_old(collection: &str, id: &str) -> Vec<u8> {
    format!("col:{collection}:doc:{id}").into_bytes()
}

// New optimized version
#[inline(never)]
fn doc_key_new(collection: &str, id: &str) -> Vec<u8> {
    let mut key = Vec::with_capacity(4 + collection.len() + 5 + id.len());
    key.extend_from_slice(b"col:");
    key.extend_from_slice(collection.as_bytes());
    key.extend_from_slice(b":doc:");
    key.extend_from_slice(id.as_bytes());
    key
}

// Alternative: pre-allocate and use write!
#[inline(never)]
fn doc_key_write(collection: &str, id: &str) -> Vec<u8> {
    use std::fmt::Write;
    let mut s = String::with_capacity(4 + collection.len() + 5 + id.len());
    let _ = write!(&mut s, "col:{collection}:doc:{id}");
    s.into_bytes()
}

fn main() {
    let n = 100_000;
    let collection = "users";
    let id = "550e8400-e29b-41d4-a716-446655440000"; // typical UUID
    
    println!("=== Key Generation Micro-Benchmark ===");
    println!("Iterations: {}", n);
    println!("Collection: {}", collection);
    println!("ID: {}\n", id);
    
    // Warm up
    for _ in 0..1000 {
        let _ = doc_key_old(collection, id);
        let _ = doc_key_new(collection, id);
        let _ = doc_key_write(collection, id);
    }
    
    // Benchmark format! (old)
    let start = Instant::now();
    for _ in 0..n {
        let _ = doc_key_old(collection, id);
    }
    let old_time = start.elapsed();
    
    // Benchmark extend_from_slice (new)
    let start = Instant::now();
    for _ in 0..n {
        let _ = doc_key_new(collection, id);
    }
    let new_time = start.elapsed();
    
    // Benchmark write! macro
    let start = Instant::now();
    for _ in 0..n {
        let _ = doc_key_write(collection, id);
    }
    let write_time = start.elapsed();
    
    println!("format! (old):          {:?} ({:.2} ns/iter)", old_time, old_time.as_nanos() as f64 / n as f64);
    println!("extend_from_slice (new):{:?} ({:.2} ns/iter)", new_time, new_time.as_nanos() as f64 / n as f64);
    println!("write! macro:           {:?} ({:.2} ns/iter)", write_time, write_time.as_nanos() as f64 / n as f64);
    
    println!("\nSpeedup: {:.2}x faster", old_time.as_secs_f64() / new_time.as_secs_f64());
    
    // Verify correctness
    let old_result = doc_key_old(collection, id);
    let new_result = doc_key_new(collection, id);
    let write_result = doc_key_write(collection, id);
    
    assert_eq!(old_result, new_result);
    assert_eq!(old_result, write_result);
    println!("\n✓ All methods produce identical output");
}
