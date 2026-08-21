// Ultra-Heavy 10M Benchmark: C (C11 + Clang/Zig Native)
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

typedef struct {
    uint64_t id;
    int32_t payload_size;
    int64_t checksum;
} Request;

static inline __attribute__((always_inline)) Request process_request(uint64_t id, int32_t size) {
    int64_t hash = 17;
    for (int j = 0; j < 32; j++) {
        hash = (hash * 31) + id + j;
    }
    return (Request){ .id = id, .payload_size = size, .checksum = hash };
}

int main(void) {
    int iterations = 10000000;
    int64_t total_checksum = 0;

    printf("Running C 10,000,000 Heavy Backend Requests Benchmark...\n");

    for (int i = 0; i < iterations; i++) {
        Request req = process_request(i, 256);
        total_checksum += req.checksum;
    }

    printf("C 10M Benchmark Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
