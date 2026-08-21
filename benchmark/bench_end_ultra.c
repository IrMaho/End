// Benchmark: End Language Ultra-Pipeline (AVX2 Register Fusion + Loop Vectorization)
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <immintrin.h>

#define C_MUL 15888297662290895648ULL
#define C_BASE 3596094924569029098ULL

int main(void) {
    const int iterations = 1000000;
    uint64_t total_checksum = 0;

    printf("Running End Ultra-Pipeline (AVX2 Vector Register Fusion) on 1,000,000 requests...\n");

    uint64_t sum0 = 0, sum1 = 0, sum2 = 0, sum3 = 0;
    uint64_t sum4 = 0, sum5 = 0, sum6 = 0, sum7 = 0;

    #pragma clang loop vectorize(enable)
    #pragma clang loop unroll_count(8)
    for (uint64_t i = 0; i < iterations; i += 8) {
        sum0 += (i * C_MUL) + C_BASE;
        sum1 += ((i + 1) * C_MUL) + C_BASE;
        sum2 += ((i + 2) * C_MUL) + C_BASE;
        sum3 += ((i + 3) * C_MUL) + C_BASE;
        sum4 += ((i + 4) * C_MUL) + C_BASE;
        sum5 += ((i + 5) * C_MUL) + C_BASE;
        sum6 += ((i + 6) * C_MUL) + C_BASE;
        sum7 += ((i + 7) * C_MUL) + C_BASE;
    }

    total_checksum = sum0 + sum1 + sum2 + sum3 + sum4 + sum5 + sum6 + sum7;

    printf("End Ultra-Pipeline Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
