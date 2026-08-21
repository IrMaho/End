const std = @import("std");

extern "c" fn QueryPerformanceFrequency(lpFrequency: *i64) callconv(.c) c_int;
extern "c" fn QueryPerformanceCounter(lpPerformanceCount: *i64) callconv(.c) c_int;

fn get_time_ns() u64 {
    var freq: i64 = 0;
    var counter: i64 = 0;
    _ = QueryPerformanceFrequency(&freq);
    _ = QueryPerformanceCounter(&counter);
    const num = @as(u128, @intCast(counter)) * 1000000000;
    const den = @as(u128, @intCast(freq));
    return @as(u64, @intCast(num / den));
}

fn splitmix64(state: *u64) u64 {
    state.* +%= 0x9E3779B97F4A7C15;
    var z = state.*;
    z = (z ^ (z >> 30)) *% 0xBF58476D1CE4E5B9;
    z = (z ^ (z >> 27)) *% 0x94D049BB133111EB;
    return z ^ (z >> 31);
}

// 1. Raymarcher
const Vec3 = struct {
    x: f32, y: f32, z: f32,
    fn add(a: Vec3, b: Vec3) Vec3 { return .{ .x = a.x + b.x, .y = a.y + b.y, .z = a.z + b.z }; }
    fn sub(a: Vec3, b: Vec3) Vec3 { return .{ .x = a.x - b.x, .y = a.y - b.y, .z = a.z - b.z }; }
    fn scale(a: Vec3, s: f32) Vec3 { return .{ .x = a.x * s, .y = a.y * s, .z = a.z * s }; }
    fn dot(a: Vec3, b: Vec3) f32 { return a.x * b.x + a.y * b.y + a.z * b.z; }
    fn length(a: Vec3) f32 { return @sqrt(a.dot(a)); }
    fn norm(a: Vec3) Vec3 { const l = a.length(); return if (l > 0.00001) a.scale(1.0 / l) else a; }
};

fn sdf_sphere(p: Vec3, r: f32) f32 { return p.length() - r; }
fn sdf_torus(p: Vec3, r1: f32, r2: f32) f32 {
    const qx = @sqrt(p.x * p.x + p.z * p.z) - r1;
    return @sqrt(qx * qx + p.y * p.y) - r2;
}
fn sdf_scene(p: Vec3) f32 {
    const d_sphere = sdf_sphere(p.sub(.{ .x = 0, .y = 1, .z = 0 }), 1.0);
    const d_torus = sdf_torus(p.sub(.{ .x = 0, .y = 0.5, .z = 0 }), 1.2, 0.3);
    const d_floor = p.y;
    const d = if (d_sphere < d_torus) d_sphere else d_torus;
    return if (d < d_floor) d else d_floor;
}
fn calc_normal(p: Vec3) Vec3 {
    const eps: f32 = 0.001;
    return Vec3.norm(.{
        .x = sdf_scene(.{ .x = p.x + eps, .y = p.y, .z = p.z }) - sdf_scene(.{ .x = p.x - eps, .y = p.y, .z = p.z }),
        .y = sdf_scene(.{ .x = p.x, .y = p.y + eps, .z = p.z }) - sdf_scene(.{ .x = p.x, .y = p.y - eps, .z = p.z }),
        .z = sdf_scene(.{ .x = p.x, .y = p.y, .z = p.z + eps }) - sdf_scene(.{ .x = p.x, .y = p.y, .z = p.z - eps }),
    });
}

fn bench_1_raymarcher() i64 {
    const W: usize = 500;
    const H: usize = 500;
    var checksum: i64 = 0;
    const ro = Vec3{ .x = 0, .y = 1.5, .z = -3.5 };
    const light_pos = Vec3{ .x = 2, .y = 4, .z = -2 };

    var y: usize = 0;
    while (y < H) : (y += 1) {
        var x: usize = 0;
        while (x < W) : (x += 1) {
            const u = (2.0 * @as(f32, @floatFromInt(x)) - @as(f32, @floatFromInt(W))) / @as(f32, @floatFromInt(H));
            const v = -(2.0 * @as(f32, @floatFromInt(y)) - @as(f32, @floatFromInt(H))) / @as(f32, @floatFromInt(H));
            const rd = Vec3.norm(.{ .x = u, .y = v, .z = 1.5 });
            var t: f32 = 0.0;
            var hit = false;
            var step: usize = 0;
            while (step < 64) : (step += 1) {
                const p = ro.add(rd.scale(t));
                const d = sdf_scene(p);
                if (d < 0.001) {
                    const n = calc_normal(p);
                    const l = Vec3.norm(light_pos.sub(p));
                    var diff = n.dot(l);
                    if (diff < 0.0) diff = 0.0;
                    const color: i64 = @intFromFloat(diff * 255.0);
                    checksum += color;
                    hit = true;
                    break;
                }
                t += d;
                if (t > 20.0) break;
            }
            if (!hit) checksum += 10;
        }
    }
    return checksum;
}

// 2. Binary Trees
const TreeNode = struct {
    left: ?*TreeNode,
    right: ?*TreeNode,
    val: i32,
};
var arena_allocator = std.heap.ArenaAllocator.init(std.heap.page_allocator);

fn create_tree(alloc: std.mem.Allocator, depth: i32) *TreeNode {
    const node = alloc.create(TreeNode) catch unreachable;
    node.val = depth;
    if (depth > 0) {
        node.left = create_tree(alloc, depth - 1);
        node.right = create_tree(alloc, depth - 1);
    } else {
        node.left = null;
        node.right = null;
    }
    return node;
}
fn sum_tree(node: ?*TreeNode) i64 {
    if (node) |n| {
        return @as(i64, n.val) + sum_tree(n.left) - sum_tree(n.right);
    }
    return 0;
}

fn bench_2_binary_trees() i64 {
    const alloc = arena_allocator.allocator();
    const max_depth: i32 = 16;
    const stretch = create_tree(alloc, max_depth + 1);
    var check = sum_tree(stretch);

    const long_lived = create_tree(alloc, max_depth);
    var depth: i32 = 4;
    while (depth <= max_depth) : (depth += 2) {
        const iterations = @as(usize, 1) << @as(u6, @intCast(max_depth - depth + 4));
        var i: usize = 1;
        while (i <= iterations) : (i += 1) {
            const t1 = create_tree(alloc, depth);
            check += sum_tree(t1);
        }
    }
    check += sum_tree(long_lived);
    return check;
}

// 3. HFT Engine
fn bench_3_hft_engine() i64 {
    var rng: u64 = 0x123456789ABCDEF0;
    var total_volume: i64 = 0;
    var buy_depth = [_]i32{0} ** 100;
    var sell_depth = [_]i32{0} ** 100;

    var i: usize = 0;
    while (i < 1000000) : (i += 1) {
        const r = splitmix64(&rng);
        const side = (r >> 63) & 1;
        const price = @as(usize, @intCast(r % 100));
        var qty = @as(i32, @intCast(((r >> 8) % 50) + 1));
        const op = (r >> 16) % 10;

        if (op == 0) {
            if (side == 0) buy_depth[price] = 0 else sell_depth[price] = 0;
        } else if (side == 0) {
            var p: isize = @as(isize, @intCast(price));
            while (p >= 0 and qty > 0) : (p -= 1) {
                const up = @as(usize, @intCast(p));
                if (sell_depth[up] > 0) {
                    const fill = if (qty < sell_depth[up]) qty else sell_depth[up];
                    sell_depth[up] -= fill;
                    qty -= fill;
                    total_volume += @as(i64, fill) * @as(i64, @intCast(p + 1));
                }
            }
            if (qty > 0) buy_depth[price] += qty;
        } else {
            var p: usize = price;
            while (p < 100 and qty > 0) : (p += 1) {
                if (buy_depth[p] > 0) {
                    const fill = if (qty < buy_depth[p]) qty else buy_depth[p];
                    buy_depth[p] -= fill;
                    qty -= fill;
                    total_volume += @as(i64, fill) * @as(i64, @intCast(p + 1));
                }
            }
            if (qty > 0) sell_depth[price] += qty;
        }
    }
    return total_volume;
}

// 4. SHA-256
fn rotr32(x: u32, comptime n: u6) u32 { return std.math.rotr(u32, x, n); }
const K256 = [_]u32{
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0x0bef9a3f,0xc67178f2
};

fn bench_4_sha256() i64 {
    var state = [_]u32{
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    };
    var w: [64]u32 = undefined;
    var prng: u64 = 0xCAFEBABE12345678;

    var block: usize = 0;
    while (block < 500000) : (block += 1) {
        var i: usize = 0;
        while (i < 16) : (i += 1) { w[i] = @as(u32, @truncate(splitmix64(&prng))); }
        while (i < 64) : (i += 1) {
            const s0 = rotr32(w[i - 15], 7) ^ rotr32(w[i - 15], 18) ^ (w[i - 15] >> 3);
            const s1 = rotr32(w[i - 2], 17) ^ rotr32(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] +% s0 +% w[i - 7] +% s1;
        }
        var a = state[0]; var b = state[1]; var c = state[2]; var d = state[3];
        var e = state[4]; var f = state[5]; var g = state[6]; var h = state[7];

        var r: usize = 0;
        while (r < 64) : (r += 1) {
            const S1 = rotr32(e, 6) ^ rotr32(e, 11) ^ rotr32(e, 25);
            const ch = (e & f) ^ ((~e) & g);
            const temp1 = h +% S1 +% ch +% K256[r] +% w[r];
            const S0 = rotr32(a, 2) ^ rotr32(a, 13) ^ rotr32(a, 22);
            const maj = (a & b) ^ (a & c) ^ (b & c);
            const temp2 = S0 +% maj;

            h = g; g = f; f = e; e = d +% temp1;
            d = c; c = b; b = a; a = temp1 +% temp2;
        }
        state[0] +%= a; state[1] +%= b; state[2] +%= c; state[3] +%= d;
        state[4] +%= e; state[5] +%= f; state[6] +%= g; state[7] +%= h;
    }
    const res: u64 = (@as(u64, state[0]) << 32) | @as(u64, state[7]);
    return @as(i64, @bitCast(res));
}

// 5. N-Body
fn bench_5_nbody() i64 {
    const N: usize = 1000;
    const STEPS: usize = 1000;
    var pos_x: [1000]f32 = undefined;
    var pos_y: [1000]f32 = undefined;
    var pos_z: [1000]f32 = undefined;
    var vel_x = [_]f32{0.0} ** 1000;
    var vel_y = [_]f32{0.0} ** 1000;
    var vel_z = [_]f32{0.0} ** 1000;
    var mass: [1000]f32 = undefined;
    var prng: u64 = 0x5555AAAA5555AAAA;

    var i: usize = 0;
    while (i < N) : (i += 1) {
        pos_x[i] = (@as(f32, @floatFromInt(splitmix64(&prng) % 1000)) / 100.0) - 5.0;
        pos_y[i] = (@as(f32, @floatFromInt(splitmix64(&prng) % 1000)) / 100.0) - 5.0;
        pos_z[i] = (@as(f32, @floatFromInt(splitmix64(&prng) % 1000)) / 100.0) - 5.0;
        mass[i] = 1.0 + (@as(f32, @floatFromInt(splitmix64(&prng) % 100)) / 10.0);
    }

    const dt: f32 = 0.01;
    const eps2: f32 = 0.001;

    var step: usize = 0;
    while (step < STEPS) : (step += 1) {
        i = 0;
        while (i < N) : (i += 1) {
            var fx: f32 = 0.0; var fy: f32 = 0.0; var fz: f32 = 0.0;
            var j: usize = 0;
            while (j < N) : (j += 1) {
                if (i == j) continue;
                const dx = pos_x[j] - pos_x[i];
                const dy = pos_y[j] - pos_y[i];
                const dz = pos_z[j] - pos_z[i];
                const dist_sq = dx * dx + dy * dy + dz * dz + eps2;
                const dist_inv = 1.0 / @sqrt(dist_sq);
                const f = mass[j] * (dist_inv * dist_inv * dist_inv);
                fx += dx * f; fy += dy * f; fz += dz * f;
            }
            vel_x[i] += fx * dt; vel_y[i] += fy * dt; vel_z[i] += fz * dt;
        }
        i = 0;
        while (i < N) : (i += 1) {
            pos_x[i] += vel_x[i] * dt; pos_y[i] += vel_y[i] * dt; pos_z[i] += vel_z[i] * dt;
        }
    }

    var total_ke: f64 = 0.0;
    i = 0;
    while (i < N) : (i += 1) {
        total_ke += 0.5 * @as(f64, mass[i]) * @as(f64, vel_x[i] * vel_x[i] + vel_y[i] * vel_y[i] + vel_z[i] * vel_z[i]);
    }
    return @as(i64, @intFromFloat(total_ke * 1000.0));
}

// 6. Ring Buffer
fn bench_6_ring_buffer() i64 {
    const CAPACITY: usize = 65536;
    const MASK: usize = CAPACITY - 1;
    const TOTAL_MSGS: usize = 10000000;
    var ring: [CAPACITY]i64 = undefined;
    var total_sum: i64 = 0;

    var head: usize = 0;
    var tail: usize = 0;
    var chunk: usize = 0;
    while (chunk < TOTAL_MSGS) : (chunk += 64) {
        var k: usize = 0;
        while (k < 64) : (k += 1) {
            ring[(tail + k) & MASK] = @as(i64, @intCast(chunk + k)) *% 31 +% 17;
        }
        tail += 64;
        k = 0;
        while (k < 64) : (k += 1) {
            total_sum += ring[(head + k) & MASK];
        }
        head += 64;
    }
    return total_sum;
}

// 7. DNA Levenshtein
fn bench_7_dna_alignment() i64 {
    const N: usize = 1000;
    var dp: [1001]i32 = undefined;
    var prng: u64 = 0x9999888877776666;
    var s1: [1000]u8 = undefined;
    var s2: [1000]u8 = undefined;
    const bases = "ACGT";

    var total_distance: i64 = 0;
    var pair: usize = 0;
    while (pair < 1000) : (pair += 1) {
        var i: usize = 0;
        while (i < N) : (i += 1) {
            s1[i] = bases[@as(usize, @intCast(splitmix64(&prng) % 4))];
            s2[i] = bases[@as(usize, @intCast(splitmix64(&prng) % 4))];
        }
        var j: usize = 0;
        while (j <= N) : (j += 1) dp[j] = @as(i32, @intCast(j));

        i = 1;
        while (i <= N) : (i += 1) {
            var prev = dp[0];
            dp[0] = @as(i32, @intCast(i));
            j = 1;
            while (j <= N) : (j += 1) {
                const temp = dp[j];
                const cost: i32 = if (s1[i - 1] == s2[j - 1]) 0 else 1;
                const d1 = dp[j - 1] + 1;
                const d2 = dp[j] + 1;
                const d3 = prev + cost;
                var min_d = if (d1 < d2) d1 else d2;
                if (d3 < min_d) min_d = d3;
                dp[j] = min_d;
                prev = temp;
            }
        }
        total_distance += dp[N];
    }
    return total_distance;
}

// 8. JSON Microservice
fn bench_8_json_serializer() i64 {
    var buf: [512]u8 = undefined;
    var hash: i64 = 0;

    var i: usize = 0;
    while (i < 100000) : (i += 1) {
        const slice = std.fmt.bufPrint(&buf, "{{\"id\":{d},\"status\":\"active\",\"latency_us\":{d},\"tags\":[\"prod\",\"edge\",\"v2\"],\"metrics\":{{\"cpu\":{d:.1},\"mem\":{d:.1}}}}}", .{
            i, (i * 37) % 500, 42.5 + @as(f32, @floatFromInt(i % 10)), 128.4 + @as(f32, @floatFromInt(i % 50))
        }) catch unreachable;
        hash = (hash *% 31) +% @as(i64, @intCast(slice.len)) +% @as(i64, slice[slice.len / 2]);
    }
    return hash;
}

// 9. FSM Lexer
fn bench_9_fsm_lexer() i64 {
    const sample = "pub fn calculate_metrics(id: u64, active: bool) -> i64 { val base = id * 31; ret base + 10; } ";
    const sample_len = sample.len;
    var token_count: i64 = 0;
    var token_hash: i64 = 0;

    const State = enum { start, ident, number, op };
    var st = State.start;

    var i: usize = 0;
    while (i < 10000000) : (i += 1) {
        const c = sample[i % sample_len];
        switch (st) {
            .start => {
                if ((c >= 'a' and c <= 'z') or (c >= 'A' and c <= 'Z') or c == '_') st = .ident
                else if (c >= '0' and c <= '9') st = .number
                else if (c != ' ' and c != '\n' and c != '\t') st = .op;
            },
            .ident => {
                if (!((c >= 'a' and c <= 'z') or (c >= 'A' and c <= 'Z') or (c >= '0' and c <= '9') or c == '_')) {
                    token_count += 1;
                    token_hash = (token_hash *% 33) +% 1;
                    st = .start;
                }
            },
            .number => {
                if (!(c >= '0' and c <= '9')) {
                    token_count += 1;
                    token_hash = (token_hash *% 33) +% 2;
                    st = .start;
                }
            },
            .op => {
                token_count += 1;
                token_hash = (token_hash *% 33) +% 3;
                st = .start;
            },
        }
    }
    return token_hash + token_count;
}

// 10. GEMM Matrix
fn bench_10_gemm_matrix() i64 {
    const N: usize = 512;
    var A = std.heap.page_allocator.alloc(f64, N * N) catch unreachable;
    defer std.heap.page_allocator.free(A);
    var B = std.heap.page_allocator.alloc(f64, N * N) catch unreachable;
    defer std.heap.page_allocator.free(B);
    var C = std.heap.page_allocator.alloc(f64, N * N) catch unreachable;
    defer std.heap.page_allocator.free(C);
    @memset(C, 0.0);

    var idx: usize = 0;
    while (idx < N * N) : (idx += 1) {
        A[idx] = @as(f64, @floatFromInt(idx % 100)) * 0.01;
        B[idx] = @as(f64, @floatFromInt((idx * 3) % 100)) * 0.01;
    }

    const BLOCK: usize = 32;
    var sj: usize = 0;
    while (sj < N) : (sj += BLOCK) {
        var si: usize = 0;
        while (si < N) : (si += BLOCK) {
            var sk: usize = 0;
            while (sk < N) : (sk += BLOCK) {
                var i = si;
                while (i < si + BLOCK) : (i += 1) {
                    var k = sk;
                    while (k < sk + BLOCK) : (k += 1) {
                        const a_ik = A[i * N + k];
                        var j = sj;
                        while (j < sj + BLOCK) : (j += 1) {
                            C[i * N + j] += a_ik * B[k * N + j];
                        }
                    }
                }
            }
        }
    }

    var trace: f64 = 0.0;
    var i: usize = 0;
    while (i < N) : (i += 1) trace += C[i * N + i];
    return @as(i64, @intFromFloat(trace * 100.0));
}

// 11. Monte Carlo Black-Scholes
fn bench_11_monte_carlo() i64 {
    const PATHS: usize = 2000000;
    const S0: f64 = 100.0; const K: f64 = 100.0; const T: f64 = 1.0; const r: f64 = 0.05; const sigma: f64 = 0.20;
    const drift = (r - 0.5 * sigma * sigma) * T;
    const vol = sigma * @sqrt(T);
    const discount = @exp(-r * T);

    var prng: u64 = 0xFEEDFACECAFE1234;
    var total_payoff: f64 = 0.0;

    var i: usize = 0;
    while (i < PATHS) : (i += 2) {
        const rand_u1 = @as(f64, @floatFromInt((splitmix64(&prng) >> 11) + 1)) / 9007199254740992.0;
        const rand_u2 = @as(f64, @floatFromInt((splitmix64(&prng) >> 11) + 1)) / 9007199254740992.0;
        const radius = @sqrt(-2.0 * @log(rand_u1));
        const theta = 2.0 * 3.14159265358979323846 * rand_u2;
        const z1 = radius * @cos(theta);
        const z2 = radius * @sin(theta);

        const s_t1 = S0 * @exp(drift + vol * z1);
        const s_t2 = S0 * @exp(drift + vol * z2);

        const payoff1 = if (s_t1 > K) (s_t1 - K) else 0.0;
        const payoff2 = if (s_t2 > K) (s_t2 - K) else 0.0;

        total_payoff += (payoff1 + payoff2);
    }
    const option_price = (total_payoff / @as(f64, @floatFromInt(PATHS))) * discount;
    return @as(i64, @intFromFloat(option_price * 1000000.0));
}

// 12. Super-Scalar 10M Reduction
const Req12 = struct { id: u64, payload_size: i32, checksum: i64 };
inline fn process_req12(id: u64, size: i32) Req12 {
    var hash: u64 = id ^ 0x9E3779B97F4A7C15;
    var j: u64 = 0;
    while (j < 50) : (j += 1) {
        hash ^= hash << 13;
        hash ^= hash >> 7;
        hash ^= hash << 17;
        hash = hash +% j +% 0xBF58476D1CE4E5B9;
    }
    return .{ .id = id, .payload_size = size, .checksum = @as(i64, @bitCast(hash)) };
}

fn bench_12_reduction() i64 {
    const iterations: usize = 10000000;
    var total_checksum: i64 = 0;
    var sum0: i64 = 0; var sum1: i64 = 0; var sum2: i64 = 0; var sum3: i64 = 0;

    var i: usize = 0;
    while (i < iterations) : (i += 4) {
        sum0 +%= process_req12(i, 256).checksum;
        sum1 +%= process_req12(i + 1, 256).checksum;
        sum2 +%= process_req12(i + 2, 256).checksum;
        sum3 +%= process_req12(i + 3, 256).checksum;
    }
    total_checksum = sum0 +% sum1 +% sum2 +% sum3;
    return total_checksum;
}

pub export fn main(argc: c_int, argv: [*][*:0]const u8) c_int {
    if (argc < 2) {
        std.debug.print("Usage: suite12_zig.exe <id (1..12)>\n", .{});
        return 1;
    }
    const id_slice = std.mem.span(argv[1]);
    const id = std.fmt.parseInt(u32, id_slice, 10) catch 1;
    var check: i64 = 0;

    const t0 = get_time_ns();
    switch (id) {
        1 => check = bench_1_raymarcher(),
        2 => check = bench_2_binary_trees(),
        3 => check = bench_3_hft_engine(),
        4 => check = bench_4_sha256(),
        5 => check = bench_5_nbody(),
        6 => check = bench_6_ring_buffer(),
        7 => check = bench_7_dna_alignment(),
        8 => check = bench_8_json_serializer(),
        9 => check = bench_9_fsm_lexer(),
        10 => check = bench_10_gemm_matrix(),
        11 => check = bench_11_monte_carlo(),
        12 => check = bench_12_reduction(),
        else => return 1,
    }
    const t1 = get_time_ns();
    const ms = @as(f64, @floatFromInt(t1 - t0)) / 1000000.0;
    std.debug.print("RESULT:bench={d},time_ms={d:.3},checksum={d}\n", .{ id, ms, check });
    return 0;
}
