const std = @import("std");

const c = @cImport({
    @cInclude("windows.h");
    @cInclude("math.h");
});

fn getTimeMs() f64 {
    var freq: c.LARGE_INTEGER = undefined;
    var count: c.LARGE_INTEGER = undefined;
    _ = c.QueryPerformanceFrequency(&freq);
    _ = c.QueryPerformanceCounter(&count);
    return (@as(f64, @floatFromInt(count.QuadPart)) * 1000.0) / @as(f64, @floatFromInt(freq.QuadPart));
}

// 1. Raymarcher
const Vec3 = struct { x: f64, y: f64, z: f64 };
fn v3(x: f64, y: f64, z: f64) Vec3 { return .{ .x = x, .y = y, .z = z }; }
fn v_add(a: Vec3, b: Vec3) Vec3 { return .{ .x = a.x + b.x, .y = a.y + b.y, .z = a.z + b.z }; }
fn v_sub(a: Vec3, b: Vec3) Vec3 { return .{ .x = a.x - b.x, .y = a.y - b.y, .z = a.z - b.z }; }
fn v_mul(a: Vec3, s: f64) Vec3 { return .{ .x = a.x * s, .y = a.y * s, .z = a.z * s }; }
fn v_dot(a: Vec3, b: Vec3) f64 { return a.x * b.x + a.y * b.y + a.z * b.z; }
fn v_len(a: Vec3) f64 { return @sqrt(v_dot(a, a)); }
fn v_norm(a: Vec3) Vec3 {
    const l = v_len(a);
    return if (l > 1e-9) v_mul(a, 1.0 / l) else a;
}

fn sdf_sphere(p: Vec3, center: Vec3, r: f64) f64 {
    return v_len(v_sub(p, center)) - r;
}
fn sdf_torus(p: Vec3, tx: f64, ty: f64) f64 {
    const qx = @sqrt(p.x * p.x + p.z * p.z) - tx;
    return @sqrt(qx * qx + p.y * p.y) - ty;
}
fn sdf_scene(p: Vec3) f64 {
    const d1 = sdf_sphere(p, v3(0.0, 0.0, 3.0), 0.8);
    const d2 = sdf_torus(v_sub(p, v3(0.0, -0.2, 3.0)), 1.2, 0.25);
    const d3 = p.y + 1.2;
    const d = if (d1 < d2) d1 else d2;
    return if (d < d3) d else d3;
}
fn calc_normal(p: Vec3) Vec3 {
    const eps = 0.001;
    const d = sdf_scene(p);
    const n = v3(
        sdf_scene(v3(p.x + eps, p.y, p.z)) - d,
        sdf_scene(v3(p.x, p.y + eps, p.z)) - d,
        sdf_scene(v3(p.x, p.y, p.z + eps)) - d,
    );
    return v_norm(n);
}

fn bench_raymarch(width: i32, height: i32) u64 {
    const ro = v3(0.0, 0.5, -1.5);
    const light_pos = v3(2.0, 4.0, -1.0);
    var total_lum: u64 = 0;

    var y: i32 = 0;
    while (y < height) : (y += 1) {
        const ny = (@as(f64, @floatFromInt(y)) / @as(f64, @floatFromInt(height))) * 2.0 - 1.0;
        var x: i32 = 0;
        while (x < width) : (x += 1) {
            const nx = (@as(f64, @floatFromInt(x)) / @as(f64, @floatFromInt(width))) * 2.0 - 1.0;
            const rd = v_norm(v3(nx * 1.2, -ny, 1.5));

            var t: f64 = 0.0;
            var hit: f64 = 0.0;
            var step: i32 = 0;
            while (step < 64) : (step += 1) {
                const p = v_add(ro, v_mul(rd, t));
                const d = sdf_scene(p);
                if (d < 0.001) {
                    const n = calc_normal(p);
                    const ld = v_norm(v_sub(light_pos, p));
                    var diff = v_dot(n, ld);
                    if (diff < 0.0) diff = 0.0;
                    hit = diff * 255.0;
                    break;
                }
                t += d;
                if (t > 20.0) break;
            }
            total_lum += @as(u64, @intFromFloat(hit));
        }
    }
    return total_lum;
}

// 2. Binary Trees
const TreeNode = struct {
    item: i32,
    left: ?*TreeNode,
    right: ?*TreeNode,
};

fn create_tree(allocator: std.mem.Allocator, item: i32, depth: i32) !*TreeNode {
    const n = try allocator.create(TreeNode);
    n.item = item;
    if (depth > 0) {
        n.left = try create_tree(allocator, 2 * item - 1, depth - 1);
        n.right = try create_tree(allocator, 2 * item, depth - 1);
    } else {
        n.left = null;
        n.right = null;
    }
    return n;
}

fn check_tree(n: ?*TreeNode) i64 {
    if (n) |node| {
        var sum: i64 = node.item;
        if (node.left) |_| {
            sum += check_tree(node.left) - check_tree(node.right);
        }
        return sum;
    }
    return 0;
}

fn free_tree(allocator: std.mem.Allocator, n: ?*TreeNode) void {
    if (n) |node| {
        if (node.left) |_| free_tree(allocator, node.left);
        if (node.right) |_| free_tree(allocator, node.right);
        allocator.destroy(node);
    }
}

fn bench_binary_trees(allocator: std.mem.Allocator, max_depth: i32) !i64 {
    const min_depth: i32 = 4;
    var grand_sum: i64 = 0;

    const stretch = try create_tree(allocator, 0, max_depth + 1);
    grand_sum += check_tree(stretch);
    free_tree(allocator, stretch);

    const long_lived = try create_tree(allocator, 0, max_depth);

    var depth = min_depth;
    while (depth <= max_depth) : (depth += 2) {
        const iterations = @as(i32, 1) << @as(u5, @intCast(max_depth - depth + min_depth));
        var check: i64 = 0;
        var i: i32 = 1;
        while (i <= iterations) : (i += 1) {
            const t1 = try create_tree(allocator, i, depth);
            check += check_tree(t1);
            free_tree(allocator, t1);

            const t2 = try create_tree(allocator, -i, depth);
            check += check_tree(t2);
            free_tree(allocator, t2);
        }
        grand_sum += check;
    }

    grand_sum += check_tree(long_lived);
    free_tree(allocator, long_lived);
    return grand_sum;
}

// 3. HFT Engine
const MAX_LEVELS = 100;
const HftResult = struct {
    total_trades: i64,
    total_volume: i64,
    bid_depth: i64,
    ask_depth: i64,
};

fn bench_hft_engine(num_orders: i32) HftResult {
    var bids = [_]i32{0} ** MAX_LEVELS;
    var asks = [_]i32{0} ** MAX_LEVELS;

    var total_trades: i64 = 0;
    var total_volume: i64 = 0;
    var rng: u64 = 0x123456789abcdef;

    var i: i32 = 0;
    while (i < num_orders) : (i += 1) {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;

        const is_buy = (rng & 1) == 1;
        const price = 20 + @as(usize, @intCast((rng >> 1) % 60));
        var qty = 1 + @as(i32, @intCast((rng >> 8) % 100));
        const is_cancel = ((rng >> 16) % 10 == 0);

        if (is_cancel) {
            if (is_buy and bids[price] > 0) {
                bids[price] = if (bids[price] > qty) bids[price] - qty else 0;
            } else if (!is_buy and asks[price] > 0) {
                asks[price] = if (asks[price] > qty) asks[price] - qty else 0;
            }
            continue;
        }

        if (is_buy) {
            var p: usize = 0;
            while (p <= price and qty > 0) : (p += 1) {
                if (asks[p] > 0) {
                    const trade_qty = if (qty < asks[p]) qty else asks[p];
                    asks[p] -= trade_qty;
                    qty -= trade_qty;
                    total_trades += 1;
                    total_volume += @as(i64, trade_qty) * @as(i64, @intCast(p));
                }
            }
            if (qty > 0) {
                bids[price] += qty;
            }
        } else {
            var p: isize = MAX_LEVELS - 1;
            while (p >= @as(isize, @intCast(price)) and qty > 0) : (p -= 1) {
                const up = @as(usize, @intCast(p));
                if (bids[up] > 0) {
                    const trade_qty = if (qty < bids[up]) qty else bids[up];
                    bids[up] -= trade_qty;
                    qty -= trade_qty;
                    total_trades += 1;
                    total_volume += @as(i64, trade_qty) * @as(i64, p);
                }
            }
            if (qty > 0) {
                asks[price] += qty;
            }
        }
    }

    var bid_depth: i64 = 0;
    var ask_depth: i64 = 0;
    for (bids) |b| bid_depth += b;
    for (asks) |a| ask_depth += a;

    return .{
        .total_trades = total_trades,
        .total_volume = total_volume,
        .bid_depth = bid_depth,
        .ask_depth = ask_depth,
    };
}

pub fn main() !void {
    std.debug.print("=== BRUTAL BENCHMARK: Zig (-O ReleaseFast) ===\n", .{});

    var t0 = getTimeMs();
    const res1 = bench_raymarch(500, 500);
    var t1 = getTimeMs();
    std.debug.print("1. Raymarcher 3D (250K rays): {d:.2} ms | Checksum: {d}\n", .{ t1 - t0, res1 });

    const gpa = std.heap.c_allocator;
    t0 = getTimeMs();
    const res2 = try bench_binary_trees(gpa, 16);
    t1 = getTimeMs();
    std.debug.print("2. Binary Trees (Depth 16):   {d:.2} ms | Checksum: {d}\n", .{ t1 - t0, res2 });

    t0 = getTimeMs();
    const res3 = bench_hft_engine(1000000);
    t1 = getTimeMs();
    std.debug.print("3. HFT Order Matching (1M):   {d:.2} ms | Trades: {d} | Vol: {d}\n", .{
        t1 - t0, res3.total_trades, res3.total_volume
    });
}
