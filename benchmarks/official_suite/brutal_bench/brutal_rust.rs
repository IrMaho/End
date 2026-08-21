use std::time::Instant;

// ============================================================================
// 1. Raymarcher
// ============================================================================
#[derive(Clone, Copy)]
struct Vec3 { x: f64, y: f64, z: f64 }
fn v3(x: f64, y: f64, z: f64) -> Vec3 { Vec3 { x, y, z } }
fn v_add(a: Vec3, b: Vec3) -> Vec3 { Vec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z } }
fn v_sub(a: Vec3, b: Vec3) -> Vec3 { Vec3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z } }
fn v_mul(a: Vec3, s: f64) -> Vec3 { Vec3 { x: a.x * s, y: a.y * s, z: a.z * s } }
fn v_dot(a: Vec3, b: Vec3) -> f64 { a.x * b.x + a.y * b.y + a.z * b.z }
fn v_len(a: Vec3) -> f64 { v_dot(a, a).sqrt() }
fn v_norm(a: Vec3) -> Vec3 {
    let l = v_len(a);
    if l > 1e-9 { v_mul(a, 1.0 / l) } else { a }
}

fn sdf_sphere(p: Vec3, center: Vec3, r: f64) -> f64 {
    v_len(v_sub(p, center)) - r
}
fn sdf_torus(p: Vec3, tx: f64, ty: f64) -> f64 {
    let qx = (p.x * p.x + p.z * p.z).sqrt() - tx;
    (qx * qx + p.y * p.y).sqrt() - ty
}
fn sdf_scene(p: Vec3) -> f64 {
    let d1 = sdf_sphere(p, v3(0.0, 0.0, 3.0), 0.8);
    let d2 = sdf_torus(v_sub(p, v3(0.0, -0.2, 3.0)), 1.2, 0.25);
    let d3 = p.y + 1.2;
    let d = if d1 < d2 { d1 } else { d2 };
    if d < d3 { d } else { d3 }
}
fn calc_normal(p: Vec3) -> Vec3 {
    let eps = 0.001;
    let d = sdf_scene(p);
    let n = v3(
        sdf_scene(v3(p.x + eps, p.y, p.z)) - d,
        sdf_scene(v3(p.x, p.y + eps, p.z)) - d,
        sdf_scene(v3(p.x, p.y, p.z + eps)) - d,
    );
    v_norm(n)
}

fn bench_raymarch(width: usize, height: usize) -> u64 {
    let ro = v3(0.0, 0.5, -1.5);
    let light_pos = v3(2.0, 4.0, -1.0);
    let mut total_lum: u64 = 0;

    for y in 0..height {
        let ny = (y as f64 / height as f64) * 2.0 - 1.0;
        for x in 0..width {
            let nx = (x as f64 / width as f64) * 2.0 - 1.0;
            let rd = v_norm(v3(nx * 1.2, -ny, 1.5));

            let mut t = 0.0;
            let mut hit = 0.0;
            for _ in 0..64 {
                let p = v_add(ro, v_mul(rd, t));
                let d = sdf_scene(p);
                if d < 0.001 {
                    let n = calc_normal(p);
                    let ld = v_norm(v_sub(light_pos, p));
                    let mut diff = v_dot(n, ld);
                    if diff < 0.0 { diff = 0.0; }
                    hit = diff * 255.0;
                    break;
                }
                t += d;
                if t > 20.0 { break; }
            }
            total_lum += hit as u64;
        }
    }
    total_lum
}

// ============================================================================
// 2. Binary Trees
// ============================================================================
struct TreeNode {
    item: i32,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

fn create_tree(item: i32, depth: i32) -> Box<TreeNode> {
    if depth > 0 {
        Box::new(TreeNode {
            item,
            left: Some(create_tree(2 * item - 1, depth - 1)),
            right: Some(create_tree(2 * item, depth - 1)),
        })
    } else {
        Box::new(TreeNode { item, left: None, right: None })
    }
}

fn check_tree(node: &TreeNode) -> i64 {
    let mut sum = node.item as i64;
    if let (Some(l), Some(r)) = (&node.left, &node.right) {
        sum += check_tree(l) - check_tree(r);
    }
    sum
}

fn bench_binary_trees(max_depth: i32) -> i64 {
    let min_depth = 4;
    let mut grand_sum: i64 = 0;

    let stretch = create_tree(0, max_depth + 1);
    grand_sum += check_tree(&stretch);
    drop(stretch);

    let long_lived = create_tree(0, max_depth);

    let mut depth = min_depth;
    while depth <= max_depth {
        let iterations = 1 << (max_depth - depth + min_depth);
        let mut check = 0;
        for i in 1..=iterations {
            let t1 = create_tree(i, depth);
            check += check_tree(&t1);
            drop(t1);

            let t2 = create_tree(-i, depth);
            check += check_tree(&t2);
            drop(t2);
        }
        grand_sum += check;
        depth += 2;
    }

    grand_sum += check_tree(&long_lived);
    grand_sum
}

// ============================================================================
// 3. HFT Engine
// ============================================================================
const MAX_LEVELS: usize = 100;
struct HftResult {
    total_trades: i64,
    total_volume: i64,
    bid_depth: i64,
    ask_depth: i64,
}

fn bench_hft_engine(num_orders: i32) -> HftResult {
    let mut bids = [0i32; MAX_LEVELS];
    let mut asks = [0i32; MAX_LEVELS];

    let mut total_trades: i64 = 0;
    let mut total_volume: i64 = 0;
    let mut rng: u64 = 0x123456789abcdef;

    for _ in 0..num_orders {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;

        let is_buy = (rng & 1) == 1;
        let price = 20 + ((rng >> 1) % 60) as usize;
        let mut qty = 1 + ((rng >> 8) % 100) as i32;
        let is_cancel = (rng >> 16) % 10 == 0;

        if is_cancel {
            if is_buy && bids[price] > 0 {
                bids[price] = if bids[price] > qty { bids[price] - qty } else { 0 };
            } else if !is_buy && asks[price] > 0 {
                asks[price] = if asks[price] > qty { asks[price] - qty } else { 0 };
            }
            continue;
        }

        if is_buy {
            for p in 0..=price {
                if qty <= 0 { break; }
                if asks[p] > 0 {
                    let trade_qty = if qty < asks[p] { qty } else { asks[p] };
                    asks[p] -= trade_qty;
                    qty -= trade_qty;
                    total_trades += 1;
                    total_volume += (trade_qty as i64) * (p as i64);
                }
            }
            if qty > 0 {
                bids[price] += qty;
            }
        } else {
            for p in (price..MAX_LEVELS).rev() {
                if qty <= 0 { break; }
                if bids[p] > 0 {
                    let trade_qty = if qty < bids[p] { qty } else { bids[p] };
                    bids[p] -= trade_qty;
                    qty -= trade_qty;
                    total_trades += 1;
                    total_volume += (trade_qty as i64) * (p as i64);
                }
            }
            if qty > 0 {
                asks[price] += qty;
            }
        }
    }

    let bid_depth: i64 = bids.iter().map(|&x| x as i64).sum();
    let ask_depth: i64 = asks.iter().map(|&x| x as i64).sum();

    HftResult { total_trades, total_volume, bid_depth, ask_depth }
}

fn main() {
    println!("=== BRUTAL BENCHMARK: Rust (1.89.0 -O -C target-cpu=native) ===");

    let t0 = Instant::now();
    let res1 = bench_raymarch(500, 500);
    let t1 = Instant::now();
    println!("1. Raymarcher 3D (250K rays): {:.2} ms | Checksum: {}", t1.duration_since(t0).as_secs_f64() * 1000.0, res1);

    let t0 = Instant::now();
    let res2 = bench_binary_trees(16);
    let t1 = Instant::now();
    println!("2. Binary Trees (Depth 16):   {:.2} ms | Checksum: {}", t1.duration_since(t0).as_secs_f64() * 1000.0, res2);

    let t0 = Instant::now();
    let res3 = bench_hft_engine(1000000);
    let t1 = Instant::now();
    println!("3. HFT Order Matching (1M):   {:.2} ms | Trades: {} | Vol: {}", 
             t1.duration_since(t0).as_secs_f64() * 1000.0, res3.total_trades, res3.total_volume);
}
