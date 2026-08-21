#include <stdio.h>
#include <stdint.h>

typedef struct {
    uint64_t id;
    int32_t payload_size;
    int64_t checksum;
} Request;

static inline __attribute__((always_inline)) Request process_request(uint64_t id, int32_t size) {
    uint64_t hash = 17;
    #pragma GCC unroll 8
    for (uint64_t j = 0; j < 50; j++) {
        hash = (hash * 31) + id + j;
    }
    return (Request){ .id = id, .payload_size = size, .checksum = (int64_t)hash };
}

int main(void) {
    const uint64_t iterations = 1000000;
    int64_t total_checksum = 0;
    printf("Running End Backend Benchmark (1,000,000 requests)...\n");

    int64_t sum0 = 0, sum1 = 0, sum2 = 0, sum3 = 0;
    for (uint64_t i = 0; i < iterations; i += 4) {
        sum0 += process_request(i, 256).checksum;
        sum1 += process_request(i + 1, 256).checksum;
        sum2 += process_request(i + 2, 256).checksum;
        sum3 += process_request(i + 3, 256).checksum;
    }
    total_checksum = sum0 + sum1 + sum2 + sum3;

    printf("End Benchmark Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
