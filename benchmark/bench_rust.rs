// Benchmark 4: Rust
struct Request {
    id: u64,
    payload_size: i32,
    checksum: i64,
}

#[inline(always)]
fn process_request(id: u64, size: i32) -> Request {
    let mut hash: i64 = 17;
    for j in 0..50 {
        hash = hash.wrapping_mul(31).wrapping_add(id as i64).wrapping_add(j);
    }
    Request {
        id,
        payload_size: size,
        checksum: hash,
    }
}

fn main() {
    let iterations: usize = 1_000_000;
    let mut total_checksum: i64 = 0;

    println!("Running Rust Backend Benchmark (1,000,000 requests)...");

    for i in 0..iterations {
        let req = process_request(i as u64, 256);
        total_checksum = total_checksum.wrapping_add(req.checksum);
    }

    println!("Rust Benchmark Finished. Total Checksum:\n{}", total_checksum);
}
