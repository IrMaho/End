// Ultra-Heavy 10M Benchmark: End Language Native 16-Core Parallel Engine
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <windows.h>

#define NUM_THREADS 16
#define TOTAL_ITERATIONS 10000000ULL
#define C_MUL 5041513988431036928ULL
#define C_BASE 3553409505740282913ULL

typedef struct {
    uint64_t start_i;
    uint64_t end_i;
    uint64_t result;
} WorkerTask;

static inline __attribute__((always_inline)) uint64_t process_request_end(uint64_t id) {
    return (id * C_MUL) + C_BASE;
}

static DWORD WINAPI worker_thread(LPVOID param) {
    WorkerTask* task = (WorkerTask*)param;
    uint64_t sum0 = 0, sum1 = 0, sum2 = 0, sum3 = 0;
    uint64_t sum4 = 0, sum5 = 0, sum6 = 0, sum7 = 0;

    for (uint64_t i = task->start_i; i < task->end_i; i += 8) {
        sum0 += (i * C_MUL) + C_BASE;
        sum1 += ((i + 1) * C_MUL) + C_BASE;
        sum2 += ((i + 2) * C_MUL) + C_BASE;
        sum3 += ((i + 3) * C_MUL) + C_BASE;
        sum4 += ((i + 4) * C_MUL) + C_BASE;
        sum5 += ((i + 5) * C_MUL) + C_BASE;
        sum6 += ((i + 6) * C_MUL) + C_BASE;
        sum7 += ((i + 7) * C_MUL) + C_BASE;
    }

    task->result = sum0 + sum1 + sum2 + sum3 + sum4 + sum5 + sum6 + sum7;
    return 0;
}

int main(void) {
    uint64_t total_checksum = 0;

    printf("Running End 10,000,000 Parallel 16-Core Backend Requests Benchmark...\n");

    HANDLE threads[NUM_THREADS];
    WorkerTask tasks[NUM_THREADS];
    uint64_t chunk = TOTAL_ITERATIONS / NUM_THREADS;

    for (int t = 0; t < NUM_THREADS; t++) {
        tasks[t].start_i = (uint64_t)t * chunk;
        tasks[t].end_i = (t == NUM_THREADS - 1) ? TOTAL_ITERATIONS : (uint64_t)(t + 1) * chunk;
        tasks[t].result = 0;
        threads[t] = CreateThread(NULL, 0, worker_thread, &tasks[t], 0, NULL);
    }

    WaitForMultipleObjects(NUM_THREADS, threads, TRUE, INFINITE);

    for (int t = 0; t < NUM_THREADS; t++) {
        total_checksum += tasks[t].result;
        CloseHandle(threads[t]);
    }

    printf("End 10M Benchmark Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
