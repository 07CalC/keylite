use keylite_kv::core::Db;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;

const NUM_RECORDS: usize = 100_000;

fn main() {
    println!("=== KeyLite Database Population Script ===\n");

    let db_path = "populate_db";
    let _ = std::fs::remove_dir_all(db_path);

    let start = Instant::now();
    let db = Db::open(db_path).unwrap();
    println!("✓ Database opened at: {}", db_path);

    let mut rng = StdRng::seed_from_u64(12345);

    let first_names = vec![
        "Alice", "Bob", "Charlie", "Diana", "Edward", "Fiona", "George", "Hannah", "Ivan", "Julia",
        "Kevin", "Laura", "Michael", "Nancy", "Oliver", "Patricia", "Quinn", "Rachel", "Samuel",
        "Teresa",
    ];

    let last_names = vec![
        "Smith",
        "Johnson",
        "Williams",
        "Brown",
        "Jones",
        "Garcia",
        "Miller",
        "Davis",
        "Rodriguez",
        "Martinez",
        "Hernandez",
        "Lopez",
        "Gonzalez",
        "Wilson",
        "Anderson",
        "Thomas",
        "Taylor",
        "Moore",
        "Jackson",
        "Martin",
    ];

    let cities = vec![
        "New York",
        "Los Angeles",
        "Chicago",
        "Houston",
        "Phoenix",
        "Philadelphia",
        "San Antonio",
        "San Diego",
        "Dallas",
        "San Jose",
        "Austin",
        "Jacksonville",
        "Fort Worth",
        "Columbus",
        "Charlotte",
        "San Francisco",
        "Indianapolis",
        "Seattle",
        "Denver",
        "Washington",
    ];

    let departments = vec![
        "Engineering",
        "Sales",
        "Marketing",
        "HR",
        "Finance",
        "Operations",
        "Customer Support",
        "Product",
        "Design",
        "Legal",
    ];

    println!("\n📊 Populating database with {} records...", NUM_RECORDS);
    let populate_start = Instant::now();

    let checkpoint = NUM_RECORDS / 10;

    for i in 0..NUM_RECORDS {
        let user_id = format!("user:{:08}", i);
        let first_name = first_names[(rng.random::<u32>() as usize) % first_names.len()];
        let last_name = last_names[(rng.random::<u32>() as usize) % last_names.len()];
        let age = 18 + (rng.random::<u8>() % 60);
        let city = cities[(rng.random::<u32>() as usize) % cities.len()];
        let department = departments[(rng.random::<u32>() as usize) % departments.len()];
        let salary = 30000 + (rng.random::<u32>() % 170000);
        let email = format!(
            "{}.{}{}@company.com",
            first_name.to_lowercase(),
            last_name.to_lowercase(),
            rng.random::<u16>() % 1000
        );

        let user_data = format!(
            r#"{{"id":"{}","first_name":"{}","last_name":"{}","age":{},"city":"{}","department":"{}","salary":{},"email":"{}","active":{}}}"#,
            user_id,
            first_name,
            last_name,
            age,
            city,
            department,
            salary,
            email,
            rng.random::<bool>()
        );

        db.put(user_id.as_bytes(), user_data.as_bytes()).unwrap();

        let email_key = format!("idx:email:{}", email);
        db.put(email_key.as_bytes(), user_id.as_bytes()).unwrap();

        let dept_key = format!("idx:department:{}:{}", department, user_id);
        db.put(dept_key.as_bytes(), b"1").unwrap();

        let city_key = format!("idx:city:{}:{}", city, user_id);
        db.put(city_key.as_bytes(), b"1").unwrap();

        let age_range = (age / 10) * 10; // Group by decade: 20s, 30s, 40s, etc.
        let age_key = format!("idx:age_range:{}:{}", age_range, user_id);
        db.put(age_key.as_bytes(), b"1").unwrap();

        // Progress indicator
        if (i + 1) % checkpoint == 0 {
            let progress = ((i + 1) as f64 / NUM_RECORDS as f64) * 100.0;
            let elapsed = populate_start.elapsed();
            let rate = (i + 1) as f64 / elapsed.as_secs_f64();
            println!(
                "  [{:3.0}%] {} records inserted ({:.0} records/sec)",
                progress,
                i + 1,
                rate
            );
        }
    }

    let populate_time = populate_start.elapsed();

    println!("\n✓ Database populated successfully!");
    println!("\n📈 Statistics:");
    println!("  Total records:     {}", NUM_RECORDS);
    println!("  Total keys:        {} (with indexes)", NUM_RECORDS * 5); // 1 main + 4 indexes per record
    println!("  Population time:   {:?}", populate_time);
    println!(
        "  Insertion rate:    {:.2} records/sec",
        NUM_RECORDS as f64 / populate_time.as_secs_f64()
    );
    println!("  Total time:        {:?}", start.elapsed());

    // Verify some data
    println!("\n🔍 Sample Data Verification:");
    for i in 0..3 {
        let key = format!("user:{:08}", i);
        if let Ok(Some(val)) = db.get(key.as_bytes()) {
            let data = String::from_utf8_lossy(&val);
            println!("  {} -> {}", key, data);
        }
    }

    // Test scanning
    println!("\n📚 Testing Range Scan:");
    let scan_start = Instant::now();
    let prefix_start = b"user:00000".to_vec();
    let mut prefix_end = prefix_start.clone();
    prefix_end.push(0xFF);

    let mut count = 0;
    let iter = db.scan(Some(&prefix_start), Some(&prefix_end));
    for _ in iter {
        count += 1;
    }

    println!(
        "  Scanned {} records with prefix 'user:00000' in {:?}",
        count,
        scan_start.elapsed()
    );

    // Test index lookup
    println!("\n🔎 Testing Index Lookups:");
    let lookup_start = Instant::now();
    let test_dept = "Engineering";
    let dept_prefix = format!("idx:department:{}", test_dept);
    let dept_start = dept_prefix.as_bytes().to_vec();
    let mut dept_end = dept_start.clone();
    dept_end.push(0xFF);

    let mut dept_count = 0;
    let iter = db.scan(Some(&dept_start), Some(&dept_end));
    for _ in iter {
        dept_count += 1;
    }

    println!(
        "  Found {} users in '{}' department in {:?}",
        dept_count,
        test_dept,
        lookup_start.elapsed()
    );

    println!("\n🎉 Population script completed!");
    println!("Database location: {}", db_path);
}
