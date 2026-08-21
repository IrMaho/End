use std::time::Instant;

#[inline(never)]
fn bench_compute(iterations: u64) -> u64 {
    let mut state: u64 = 0x853c49e6748fea9b;
    for _ in 0..iterations {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    }
    state
}

#[inline(never)]
fn bench_memory(batches: i32, elements_per_batch: usize) -> i64 {
    let mut grand_total: i64 = 0;
    for b in 0..batches {
        let mut arr: Vec<i64> = Vec::with_capacity(elements_per_batch);
        let mut batch_sum: i64 = 0;
        for i in 0..elements_per_batch {
            let val = (b as i64) * 31 + (i as i64) * 17;
            arr.push(val);
            batch_sum += val;
        }
        grand_total += batch_sum ^ arr[0];
    }
    grand_total
}

#[inline(never)]
fn fib(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}

fn main() {
    println!("=== Rust Benchmark (rustc -O -C target-cpu=native) ===");

    let t0 = Instant::now();
    let res1 = bench_compute(100_000_000);
    let t1 = Instant::now();
    println!("1. Compute (100M iter): {:.2} ms (Hash: {})", t1.duration_since(t0).as_secs_f64() * 1000.0, res1);

    let t0 = Instant::now();
    let res2 = bench_memory(5000, 20000);
    let t1 = Instant::now();
    println!("2. Memory Churn (100M items): {:.2} ms (Sum: {})", t1.duration_since(t0).as_secs_f64() * 1000.0, res2);

    let t0 = Instant::now();
    let res3 = fib(42);
    let t1 = Instant::now();
    println!("3. Recursion (fib 42): {:.2} ms (Val: {})", t1.duration_since(t0).as_secs_f64() * 1000.0, res3);
}
