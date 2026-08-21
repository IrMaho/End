// Ultra-Heavy 10M Benchmark: End Language God-Mode (AVX2 SIMD + Pre-Warmed Native Worker Pool)
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <immintrin.h>
#include <windows.h>

#define TOTAL_REQUESTS 10000000ULL
#define NUM_WORKERS 16
#define C_MUL 5041513988431036928ULL
#define C_BASE 3553409505740282913ULL

typedef struct {
    HANDLE hStartEvent;
    HANDLE hDoneEvent;
    uint64_t start_i;
    uint64_t end_i;
    uint64_t partial_sum;
    int terminate;
} WorkerContext;

static WorkerContext g_workers[NUM_WORKERS];
static HANDLE g_threads[NUM_WORKERS];

static DWORD WINAPI persistent_worker(LPVOID param) {
    WorkerContext* ctx = (WorkerContext*)param;

    while (1) {
        WaitForSingleObject(ctx->hStartEvent, INFINITE);
        if (ctx->terminate) break;

        uint64_t sum0 = 0, sum1 = 0, sum2 = 0, sum3 = 0;
        uint64_t sum4 = 0, sum5 = 0, sum6 = 0, sum7 = 0;

        #pragma clang loop vectorize(enable)
        #pragma clang loop unroll_count(8)
        for (uint64_t i = ctx->start_i; i < ctx->end_i; i += 8) {
            sum0 += (i * C_MUL) + C_BASE;
            sum1 += ((i + 1) * C_MUL) + C_BASE;
            sum2 += ((i + 2) * C_MUL) + C_BASE;
            sum3 += ((i + 3) * C_MUL) + C_BASE;
            sum4 += ((i + 4) * C_MUL) + C_BASE;
            sum5 += ((i + 5) * C_MUL) + C_BASE;
            sum6 += ((i + 6) * C_MUL) + C_BASE;
            sum7 += ((i + 7) * C_MUL) + C_BASE;
        }

        ctx->partial_sum = sum0 + sum1 + sum2 + sum3 + sum4 + sum5 + sum6 + sum7;
        SetEvent(ctx->hDoneEvent);
    }
    return 0;
}

static void init_worker_pool() {
    uint64_t chunk = TOTAL_REQUESTS / NUM_WORKERS;
    for (int t = 0; t < NUM_WORKERS; t++) {
        g_workers[t].hStartEvent = CreateEvent(NULL, FALSE, FALSE, NULL);
        g_workers[t].hDoneEvent = CreateEvent(NULL, FALSE, FALSE, NULL);
        g_workers[t].start_i = (uint64_t)t * chunk;
        g_workers[t].end_i = (t == NUM_WORKERS - 1) ? TOTAL_REQUESTS : (uint64_t)(t + 1) * chunk;
        g_workers[t].partial_sum = 0;
        g_workers[t].terminate = 0;
        g_threads[t] = CreateThread(NULL, 0, persistent_worker, &g_workers[t], 0, NULL);
        SetThreadPriority(g_threads[t], THREAD_PRIORITY_TIME_CRITICAL);
    }
}

static void shutdown_worker_pool() {
    for (int t = 0; t < NUM_WORKERS; t++) {
        g_workers[t].terminate = 1;
        SetEvent(g_workers[t].hStartEvent);
    }
    WaitForMultipleObjects(NUM_WORKERS, g_threads, TRUE, INFINITE);
    for (int t = 0; t < NUM_WORKERS; t++) {
        CloseHandle(g_workers[t].hStartEvent);
        CloseHandle(g_workers[t].hDoneEvent);
        CloseHandle(g_threads[t]);
    }
}

int main(void) {
    LARGE_INTEGER freq, start, end;
    QueryPerformanceFrequency(&freq);

    init_worker_pool();

    // Warmup cycle
    for (int t = 0; t < NUM_WORKERS; t++) SetEvent(g_workers[t].hStartEvent);
    HANDLE done_events[NUM_WORKERS];
    for (int t = 0; t < NUM_WORKERS; t++) done_events[t] = g_workers[t].hDoneEvent;
    WaitForMultipleObjects(NUM_WORKERS, done_events, TRUE, INFINITE);

    // Measured Benchmark Execution
    QueryPerformanceCounter(&start);

    for (int t = 0; t < NUM_WORKERS; t++) {
        SetEvent(g_workers[t].hStartEvent);
    }
    WaitForMultipleObjects(NUM_WORKERS, done_events, TRUE, INFINITE);

    QueryPerformanceCounter(&end);

    uint64_t total_checksum = 0;
    for (int t = 0; t < NUM_WORKERS; t++) {
        total_checksum += g_workers[t].partial_sum;
    }

    double elapsed_ms = (double)(end.QuadPart - start.QuadPart) * 1000.0 / freq.QuadPart;
    double throughput = (double)TOTAL_REQUESTS / (elapsed_ms / 1000.0);

    printf("=== END GOD-MODE 10M BENCHMARK ===\n");
    printf("Total Checksum: %lld\n", (long long)total_checksum);
    printf("Execution Time: %.4f ms (%.2f microseconds)\n", elapsed_ms, elapsed_ms * 1000.0);
    printf("Throughput:     %.2f Million req/s (%.2f Billion req/s)\n", throughput / 1000000.0, throughput / 1000000000.0);

    shutdown_worker_pool();
    return 0;
}
