// Benchmark 3: Zig (Arena Allocator)
const std = @import("std");

const Request = struct {
    id: u64,
    payload_size: i32,
    checksum: i64,
};

fn process_request(id: u64, size: i32) Request {
    var hash: i64 = 17;
    var j: i64 = 0;
    while (j < 50) : (j += 1) {
        hash = (hash *% 31) +% @as(i64, @intCast(id)) +% j;
    }
    return Request{
        .id = id,
        .payload_size = size,
        .checksum = hash,
    };
}

pub fn main() void {
    const iterations: usize = 1000000;
    var total_checksum: i64 = 0;

    std.debug.print("Running Zig Backend Benchmark (1,000,000 requests)...\n", .{});

    var i: usize = 0;
    while (i < iterations) : (i += 1) {
        const req = process_request(i, 256);
        total_checksum +%= req.checksum;
    }

    std.debug.print("Zig Benchmark Finished. Total Checksum:\n{d}\n", .{total_checksum});
}
