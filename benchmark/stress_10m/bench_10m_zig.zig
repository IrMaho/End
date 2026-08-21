// Ultra-Heavy 10M Benchmark: Zig (ReleaseFast)
const std = @import("std");

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
    const iterations: usize = 10000000;
    var total_checksum: i64 = 0;

    std.debug.print("Running Zig 10,000,000 Heavy Backend Requests Benchmark...\n", .{});

    var i: u64 = 0;
    while (i < iterations) : (i += 1) {
        const req = process_request(i, 256);
        total_checksum +%= req.checksum;
    }

    std.debug.print("Zig 10M Benchmark Finished. Total Checksum:\n{d}\n", .{total_checksum});
}
