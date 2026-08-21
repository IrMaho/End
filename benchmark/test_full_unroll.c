#include <stdio.h>
#include <stdint.h>

typedef struct {
    uint64_t id;
    int32_t payload_size;
    int64_t checksum;
} Request;

static inline __attribute__((always_inline)) Request process_request(uint64_t id, int32_t size) {
    int64_t hash = 17;
    int64_t id_i64 = (int64_t)id;
    #pragma unroll
    for (int64_t j = 0; j < 50; j++) {
        hash = (int64_t)((uint64_t)hash * 31ULL + (uint64_t)id_i64 + (uint64_t)j);
    }
    return (Request){ .id = id, .payload_size = size, .checksum = hash };
}

int main(void) {
    const uint64_t iterations = 1000000;
    int64_t total_checksum = 0;
    printf("Running End Backend Benchmark (1,000,000 requests)...\n");

    for (uint64_t i = 0; i < iterations; i++) {
        Request req = process_request(i, 256);
        total_checksum += req.checksum;
    }

    printf("End Benchmark Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
