// Ultra-Heavy Benchmark 3: 100,000,000 Vector Elements Big-Data Processing & GFLOPS Compute
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <windows.h>
#include <immintrin.h>

#define DATA_SIZE 100000000ULL

int main(void) {
    LARGE_INTEGER freq, start, end;
    QueryPerformanceFrequency(&freq);

    printf("========================================================================\n");
    printf("🚀 ULTRA-HEAVY TEST 3: 100,000,000 Float64 Big-Data & Vector Processing\n");
    printf("========================================================================\n");

    double* vec_a = (double*)_aligned_malloc(DATA_SIZE * sizeof(double), 64);
    double* vec_b = (double*)_aligned_malloc(DATA_SIZE * sizeof(double), 64);
    double* vec_res = (double*)_aligned_malloc(DATA_SIZE * sizeof(double), 64);

    if (!vec_a || !vec_b || !vec_res) {
        printf("Memory allocation failed for 100M elements!\n");
        return 1;
    }

    // Initialize arrays
    for (uint64_t i = 0; i < 1000; i++) {
        vec_a[i] = 1.05 * (double)i;
        vec_b[i] = 2.718;
    }

    printf("Executing 100,000,000 Element AVX2 Vector Crunching...\n");

    QueryPerformanceCounter(&start);

    // 4 double elements per 256-bit AVX2 register with FMA
    __m256d v_scale = _mm256_set1_pd(1.41421356);

    #pragma clang loop vectorize(enable)
    #pragma clang loop unroll_count(8)
    for (uint64_t i = 0; i < DATA_SIZE; i += 4) {
        __m256d va = _mm256_load_pd(&vec_a[i]);
        __m256d vb = _mm256_load_pd(&vec_b[i]);
        // FMA: (va * v_scale) + vb
        __m256d vr = _mm256_fmadd_pd(va, v_scale, vb);
        _mm256_store_pd(&vec_res[i], vr);
    }

    QueryPerformanceCounter(&end);

    double elapsed_ms = (double)(end.QuadPart - start.QuadPart) * 1000.0 / freq.QuadPart;
    double ops_per_sec = (double)(DATA_SIZE * 2) / (elapsed_ms / 1000.0); // 2 FLOPs per FMA
    double gflops = ops_per_sec / 1000000000.0;

    printf("\n📊 BIG-DATA VECTOR PROCESSING RESULTS (100,000,000 Items):\n");
    printf("  - Total Processing Time: %.2f ms\n", elapsed_ms);
    printf("  - Processing Bandwidth:  %.2f Million elements/second\n", (DATA_SIZE / elapsed_ms) / 1000.0);
    printf("  - Compute Throughput:    %.2f GigaFLOPS (Billion Floating-Point Ops/sec)\n", gflops);
    printf("  - Memory Transferred:    %.2f GB\n", (DATA_SIZE * sizeof(double) * 3) / (1024.0 * 1024.0 * 1024.0));

    _aligned_free(vec_a);
    _aligned_free(vec_b);
    _aligned_free(vec_res);
    return 0;
}
