use std::time::Instant;
use std::env;

#[inline(always)]
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

// 1. Raymarcher
#[derive(Clone, Copy)]
struct Vec3 { x: f32, y: f32, z: f32 }
impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    fn add(self, b: Vec3) -> Vec3 { Vec3::new(self.x + b.x, self.y + b.y, self.z + b.z) }
    fn sub(self, b: Vec3) -> Vec3 { Vec3::new(self.x - b.x, self.y - b.y, self.z - b.z) }
    fn scale(self, s: f32) -> Vec3 { Vec3::new(self.x * s, self.y * s, self.z * s) }
    fn dot(self, b: Vec3) -> f32 { self.x * b.x + self.y * b.y + self.z * b.z }
    fn length(self) -> f32 { self.dot(self).sqrt() }
    fn norm(self) -> Vec3 { let l = self.length(); if l > 0.00001 { self.scale(1.0 / l) } else { self } }
}

fn sdf_sphere(p: Vec3, r: f32) -> f32 { p.length() - r }
fn sdf_torus(p: Vec3, r1: f32, r2: f32) -> f32 {
    let qx = (p.x * p.x + p.z * p.z).sqrt() - r1;
    (qx * qx + p.y * p.y).sqrt() - r2
}
fn sdf_scene(p: Vec3) -> f32 {
    let d_sphere = sdf_sphere(p.sub(Vec3::new(0.0, 1.0, 0.0)), 1.0);
    let d_torus = sdf_torus(p.sub(Vec3::new(0.0, 0.5, 0.0)), 1.2, 0.3);
    let d_floor = p.y;
    let d = if d_sphere < d_torus { d_sphere } else { d_torus };
    if d < d_floor { d } else { d_floor }
}
fn calc_normal(p: Vec3) -> Vec3 {
    let eps = 0.001;
    Vec3::new(
        sdf_scene(Vec3::new(p.x + eps, p.y, p.z)) - sdf_scene(Vec3::new(p.x - eps, p.y, p.z)),
        sdf_scene(Vec3::new(p.x, p.y + eps, p.z)) - sdf_scene(Vec3::new(p.x, p.y - eps, p.z)),
        sdf_scene(Vec3::new(p.x, p.y, p.z + eps)) - sdf_scene(Vec3::new(p.x, p.y, p.z - eps)),
    ).norm()
}

fn bench_1_raymarcher() -> i64 {
    let w = 500;
    let h = 500;
    let mut checksum: i64 = 0;
    let ro = Vec3::new(0.0, 1.5, -3.5);
    let light_pos = Vec3::new(2.0, 4.0, -2.0);

    for y in 0..h {
        for x in 0..w {
            let u = (2.0 * (x as f32) - (w as f32)) / (h as f32);
            let v = -(2.0 * (y as f32) - (h as f32)) / (h as f32);
            let rd = Vec3::new(u, v, 1.5).norm();
            let mut t = 0.0;
            let mut hit = false;
            for _ in 0..64 {
                let p = ro.add(rd.scale(t));
                let d = sdf_scene(p);
                if d < 0.001 {
                    let n = calc_normal(p);
                    let l = light_pos.sub(p).norm();
                    let diff = n.dot(l).max(0.0);
                    let color = (diff * 255.0) as i64;
                    checksum += color;
                    hit = true;
                    break;
                }
                t += d;
                if t > 20.0 { break; }
            }
            if !hit { checksum += 10; }
        }
    }
    checksum
}

// 2. Binary Trees
struct TreeNode {
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    val: i32,
}

fn create_tree(depth: i32) -> Box<TreeNode> {
    if depth > 0 {
        Box::new(TreeNode {
            left: Some(create_tree(depth - 1)),
            right: Some(create_tree(depth - 1)),
            val: depth,
        })
    } else {
        Box::new(TreeNode { left: None, right: None, val: depth })
    }
}
fn sum_tree(node: &Option<Box<TreeNode>>) -> i64 {
    match node {
        Some(n) => n.val as i64 + sum_tree(&n.left) - sum_tree(&n.right),
        None => 0,
    }
}

fn bench_2_binary_trees() -> i64 {
    let max_depth = 16;
    let stretch = Some(create_tree(max_depth + 1));
    let mut check = sum_tree(&stretch);
    drop(stretch);

    let long_lived = Some(create_tree(max_depth));
    let mut depth = 4;
    while depth <= max_depth {
        let iterations = 1 << (max_depth - depth + 4);
        for _ in 1..=iterations {
            let t1 = Some(create_tree(depth));
            check += sum_tree(&t1);
        }
        depth += 2;
    }
    check += sum_tree(&long_lived);
    check
}

// 3. HFT Engine
fn bench_3_hft_engine() -> i64 {
    let mut rng: u64 = 0x123456789ABCDEF0;
    let mut total_volume: i64 = 0;
    let mut buy_depth = [0i32; 100];
    let mut sell_depth = [0i32; 100];

    for _ in 0..1000000 {
        let r = splitmix64(&mut rng);
        let side = (r >> 63) & 1;
        let price = (r % 100) as usize;
        let mut qty = (((r >> 8) % 50) + 1) as i32;
        let op = (r >> 16) % 10;

        if op == 0 {
            if side == 0 { buy_depth[price] = 0; } else { sell_depth[price] = 0; }
        } else if side == 0 {
            let mut p = price as isize;
            while p >= 0 && qty > 0 {
                let up = p as usize;
                if sell_depth[up] > 0 {
                    let fill = qty.min(sell_depth[up]);
                    sell_depth[up] -= fill;
                    qty -= fill;
                    total_volume += (fill as i64) * (p as i64 + 1);
                }
                p -= 1;
            }
            if qty > 0 { buy_depth[price] += qty; }
        } else {
            let mut p = price;
            while p < 100 && qty > 0 {
                if buy_depth[p] > 0 {
                    let fill = qty.min(buy_depth[p]);
                    buy_depth[p] -= fill;
                    qty -= fill;
                    total_volume += (fill as i64) * (p as i64 + 1);
                }
                p += 1;
            }
            if qty > 0 { sell_depth[price] += qty; }
        }
    }
    total_volume
}

// 4. SHA-256
#[inline(always)]
fn rotr(x: u32, n: u32) -> u32 { (x >> n) | (x << (32 - n)) }
const K: [u32; 64] = [
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0x0bef9a3f,0xc67178f2
];

fn bench_4_sha256() -> i64 {
    let mut state = [
        0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    ];
    let mut w = [0u32; 64];
    let mut prng: u64 = 0xCAFEBABE12345678;

    for _ in 0..500000 {
        for i in 0..16 { w[i] = splitmix64(&mut prng) as u32; }
        for i in 16..64 {
            let s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            let s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut a = state[0]; let mut b = state[1]; let mut c = state[2]; let mut d = state[3];
        let mut e = state[4]; let mut f = state[5]; let mut g = state[6]; let mut h = state[7];

        for i in 0..64 {
            let s1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g; g = f; f = e; e = d.wrapping_add(temp1);
            d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
        }
        state[0] = state[0].wrapping_add(a); state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c); state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e); state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g); state[7] = state[7].wrapping_add(h);
    }
    let res = ((state[0] as u64) << 32) | (state[7] as u64);
    res as i64
}

// 5. N-Body
fn bench_5_nbody() -> i64 {
    let n = 1000;
    let steps = 1000;
    let mut pos_x = vec![0.0f32; 1000];
    let mut pos_y = vec![0.0f32; 1000];
    let mut pos_z = vec![0.0f32; 1000];
    let mut vel_x = vec![0.0f32; 1000];
    let mut vel_y = vec![0.0f32; 1000];
    let mut vel_z = vec![0.0f32; 1000];
    let mut mass = vec![0.0f32; 1000];
    let mut prng: u64 = 0x5555AAAA5555AAAA;

    for i in 0..n {
        pos_x[i] = (((splitmix64(&mut prng) % 1000) as f32) / 100.0) - 5.0;
        pos_y[i] = (((splitmix64(&mut prng) % 1000) as f32) / 100.0) - 5.0;
        pos_z[i] = (((splitmix64(&mut prng) % 1000) as f32) / 100.0) - 5.0;
        mass[i] = 1.0 + (((splitmix64(&mut prng) % 100) as f32) / 10.0);
    }

    let dt = 0.01f32;
    let eps2 = 0.001f32;

    for _ in 0..steps {
        for i in 0..n {
            let mut fx = 0.0f32; let mut fy = 0.0f32; let mut fz = 0.0f32;
            for j in 0..n {
                if i == j { continue; }
                let dx = pos_x[j] - pos_x[i];
                let dy = pos_y[j] - pos_y[i];
                let dz = pos_z[j] - pos_z[i];
                let dist_sq = dx * dx + dy * dy + dz * dz + eps2;
                let dist_inv = 1.0 / dist_sq.sqrt();
                let f = mass[j] * (dist_inv * dist_inv * dist_inv);
                fx += dx * f; fy += dy * f; fz += dz * f;
            }
            vel_x[i] += fx * dt; vel_y[i] += fy * dt; vel_z[i] += fz * dt;
        }
        for i in 0..n {
            pos_x[i] += vel_x[i] * dt; pos_y[i] += vel_y[i] * dt; pos_z[i] += vel_z[i] * dt;
        }
    }

    let mut total_ke = 0.0f64;
    for i in 0..n {
        total_ke += 0.5 * (mass[i] as f64) * ((vel_x[i] * vel_x[i] + vel_y[i] * vel_y[i] + vel_z[i] * vel_z[i]) as f64);
    }
    (total_ke * 1000.0) as i64
}

// 6. Ring Buffer
fn bench_6_ring_buffer() -> i64 {
    const CAPACITY: usize = 65536;
    const MASK: usize = CAPACITY - 1;
    const TOTAL_MSGS: usize = 10000000;
    let mut ring = vec![0i64; CAPACITY];
    let mut total_sum: i64 = 0;

    let mut head = 0;
    let mut tail = 0;
    let mut chunk = 0;
    while chunk < TOTAL_MSGS {
        for k in 0..64 {
            ring[(tail + k) & MASK] = ((chunk + k) as i64).wrapping_mul(31).wrapping_add(17);
        }
        tail += 64;
        for k in 0..64 {
            total_sum = total_sum.wrapping_add(ring[(head + k) & MASK]);
        }
        head += 64;
        chunk += 64;
    }
    total_sum
}

// 7. DNA Levenshtein
fn bench_7_dna_alignment() -> i64 {
    const N: usize = 1000;
    let mut dp = vec![0i32; 1001];
    let mut prng: u64 = 0x9999888877776666;
    let mut s1 = vec![0u8; 1000];
    let mut s2 = vec![0u8; 1000];
    let bases = b"ACGT";

    let mut total_distance: i64 = 0;
    for _ in 0..1000 {
        for i in 0..N {
            s1[i] = bases[(splitmix64(&mut prng) % 4) as usize];
            s2[i] = bases[(splitmix64(&mut prng) % 4) as usize];
        }
        for j in 0..=N { dp[j] = j as i32; }

        for i in 1..=N {
            let mut prev = dp[0];
            dp[0] = i as i32;
            for j in 1..=N {
                let temp = dp[j];
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                let d1 = dp[j - 1] + 1;
                let d2 = dp[j] + 1;
                let d3 = prev + cost;
                let min_d = d1.min(d2).min(d3);
                dp[j] = min_d;
                prev = temp;
            }
        }
        total_distance += dp[N] as i64;
    }
    total_distance
}

// 8. JSON Microservice
fn bench_8_json_serializer() -> i64 {
    let mut hash: i64 = 0;
    for i in 0..100000 {
        let s = format!("{{\"id\":{},\"status\":\"active\",\"latency_us\":{},\"tags\":[\"prod\",\"edge\",\"v2\"],\"metrics\":{{\"cpu\":{:.1},\"mem\":{:.1}}}}}",
            i, (i * 37) % 500, 42.5 + (i % 10) as f32, 128.4 + (i % 50) as f32);
        let bytes = s.as_bytes();
        let len = bytes.len();
        hash = hash.wrapping_mul(31).wrapping_add(len as i64).wrapping_add(bytes[len / 2] as i64);
    }
    hash
}

// 9. FSM Lexer
fn bench_9_fsm_lexer() -> i64 {
    let sample = b"pub fn calculate_metrics(id: u64, active: bool) -> i64 { val base = id * 31; ret base + 10; } ";
    let sample_len = sample.len();
    let mut token_count: i64 = 0;
    let mut token_hash: i64 = 0;

    #[derive(PartialEq)]
    enum State { Start, Ident, Number, Op }
    let mut st = State::Start;

    for i in 0..10000000 {
        let c = sample[i % sample_len];
        match st {
            State::Start => {
                if (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' { st = State::Ident; }
                else if c >= b'0' && c <= b'9' { st = State::Number; }
                else if c != b' ' && c != b'\n' && c != b'\t' { st = State::Op; }
            },
            State::Ident => {
                if !((c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') || c == b'_') {
                    token_count += 1;
                    token_hash = token_hash.wrapping_mul(33).wrapping_add(1);
                    st = State::Start;
                }
            },
            State::Number => {
                if !(c >= b'0' && c <= b'9') {
                    token_count += 1;
                    token_hash = token_hash.wrapping_mul(33).wrapping_add(2);
                    st = State::Start;
                }
            },
            State::Op => {
                token_count += 1;
                token_hash = token_hash.wrapping_mul(33).wrapping_add(3);
                st = State::Start;
            },
        }
    }
    token_hash + token_count
}

// 10. GEMM Matrix
fn bench_10_gemm_matrix() -> i64 {
    let n = 512;
    let mut a = vec![0.0f64; n * n];
    let mut b = vec![0.0f64; n * n];
    let mut c = vec![0.0f64; n * n];

    for idx in 0..n * n {
        a[idx] = ((idx % 100) as f64) * 0.01;
        b[idx] = (((idx * 3) % 100) as f64) * 0.01;
    }

    let block = 32;
    let mut sj = 0;
    while sj < n {
        let mut si = 0;
        while si < n {
            let mut sk = 0;
            while sk < n {
                for i in si..si + block {
                    for k in sk..sk + block {
                        let a_ik = a[i * n + k];
                        for j in sj..sj + block {
                            c[i * n + j] += a_ik * b[k * n + j];
                        }
                    }
                }
                sk += block;
            }
            si += block;
        }
        sj += block;
    }

    let mut trace = 0.0f64;
    for i in 0..n { trace += c[i * n + i]; }
    (trace * 100.0) as i64
}

// 11. Monte Carlo Black-Scholes
fn bench_11_monte_carlo() -> i64 {
    let paths = 2000000;
    let s0 = 100.0f64; let k = 100.0f64; let t = 1.0f64; let r = 0.05f64; let sigma = 0.20f64;
    let drift = (r - 0.5 * sigma * sigma) * t;
    let vol = sigma * t.sqrt();
    let discount = (-r * t).exp();

    let mut prng: u64 = 0xFEEDFACECAFE1234;
    let mut total_payoff = 0.0f64;

    let mut i = 0;
    while i < paths {
        let u1 = (((splitmix64(&mut prng) >> 11) + 1) as f64) / 9007199254740992.0;
        let u2 = (((splitmix64(&mut prng) >> 11) + 1) as f64) / 9007199254740992.0;
        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        let z1 = radius * theta.cos();
        let z2 = radius * theta.sin();

        let s_t1 = s0 * (drift + vol * z1).exp();
        let s_t2 = s0 * (drift + vol * z2).exp();

        let payoff1 = if s_t1 > k { s_t1 - k } else { 0.0 };
        let payoff2 = if s_t2 > k { s_t2 - k } else { 0.0 };

        total_payoff += payoff1 + payoff2;
        i += 2;
    }
    let option_price = (total_payoff / (paths as f64)) * discount;
    (option_price * 1000000.0) as i64
}

// 12. Super-Scalar Reduction
struct Req12 { id: u64, payload_size: i32, checksum: i64 }
#[inline(always)]
fn process_req12(id: u64, size: i32) -> Req12 {
    let mut hash: u64 = 17;
    for j in 0..50 {
        hash = hash.wrapping_mul(31).wrapping_add(id).wrapping_add(j);
    }
    Req12 { id, payload_size: size, checksum: hash as i64 }
}

fn bench_12_reduction() -> i64 {
    let iterations: usize = 10000000;
    let mut sum0: i64 = 0; let mut sum1: i64 = 0; let mut sum2: i64 = 0; let mut sum3: i64 = 0;

    let mut i = 0;
    while i < iterations {
        sum0 = sum0.wrapping_add(process_req12(i as u64, 256).checksum);
        sum1 = sum1.wrapping_add(process_req12((i + 1) as u64, 256).checksum);
        sum2 = sum2.wrapping_add(process_req12((i + 2) as u64, 256).checksum);
        sum3 = sum3.wrapping_add(process_req12((i + 3) as u64, 256).checksum);
        i += 4;
    }
    sum0.wrapping_add(sum1).wrapping_add(sum2).wrapping_add(sum3)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: suite12_rust.exe <id (1..12)>");
        return;
    }
    let id: u32 = args[1].parse().unwrap_or(1);
    let t0 = Instant::now();
    let check: i64 = match id {
        1 => bench_1_raymarcher(),
        2 => bench_2_binary_trees(),
        3 => bench_3_hft_engine(),
        4 => bench_4_sha256(),
        5 => bench_5_nbody(),
        6 => bench_6_ring_buffer(),
        7 => bench_7_dna_alignment(),
        8 => bench_8_json_serializer(),
        9 => bench_9_fsm_lexer(),
        10 => bench_10_gemm_matrix(),
        11 => bench_11_monte_carlo(),
        12 => bench_12_reduction(),
        _ => return,
    };
    let elapsed = t0.elapsed();
    let ms = (elapsed.as_nanos() as f64) / 1000000.0;
    println!("RESULT:bench={},time_ms={:.3},checksum={}", id, ms, check);
}
