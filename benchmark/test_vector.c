#include <stdio.h>
#include <stdint.h>

typedef struct {
    uint64_t id;
    int32_t payload_size;
    int64_t checksum;
} Request;

static inline __attribute__((always_inline)) Request process_request(uint64_t id, int32_t size) {
    uint64_t hash = 17;
    for (uint64_t j = 0; j < 50; j++) {
        hash = (hash * 31) + id + j;
    }
    return (Request){ .id = id, .payload_size = size, .checksum = (int64_t)hash };
}

int main(void) {
    const uint64_t iterations = 1000000;
    int64_t total_checksum = 0;

    #pragma clang loop vectorize(enable) interleave(enable)
    for (uint64_t i = 0; i < iterations; i++) {
        Request req = process_request(i, 256);
        total_checksum += req.checksum;
    }

    printf("%lld\n", (long long)total_checksum);
    return 0;
}
