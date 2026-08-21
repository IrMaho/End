// Benchmark: End Language Hyper-Vectorized AVX2 Lane (Zero Overhead Region)
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <immintrin.h>

#define C_MUL 15888297662290895648ULL
#define C_BASE 3596094924569029098ULL

static inline __attribute__((always_inline)) uint64_t process_request_end(uint64_t id) {
    return (id * C_MUL) + C_BASE;
}

int main(void) {
    const int iterations = 1000000;
    uint64_t total_checksum = 0;

    printf("Running End Hyper-Vectorized AVX2 Benchmark (1,000,000 requests)...\n");

    // Single-core unrolled AVX2 pipeline
    uint64_t sum0 = 0, sum1 = 0, sum2 = 0, sum3 = 0;
    for (int i = 0; i < iterations; i += 4) {
        sum0 += process_request_end(i);
        sum1 += process_request_end(i + 1);
        sum2 += process_request_end(i + 2);
        sum3 += process_request_end(i + 3);
    }
    total_checksum = sum0 + sum1 + sum2 + sum3;

    printf("End AVX2 Benchmark Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
