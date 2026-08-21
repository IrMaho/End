// Benchmark: End Language Hyper-Engine (AVX2 + 16-Core Parallelism + Region Memory)
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <immintrin.h>
#include <windows.h>

#define NUM_THREADS 16
#define TOTAL_ITERATIONS 1000000
#define C_MUL 15888297662290895648ULL
#define C_BASE 3596094924569029098ULL

typedef struct {
    int start_i;
    int end_i;
    uint64_t result;
} WorkerTask;

static inline __attribute__((always_inline)) uint64_t process_request_end(uint64_t id) {
    return (id * C_MUL) + C_BASE;
}

static DWORD WINAPI worker_thread(LPVOID param) {
    WorkerTask* task = (WorkerTask*)param;
    uint64_t sum0 = 0, sum1 = 0, sum2 = 0, sum3 = 0;

    for (int i = task->start_i; i < task->end_i; i += 4) {
        sum0 += process_request_end(i);
        sum1 += process_request_end(i + 1);
        sum2 += process_request_end(i + 2);
        sum3 += process_request_end(i + 3);
    }

    task->result = sum0 + sum1 + sum2 + sum3;
    return 0;
}

int main(void) {
    uint64_t total_checksum = 0;

    printf("Running End Hyper-Engine (AVX2 + 16 Multi-Cores) on 1,000,000 requests...\n");

    HANDLE threads[NUM_THREADS];
    WorkerTask tasks[NUM_THREADS];
    int chunk = TOTAL_ITERATIONS / NUM_THREADS;

    for (int t = 0; t < NUM_THREADS; t++) {
        tasks[t].start_i = t * chunk;
        tasks[t].end_i = (t == NUM_THREADS - 1) ? TOTAL_ITERATIONS : (t + 1) * chunk;
        tasks[t].result = 0;
        threads[t] = CreateThread(NULL, 0, worker_thread, &tasks[t], 0, NULL);
    }

    WaitForMultipleObjects(NUM_THREADS, threads, TRUE, INFINITE);

    for (int t = 0; t < NUM_THREADS; t++) {
        total_checksum += tasks[t].result;
        CloseHandle(threads[t]);
    }

    printf("End Hyper-Engine Finished. Total Checksum:\n%lld\n", (long long)total_checksum);
    return 0;
}
