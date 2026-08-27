#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <math.h>
#include <time.h>

static uint64_t get_time_ns(void) {
#if defined(CLOCK_MONOTONIC)
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
#else
    return (uint64_t)(((double)clock() / CLOCKS_PER_SEC) * 1000000000.0);
#endif
}

// PRNG SplitMix64
static inline uint64_t splitmix64(uint64_t* state) {
    *state += 0x9E3779B97F4A7C15ULL;
    uint64_t z = *state;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}

// ==============================================================================
// 1. 3D Raymarcher (250,000 rays)
// ==============================================================================
typedef struct { float x, y, z; } Vec3;
static inline Vec3 v3(float x, float y, float z) { return (Vec3){x, y, z}; }
static inline Vec3 v_add(Vec3 a, Vec3 b) { return v3(a.x + b.x, a.y + b.y, a.z + b.z); }
static inline Vec3 v_sub(Vec3 a, Vec3 b) { return v3(a.x - b.x, a.y - b.y, a.z - b.z); }
static inline Vec3 v_scale(Vec3 a, float s) { return v3(a.x * s, a.y * s, a.z * s); }
static inline float v_dot(Vec3 a, Vec3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
static inline float v_length(Vec3 a) { return sqrtf(v_dot(a, a)); }
static inline Vec3 v_norm(Vec3 a) { float l = v_length(a); return l > 0.00001f ? v_scale(a, 1.0f / l) : a; }

static inline float sdf_sphere(Vec3 p, float r) { return v_length(p) - r; }
static inline float sdf_torus(Vec3 p, float r1, float r2) {
    float qx = sqrtf(p.x * p.x + p.z * p.z) - r1;
    return sqrtf(qx * qx + p.y * p.y) - r2;
}
static inline float sdf_scene(Vec3 p) {
    float d_sphere = sdf_sphere(v_sub(p, v3(0.0f, 1.0f, 0.0f)), 1.0f);
    float d_torus = sdf_torus(v_sub(p, v3(0.0f, 0.5f, 0.0f)), 1.2f, 0.3f);
    float d_floor = p.y;
    float d = d_sphere < d_torus ? d_sphere : d_torus;
    return d < d_floor ? d : d_floor;
}
static inline Vec3 calc_normal(Vec3 p) {
    const float eps = 0.001f;
    return v_norm(v3(
        sdf_scene(v3(p.x + eps, p.y, p.z)) - sdf_scene(v3(p.x - eps, p.y, p.z)),
        sdf_scene(v3(p.x, p.y + eps, p.z)) - sdf_scene(v3(p.x, p.y - eps, p.z)),
        sdf_scene(v3(p.x, p.y, p.z + eps)) - sdf_scene(v3(p.x, p.y, p.z - eps))
    ));
}

int64_t bench_1_raymarcher(void) {
    const int W = 500, H = 500;
    int64_t checksum = 0;
    Vec3 ro = v3(0.0f, 1.5f, -3.5f);
    Vec3 light_pos = v3(2.0f, 4.0f, -2.0f);

    for (int y = 0; y < H; y++) {
        for (int x = 0; x < W; x++) {
            float u = (2.0f * (float)x - (float)W) / (float)H;
            float v = -(2.0f * (float)y - (float)H) / (float)H;
            Vec3 rd = v_norm(v3(u, v, 1.5f));
            float t = 0.0f;
            int hit = 0;
            for (int step = 0; step < 64; step++) {
                Vec3 p = v_add(ro, v_scale(rd, t));
                float d = sdf_scene(p);
                if (d < 0.001f) {
                    Vec3 n = calc_normal(p);
                    Vec3 l = v_norm(v_sub(light_pos, p));
                    float diff = v_dot(n, l);
                    if (diff < 0.0f) diff = 0.0f;
                    int color = (int)(diff * 255.0f);
                    checksum += color;
                    hit = 1;
                    break;
                }
                t += d;
                if (t > 20.0f) break;
            }
            if (!hit) checksum += 10;
        }
    }
    return checksum;
}

// ==============================================================================
// 2. Binary Trees Depth 16 (Memory Pressure)
// ==============================================================================
typedef struct TreeNode {
    struct TreeNode* left;
    struct TreeNode* right;
    int32_t val;
} TreeNode;

static TreeNode* create_tree(int depth) {
    TreeNode* node = (TreeNode*)malloc(sizeof(TreeNode));
    node->val = depth;
    if (depth > 0) {
        node->left = create_tree(depth - 1);
        node->right = create_tree(depth - 1);
    } else {
        node->left = NULL;
        node->right = NULL;
    }
    return node;
}
static int64_t sum_tree(TreeNode* node) {
    if (!node) return 0;
    int64_t s = node->val + sum_tree(node->left) - sum_tree(node->right);
    return s;
}
static void free_tree(TreeNode* node) {
    if (!node) return;
    free_tree(node->left);
    free_tree(node->right);
    free(node);
}

int64_t bench_2_binary_trees(void) {
    int max_depth = 16;
    TreeNode* stretch = create_tree(max_depth + 1);
    int64_t check = sum_tree(stretch);
    free_tree(stretch);

    TreeNode* long_lived = create_tree(max_depth);
    for (int depth = 4; depth <= max_depth; depth += 2) {
        int iterations = 1 << (max_depth - depth + 4);
        for (int i = 1; i <= iterations; i++) {
            TreeNode* t1 = create_tree(depth);
            check += sum_tree(t1);
            free_tree(t1);
        }
    }
    check += sum_tree(long_lived);
    free_tree(long_lived);
    return check;
}

// ==============================================================================
// 3. HFT Order Matching Engine (1,000,000 Orders)
// ==============================================================================
int64_t bench_3_hft_engine(void) {
    uint64_t rng = 0x123456789ABCDEF0ULL;
    int64_t total_volume = 0;
    int64_t trades = 0;
    int32_t buy_depth[100] = {0};
    int32_t sell_depth[100] = {0};

    for (int i = 0; i < 1000000; i++) {
        uint64_t r = splitmix64(&rng);
        int side = (r >> 63) & 1;
        int price = (r % 100);
        int qty = ((r >> 8) % 50) + 1;
        int op = (r >> 16) % 10;

        if (op == 0) {
            if (side == 0) buy_depth[price] = 0;
            else sell_depth[price] = 0;
        } else if (side == 0) { // Buy
            for (int p = price; p >= 0 && qty > 0; p--) {
                if (sell_depth[p] > 0) {
                    int fill = qty < sell_depth[p] ? qty : sell_depth[p];
                    sell_depth[p] -= fill;
                    qty -= fill;
                    total_volume += (int64_t)fill * (p + 1);
                    trades++;
                }
            }
            if (qty > 0) buy_depth[price] += qty;
        } else { // Sell
            for (int p = price; p < 100 && qty > 0; p++) {
                if (buy_depth[p] > 0) {
                    int fill = qty < buy_depth[p] ? qty : buy_depth[p];
                    buy_depth[p] -= fill;
                    qty -= fill;
                    total_volume += (int64_t)fill * (p + 1);
                    trades++;
                }
            }
            if (qty > 0) sell_depth[price] += qty;
        }
    }
    return total_volume;
}

// ==============================================================================
// 4. SHA-256 Cryptographic Hashing (500,000 Blocks = 32 MB)
// ==============================================================================
static inline uint32_t rotr(uint32_t x, uint32_t n) { return (x >> n) | (x << (32 - n)); }
static const uint32_t K[64] = {
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f,0xc67178f2
};

int64_t bench_4_sha256(void) {
    uint32_t state[8] = {
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    };
    uint32_t w[64];
    uint64_t prng = 0xCAFEBABE12345678ULL;

    for (int block = 0; block < 500000; block++) {
        for (int i = 0; i < 16; i++) {
            w[i] = (uint32_t)splitmix64(&prng);
        }
        for (int i = 16; i < 64; i++) {
            uint32_t s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            uint32_t s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }
        uint32_t a = state[0], b = state[1], c = state[2], d = state[3];
        uint32_t e = state[4], f = state[5], g = state[6], h = state[7];

        for (int i = 0; i < 64; i++) {
            uint32_t S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            uint32_t ch = (e & f) ^ ((~e) & g);
            uint32_t temp1 = h + S1 + ch + K[i] + w[i];
            uint32_t S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t temp2 = S0 + maj;

            h = g; g = f; f = e; e = d + temp1;
            d = c; c = b; b = a; a = temp1 + temp2;
        }
        state[0] += a; state[1] += b; state[2] += c; state[3] += d;
        state[4] += e; state[5] += f; state[6] += g; state[7] += h;
    }
    return (int64_t)((((uint64_t)state[0]) << 32) | state[7]);
}

// ==============================================================================
// 5. N-Body Gravitational Orbital Simulation (1,000 Bodies × 1,000 Steps)
// ==============================================================================
int64_t bench_5_nbody(void) {
    const int N = 1000;
    const int STEPS = 1000;
    float pos_x[1000], pos_y[1000], pos_z[1000];
    float vel_x[1000], vel_y[1000], vel_z[1000];
    float mass[1000];
    uint64_t prng = 0x5555AAAA5555AAAAULL;

    for (int i = 0; i < N; i++) {
        pos_x[i] = ((float)(splitmix64(&prng) % 1000) / 100.0f) - 5.0f;
        pos_y[i] = ((float)(splitmix64(&prng) % 1000) / 100.0f) - 5.0f;
        pos_z[i] = ((float)(splitmix64(&prng) % 1000) / 100.0f) - 5.0f;
        vel_x[i] = 0.0f; vel_y[i] = 0.0f; vel_z[i] = 0.0f;
        mass[i] = 1.0f + ((float)(splitmix64(&prng) % 100) / 10.0f);
    }

    const float dt = 0.01f;
    const float eps2 = 0.001f;

    for (int step = 0; step < STEPS; step++) {
        for (int i = 0; i < N; i++) {
            float fx = 0.0f, fy = 0.0f, fz = 0.0f;
            for (int j = 0; j < N; j++) {
                if (i == j) continue;
                float dx = pos_x[j] - pos_x[i];
                float dy = pos_y[j] - pos_y[i];
                float dz = pos_z[j] - pos_z[i];
                float dist_sq = dx * dx + dy * dy + dz * dz + eps2;
                float dist_inv = 1.0f / sqrtf(dist_sq);
                float f = mass[j] * (dist_inv * dist_inv * dist_inv);
                fx += dx * f; fy += dy * f; fz += dz * f;
            }
            vel_x[i] += fx * dt;
            vel_y[i] += fy * dt;
            vel_z[i] += fz * dt;
        }
        for (int i = 0; i < N; i++) {
            pos_x[i] += vel_x[i] * dt;
            pos_y[i] += vel_y[i] * dt;
            pos_z[i] += vel_z[i] * dt;
        }
    }

    double total_ke = 0.0;
    for (int i = 0; i < N; i++) {
        total_ke += 0.5 * mass[i] * (vel_x[i] * vel_x[i] + vel_y[i] * vel_y[i] + vel_z[i] * vel_z[i]);
    }
    return (int64_t)(total_ke * 1000.0);
}

// ==============================================================================
// 6. Lock-Free Ring Buffer SPSC Queue (10,000,000 Items)
// ==============================================================================
int64_t bench_6_ring_buffer(void) {
    const int CAPACITY = 65536;
    const int MASK = CAPACITY - 1;
    const int TOTAL_MSGS = 10000000;
    int64_t* ring = (int64_t*)malloc(CAPACITY * sizeof(int64_t));
    int64_t total_sum = 0;

    int head = 0;
    int tail = 0;
    for (int i = 0; i < TOTAL_MSGS; i++) {
        // Enqueue
        ring[tail & MASK] = (int64_t)i * 3ULL + 17ULL;
        tail++;
        // Dequeue
        total_sum += ring[head & MASK];
        head++;
    }
    free(ring);
    return total_sum;
}

// ==============================================================================
// 7. DNA Levenshtein Alignment Distance (1,000 Pairs × 1,000 Base Pairs)
// ==============================================================================
int64_t bench_7_dna_alignment(void) {
    const int N = 1000;
    int32_t dp[1001];
    uint64_t prng = 0x9999888877776666ULL;
    char s1[1000], s2[1000];
    const char bases[] = {'A', 'C', 'G', 'T'};

    int64_t total_distance = 0;
    for (int pair = 0; pair < 1000; pair++) {
        for (int i = 0; i < N; i++) {
            s1[i] = bases[splitmix64(&prng) % 4];
            s2[i] = bases[splitmix64(&prng) % 4];
        }
        for (int j = 0; j <= N; j++) dp[j] = j;

        for (int i = 1; i <= N; i++) {
            int32_t prev = dp[0];
            dp[0] = i;
            for (int j = 1; j <= N; j++) {
                int32_t temp = dp[j];
                int cost = (s1[i - 1] == s2[j - 1]) ? 0 : 1;
                int32_t d1 = dp[j - 1] + 1;
                int32_t d2 = dp[j] + 1;
                int32_t d3 = prev + cost;
                int32_t min_d = d1 < d2 ? d1 : d2;
                dp[j] = min_d < d3 ? min_d : d3;
                prev = temp;
            }
        }
        total_distance += dp[N];
    }
    return total_distance;
}

// ==============================================================================
// 8. JSON Microservice Serialization & Router (100,000 Requests)
// ==============================================================================
int64_t bench_8_json_serializer(void) {
    char buf[512];
    int64_t total_bytes = 0;
    int64_t hash = 0;

    for (int i = 0; i < 100000; i++) {
        int len = sprintf(buf, "{\"id\":%d,\"status\":\"active\",\"latency_us\":%d,\"tags\":[\"prod\",\"edge\",\"v2\"],\"metrics\":{\"cpu\":%.1f,\"mem\":%.1f}}",
            i, (i * 37) % 500, 42.5f + (float)(i % 10), 128.4f + (float)(i % 50));
        total_bytes += len;
        hash = (hash * 31) + len + buf[len / 2];
    }
    return hash;
}

// ==============================================================================
// 9. Finite State Machine (FSM) Lexer Stream (10,000,000 Characters)
// ==============================================================================
int64_t bench_9_fsm_lexer(void) {
    const char* sample = "pub fn calculate_metrics(id: u64, active: bool) -> i64 { val base = id * 31; ret base + 10; } ";
    int sample_len = strlen(sample);
    int64_t token_count = 0;
    int64_t token_hash = 0;

    enum State { STATE_START, STATE_IDENT, STATE_NUMBER, STATE_OP };
    enum State st = STATE_START;

    for (int i = 0; i < 10000000; i++) {
        char c = sample[i % sample_len];
        switch (st) {
            case STATE_START:
                if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_') st = STATE_IDENT;
                else if (c >= '0' && c <= '9') st = STATE_NUMBER;
                else if (c != ' ' && c != '\n' && c != '\t') st = STATE_OP;
                break;
            case STATE_IDENT:
                if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_')) {
                    token_count++;
                    token_hash = (token_hash * 33) + 1;
                    st = STATE_START;
                }
                break;
            case STATE_NUMBER:
                if (!(c >= '0' && c <= '9')) {
                    token_count++;
                    token_hash = (token_hash * 33) + 2;
                    st = STATE_START;
                }
                break;
            case STATE_OP:
                token_count++;
                token_hash = (token_hash * 33) + 3;
                st = STATE_START;
                break;
        }
    }
    return token_hash + token_count;
}

// ==============================================================================
// 10. Cache-Blocked Matrix Multiplication GEMM 512x512
// ==============================================================================
int64_t bench_10_gemm_matrix(void) {
    const int N = 512;
    double* A = (double*)malloc(N * N * sizeof(double));
    double* B = (double*)malloc(N * N * sizeof(double));
    double* C = (double*)calloc(N * N, sizeof(double));

    for (int i = 0; i < N * N; i++) {
        A[i] = (double)(i % 100) * 0.01;
        B[i] = (double)((i * 3) % 100) * 0.01;
    }

    const int BLOCK = 32;
    for (int sj = 0; sj < N; sj += BLOCK) {
        for (int si = 0; si < N; si += BLOCK) {
            for (int sk = 0; sk < N; sk += BLOCK) {
                for (int i = si; i < si + BLOCK; i++) {
                    for (int k = sk; k < sk + BLOCK; k++) {
                        double a_ik = A[i * N + k];
                        for (int j = sj; j < sj + BLOCK; j++) {
                            C[i * N + j] += a_ik * B[k * N + j];
                        }
                    }
                }
            }
        }
    }

    double trace = 0.0;
    for (int i = 0; i < N; i++) trace += C[i * N + i];

    free(A); free(B); free(C);
    return (int64_t)(trace * 100.0);
}

// ==============================================================================
// 11. Monte Carlo Black-Scholes Derivatives Pricing (2,000,000 Paths)
// ==============================================================================
int64_t bench_11_monte_carlo(void) {
    const int PATHS = 2000000;
    const double S0 = 100.0, K = 100.0, T = 1.0, r = 0.05, sigma = 0.20;
    const double drift = (r - 0.5 * sigma * sigma) * T;
    const double vol = sigma * sqrt(T);
    const double discount = exp(-r * T);

    uint64_t prng = 0xFEEDFACECAFE1234ULL;
    double total_payoff = 0.0;

    for (int i = 0; i < PATHS; i += 2) {
        // Box-Muller transform for 2 normal random variables
        double u1 = (double)((splitmix64(&prng) >> 11) + 1) / 9007199254740992.0;
        double u2 = (double)((splitmix64(&prng) >> 11) + 1) / 9007199254740992.0;
        double radius = sqrt(-2.0 * log(u1));
        double theta = 2.0 * 3.14159265358979323846 * u2;
        double z1 = radius * cos(theta);
        double z2 = radius * sin(theta);

        double s_t1 = S0 * exp(drift + vol * z1);
        double s_t2 = S0 * exp(drift + vol * z2);

        double payoff1 = s_t1 > K ? (s_t1 - K) : 0.0;
        double payoff2 = s_t2 > K ? (s_t2 - K) : 0.0;

        total_payoff += (payoff1 + payoff2);
    }
    double option_price = (total_payoff / (double)PATHS) * discount;
    return (int64_t)(option_price * 1000000.0);
}

// ==============================================================================
// 12. Super-Scalar 10M Reduction (500M Ops)
// ==============================================================================
typedef struct { uint64_t id; int32_t payload_size; int64_t checksum; } Req12;
static inline __attribute__((always_inline)) Req12 process_req12(uint64_t id, int32_t size) {
    uint64_t hash = id ^ 0x9E3779B97F4A7C15ULL;
    for (int64_t j = 0; j < 50; j++) {
        hash ^= (hash << 13);
        hash ^= (hash >> 7);
        hash ^= (hash << 17);
        hash += (uint64_t)j + 0xBF58476D1CE4E5B9ULL;
    }
    return (Req12){ .id = id, .payload_size = size, .checksum = (int64_t)hash };
}

int64_t bench_12_reduction(void) {
    const uint64_t iterations = 10000000;
    int64_t total_checksum = 0;

    int64_t sum0 = 0, sum1 = 0, sum2 = 0, sum3 = 0;
    for (uint64_t i = 0; i < iterations; i += 4) {
        sum0 += process_req12(i, 256).checksum;
        sum1 += process_req12(i + 1, 256).checksum;
        sum2 += process_req12(i + 2, 256).checksum;
        sum3 += process_req12(i + 3, 256).checksum;
    }
    total_checksum = sum0 + sum1 + sum2 + sum3;
    return total_checksum;
}

// ==============================================================================
// Main Dispatcher
// ==============================================================================
int main(int argc, char** argv) {
    if (argc < 2) {
        printf("Usage: suite12_c.exe <bench_id (1..12)>\n");
        return 1;
    }
    int id = atoi(argv[1]);
    int64_t check = 0;

    uint64_t t0 = get_time_ns();
    switch (id) {
        case 1: check = bench_1_raymarcher(); break;
        case 2: check = bench_2_binary_trees(); break;
        case 3: check = bench_3_hft_engine(); break;
        case 4: check = bench_4_sha256(); break;
        case 5: check = bench_5_nbody(); break;
        case 6: check = bench_6_ring_buffer(); break;
        case 7: check = bench_7_dna_alignment(); break;
        case 8: check = bench_8_json_serializer(); break;
        case 9: check = bench_9_fsm_lexer(); break;
        case 10: check = bench_10_gemm_matrix(); break;
        case 11: check = bench_11_monte_carlo(); break;
        case 12: check = bench_12_reduction(); break;
        default: printf("Invalid bench ID\n"); return 1;
    }
    uint64_t t1 = get_time_ns();
    double ms = (double)(t1 - t0) / 1000000.0;
    printf("RESULT:bench=%d,time_ms=%.3f,checksum=%lld\n", id, ms, (long long)check);
    return 0;
}
