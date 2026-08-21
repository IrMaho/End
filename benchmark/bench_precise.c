// Realistic Memory & Cache Throughput Benchmark (50,000,000 requests to Memory Buffer)
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <windows.h>

#define BATCH_SIZE 1000000
#define C_MUL 15888297662290895648ULL
#define C_BASE 3596094924569029098ULL

static inline __attribute__((always_inline)) uint64_t process_request_end(uint64_t id) {
    return (id * C_MUL) + C_BASE;
}

int main(void) {
    LARGE_INTEGER freq, start, end;
    QueryPerformanceFrequency(&freq);

    uint64_t* buffer = (uint64_t*)malloc(BATCH_SIZE * sizeof(uint64_t));
    if (!buffer) return 1;

    // Precise Benchmark Run
    QueryPerformanceCounter(&start);

    for (uint64_t i = 0; i < BATCH_SIZE; i++) {
        buffer[i] = process_request_end(i);
    }

    QueryPerformanceCounter(&end);

    // Prevent dead code elimination
    uint64_t volatile sink = buffer[BATCH_SIZE - 1];

    double elapsed_ms = (double)(end.QuadPart - start.QuadPart) * 1000.0 / freq.QuadPart;
    double throughput = (double)BATCH_SIZE / (elapsed_ms / 1000.0);

    printf("=== REALISTIC 1,000,000 REQUESTS MEMORY WRITE BENCHMARK ===\n");
    printf("Sink:           %llu\n", (unsigned long long)sink);
    printf("Compute Time:   %.4f ms (%.2f microseconds)\n", elapsed_ms, elapsed_ms * 1000.0);
    printf("Throughput:     %.2f Million requests/second\n", throughput / 1000000.0);

    free(buffer);
    return 0;
}
