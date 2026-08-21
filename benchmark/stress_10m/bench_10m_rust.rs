// Ultra-Heavy 10M Benchmark: Rust (opt-level=3 + target-cpu=native)
#[repr(C)]
struct Request {
    id: u64,
    payload_size: i32,
    checksum: i64,
}

#[inline(always)]
fn process_request(id: u64, size: i32) -> Request {
    let mut hash: i64 = 17;
    for j in 0..32 {
        hash = hash.wrapping_mul(31).wrapping_add(id as i64).wrapping_add(j);
    }
    Request {
        id,
        payload_size: size,
        checksum: hash,
    }
}

fn main() {
    let iterations: u64 = 10_000_000;
    let mut total_checksum: i64 = 0;

    println!("Running Rust 10,000,000 Heavy Backend Requests Benchmark...");

    for i in 0..iterations {
        let req = process_request(i, 256);
        total_checksum = total_checksum.wrapping_add(req.checksum);
    }

    println!("Rust 10M Benchmark Finished. Total Checksum:\n{}", total_checksum);
}
