// Benchmark 2: C (C11 + Arena Allocator)
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>

typedef struct {
    uint64_t id;
    int32_t payload_size;
    int64_t checksum;
} Request;

static Request process_request(uint64_t id, int32_t size) {
    int64_t hash = 17;
    for (int j = 0; j < 50; j++) {
        hash = (hash * 31) + id + j;
    }
    return (Request){ .id = id, .payload_size = size, .checksum = hash };
}

int main(void) {
    int iterations = 1000000;
    int64_t total_checksum = 0;

    printf("Running C Backend Benchmark (1,000,000 requests)...\n");

    for (int i = 0; i < iterations; i++) {
        Request req = process_request(i, 256);
        total_checksum += req.checksum;
    }

    printf("C Benchmark Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
