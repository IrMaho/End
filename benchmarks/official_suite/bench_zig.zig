const std = @import("std");

const c = @cImport({
    @cInclude("windows.h");
});

fn getTimeSec() f64 {
    var freq: c.LARGE_INTEGER = undefined;
    var count: c.LARGE_INTEGER = undefined;
    _ = c.QueryPerformanceFrequency(&freq);
    _ = c.QueryPerformanceCounter(&count);
    return @as(f64, @floatFromInt(count.QuadPart)) / @as(f64, @floatFromInt(freq.QuadPart));
}

fn bench_compute(iterations: u64) u64 {
    var state: u64 = 0x853c49e6748fea9b;
    var i: u64 = 0;
    while (i < iterations) : (i += 1) {
        state ^= (state << 13);
        state ^= (state >> 7);
        state ^= (state << 17);
        state = state *% 6364136223846793005 +% 1442695040888963407;
    }
    return state;
}

fn bench_memory(allocator: std.mem.Allocator, batches: usize, elements_per_batch: usize) !i64 {
    var grand_total: i64 = 0;
    var b: usize = 0;
    while (b < batches) : (b += 1) {
        const arr = try allocator.alloc(i64, elements_per_batch);
        defer allocator.free(arr);
        var batch_sum: i64 = 0;
        var i: usize = 0;
        while (i < elements_per_batch) : (i += 1) {
            const val = @as(i64, @intCast(b)) * 31 + @as(i64, @intCast(i)) * 17;
            arr[i] = val;
            batch_sum += val;
        }
        grand_total += (batch_sum ^ arr[0]);
    }
    return grand_total;
}

fn fib(n: i64) i64 {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

pub fn main() !void {
    std.debug.print("=== Zig Benchmark (-O ReleaseFast) ===\n", .{});

    const t0 = getTimeSec();
    const res1 = bench_compute(100000000);
    const t1 = getTimeSec();
    std.debug.print("1. Compute (100M iter): {d:.2} ms (Hash: {d})\n", .{ (t1 - t0) * 1000.0, res1 });

    const gpa = std.heap.c_allocator;
    const t2 = getTimeSec();
    const res2 = try bench_memory(gpa, 5000, 20000);
    const t3 = getTimeSec();
    std.debug.print("2. Memory Churn (100M items): {d:.2} ms (Sum: {d})\n", .{ (t3 - t2) * 1000.0, res2 });

    const t4 = getTimeSec();
    const res3 = fib(42);
    const t5 = getTimeSec();
    std.debug.print("3. Recursion (fib 42): {d:.2} ms (Val: {d})\n", .{ (t5 - t4) * 1000.0, res3 });
}
