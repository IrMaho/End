// Precise Zig 10M Benchmark via Windows High-Resolution QPC
const std = @import("std");

extern "kernel32" fn QueryPerformanceCounter(lpPerformanceCount: *i64) callconv(.c) i32;
extern "kernel32" fn QueryPerformanceFrequency(lpFrequency: *i64) callconv(.c) i32;

const Request = struct {
    id: u64,
    payload_size: i32,
    checksum: i64,
};

inline fn process_request(id: u64, size: i32) Request {
    var hash: i64 = 17;
    var j: i64 = 0;
    while (j < 32) : (j += 1) {
        hash = (hash *% 31) +% @as(i64, @intCast(id)) +% j;
    }

    return Request{
        .id = id,
        .payload_size = size,
        .checksum = hash,
    };
}

pub fn main() !void {
    var freq: i64 = 0;
    _ = QueryPerformanceFrequency(&freq);

    const iterations: usize = 10000000;
    var total_checksum: i64 = 0;

    // Warmup
    var dummy: i64 = 0;
    var w: u64 = 0;
    while (w < 10000) : (w += 1) {
        dummy +%= process_request(w, 256).checksum;
    }

    var start: i64 = 0;
    var end: i64 = 0;
    _ = QueryPerformanceCounter(&start);

    var i: u64 = 0;
    while (i < iterations) : (i += 1) {
        const req = process_request(i, 256);
        total_checksum +%= req.checksum;
    }

    _ = QueryPerformanceCounter(&end);

    const elapsed_ms = @as(f64, @floatFromInt(end - start)) * 1000.0 / @as(f64, @floatFromInt(freq));
    const throughput = (10_000_000.0 / (elapsed_ms / 1000.0)) / 1_000_000.0;

    std.debug.print("=== ZIG PRECISE 10M BENCHMARK ===\n", .{});
    std.debug.print("Total Checksum: {d} (dummy {d})\n", .{ total_checksum, dummy });
    std.debug.print("Execution Time: {d:.4} ms\n", .{elapsed_ms});
    std.debug.print("Throughput:     {d:.2} Million req/s\n", .{throughput});
}
