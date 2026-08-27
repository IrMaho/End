#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <math.h>
#include <time.h>

static double get_time_ms(void) {
#if defined(CLOCK_MONOTONIC)
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ((double)ts.tv_sec * 1000.0) + ((double)ts.tv_nsec / 1e6);
#else
    return ((double)clock() / (double)CLOCKS_PER_SEC) * 1000.0;
#endif
}

// ============================================================================
// 🥊 CHALLENGE 1: 3D RAYMARCHER & FLOATING-POINT VECTOR PHYSICS (500x500 = 250K Rays)
// ============================================================================
typedef struct { double x, y, z; } Vec3;
static inline Vec3 v3(double x, double y, double z) { return (Vec3){x, y, z}; }
static inline Vec3 v_add(Vec3 a, Vec3 b) { return (Vec3){a.x + b.x, a.y + b.y, a.z + b.z}; }
static inline Vec3 v_sub(Vec3 a, Vec3 b) { return (Vec3){a.x - b.x, a.y - b.y, a.z - b.z}; }
static inline Vec3 v_mul(Vec3 a, double s) { return (Vec3){a.x * s, a.y * s, a.z * s}; }
static inline double v_dot(Vec3 a, Vec3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
static inline double v_len(Vec3 a) { return sqrt(v_dot(a, a)); }
static inline Vec3 v_norm(Vec3 a) {
    double l = v_len(a);
    return l > 1e-9 ? v_mul(a, 1.0 / l) : a;
}

static inline double sdf_sphere(Vec3 p, Vec3 c, double r) {
    return v_len(v_sub(p, c)) - r;
}
static inline double sdf_torus(Vec3 p, double tx, double ty) {
    double qx = sqrt(p.x * p.x + p.z * p.z) - tx;
    return sqrt(qx * qx + p.y * p.y) - ty;
}
static inline double sdf_scene(Vec3 p) {
    double d1 = sdf_sphere(p, v3(0.0, 0.0, 3.0), 0.8);
    double d2 = sdf_torus(v_sub(p, v3(0.0, -0.2, 3.0)), 1.2, 0.25);
    double d3 = p.y + 1.2; // Floor plane
    double d = d1 < d2 ? d1 : d2;
    return d < d3 ? d : d3;
}
static inline Vec3 calc_normal(Vec3 p) {
    double eps = 0.001;
    double d = sdf_scene(p);
    Vec3 n = v3(
        sdf_scene(v3(p.x + eps, p.y, p.z)) - d,
        sdf_scene(v3(p.x, p.y + eps, p.z)) - d,
        sdf_scene(v3(p.x, p.y, p.z + eps)) - d
    );
    return v_norm(n);
}

static uint64_t bench_raymarch(int width, int height) {
    Vec3 ro = v3(0.0, 0.5, -1.5);
    Vec3 light_pos = v3(2.0, 4.0, -1.0);
    uint64_t total_lum = 0;

    for (int y = 0; y < height; y++) {
        double ny = ((double)y / (double)height) * 2.0 - 1.0;
        for (int x = 0; x < width; x++) {
            double nx = ((double)x / (double)width) * 2.0 - 1.0;
            Vec3 rd = v_norm(v3(nx * 1.2, -ny, 1.5));

            double t = 0.0;
            double hit = 0.0;
            for (int step = 0; step < 64; step++) {
                Vec3 p = v_add(ro, v_mul(rd, t));
                double d = sdf_scene(p);
                if (d < 0.001) {
                    Vec3 n = calc_normal(p);
                    Vec3 ld = v_norm(v_sub(light_pos, p));
                    double diff = v_dot(n, ld);
                    if (diff < 0.0) diff = 0.0;
                    hit = diff * 255.0;
                    break;
                }
                t += d;
                if (t > 20.0) break;
            }
            total_lum += (uint64_t)hit;
        }
    }
    return total_lum;
}

// ============================================================================
// 🥊 CHALLENGE 2: CLBG BINARY TREES MEMORY ALLOCATION TORTURE (Depth 16)
// ============================================================================
typedef struct TreeNode {
    int32_t item;
    struct TreeNode *left, *right;
} TreeNode;

static TreeNode* create_tree(int32_t item, int32_t depth) {
    TreeNode* n = (TreeNode*)malloc(sizeof(TreeNode));
    n->item = item;
    if (depth > 0) {
        n->left = create_tree(2 * item - 1, depth - 1);
        n->right = create_tree(2 * item, depth - 1);
    } else {
        n->left = NULL;
        n->right = NULL;
    }
    return n;
}

static int64_t check_tree(TreeNode* n) {
    if (!n) return 0;
    int64_t sum = n->item;
    if (n->left) sum += check_tree(n->left) - check_tree(n->right);
    return sum;
}

static void free_tree(TreeNode* n) {
    if (!n) return;
    if (n->left) free_tree(n->left);
    if (n->right) free_tree(n->right);
    free(n);
}

static int64_t bench_binary_trees(int32_t max_depth) {
    int32_t min_depth = 4;
    int64_t grand_sum = 0;

    // Stretch tree
    TreeNode* stretch = create_tree(0, max_depth + 1);
    grand_sum += check_tree(stretch);
    free_tree(stretch);

    // Long-lived tree
    TreeNode* long_lived = create_tree(0, max_depth);

    for (int32_t depth = min_depth; depth <= max_depth; depth += 2) {
        int32_t iterations = 1 << (max_depth - depth + min_depth);
        int64_t check = 0;
        for (int32_t i = 1; i <= iterations; i++) {
            TreeNode* t1 = create_tree(i, depth);
            check += check_tree(t1);
            free_tree(t1);

            TreeNode* t2 = create_tree(-i, depth);
            check += check_tree(t2);
            free_tree(t2);
        }
        grand_sum += check;
    }

    grand_sum += check_tree(long_lived);
    free_tree(long_lived);
    return grand_sum;
}

// ============================================================================
// 🥊 CHALLENGE 3: HIGH-FREQUENCY TRADING (HFT) ORDER BOOK (1,000,000 Orders)
// ============================================================================
#define MAX_LEVELS 100

typedef struct {
    int64_t total_trades;
    int64_t total_volume;
    int64_t bid_depth;
    int64_t ask_depth;
} HftResult;

static HftResult bench_hft_engine(int32_t num_orders) {
    int32_t bids[MAX_LEVELS] = {0}; // price index -> volume
    int32_t asks[MAX_LEVELS] = {0}; // price index -> volume

    int64_t total_trades = 0;
    int64_t total_volume = 0;
    uint64_t rng = 0x123456789abcdefULL;

    for (int32_t i = 0; i < num_orders; i++) {
        // Fast PRNG
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;

        int32_t is_buy = (rng & 1);
        int32_t price = 20 + ((rng >> 1) % 60); // Prices between 20 and 79
        int32_t qty = 1 + ((rng >> 8) % 100);   // Quantities between 1 and 100
        int32_t is_cancel = ((rng >> 16) % 10 == 0); // 10% cancellations

        if (is_cancel) {
            if (is_buy && bids[price] > 0) {
                bids[price] = bids[price] > qty ? bids[price] - qty : 0;
            } else if (!is_buy && asks[price] > 0) {
                asks[price] = asks[price] > qty ? asks[price] - qty : 0;
            }
            continue;
        }

        if (is_buy) {
            // Match against asks <= price
            for (int32_t p = 0; p <= price && qty > 0; p++) {
                if (asks[p] > 0) {
                    int32_t trade_qty = qty < asks[p] ? qty : asks[p];
                    asks[p] -= trade_qty;
                    qty -= trade_qty;
                    total_trades++;
                    total_volume += (int64_t)trade_qty * p;
                }
            }
            if (qty > 0) {
                bids[price] += qty;
            }
        } else {
            // Match against bids >= price
            for (int32_t p = MAX_LEVELS - 1; p >= price && qty > 0; p--) {
                if (bids[p] > 0) {
                    int32_t trade_qty = qty < bids[p] ? qty : bids[p];
                    bids[p] -= trade_qty;
                    qty -= trade_qty;
                    total_trades++;
                    total_volume += (int64_t)trade_qty * p;
                }
            }
            if (qty > 0) {
                asks[price] += qty;
            }
        }
    }

    int64_t bid_depth = 0;
    int64_t ask_depth = 0;
    for (int32_t i = 0; i < MAX_LEVELS; i++) {
        bid_depth += bids[i];
        ask_depth += asks[i];
    }

    return (HftResult){total_trades, total_volume, bid_depth, ask_depth};
}

int main(void) {
    printf("=== BRUTAL BENCHMARK: C (GCC 15.2 -O3 -march=native -flto) ===\n");

    // 1. Raymarcher (500x500)
    double t0 = get_time_ms();
    uint64_t res1 = bench_raymarch(500, 500);
    double t1 = get_time_ms();
    printf("1. Raymarcher 3D (250K rays): %.2f ms | Checksum: %llu\n", t1 - t0, (unsigned long long)res1);

    // 2. Binary Trees (Depth 16)
    t0 = get_time_ms();
    int64_t res2 = bench_binary_trees(16);
    t1 = get_time_ms();
    printf("2. Binary Trees (Depth 16):   %.2f ms | Checksum: %lld\n", t1 - t0, (long long)res2);

    // 3. HFT Order Book (1M orders)
    t0 = get_time_ms();
    HftResult res3 = bench_hft_engine(1000000);
    t1 = get_time_ms();
    printf("3. HFT Order Matching (1M):   %.2f ms | Trades: %lld | Vol: %lld\n", 
           t1 - t0, (long long)res3.total_trades, (long long)res3.total_volume);

    return 0;
}
