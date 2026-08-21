#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <windows.h>

static double get_time_sec(void) {
    LARGE_INTEGER freq, counter;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&counter);
    return (double)counter.QuadPart / (double)freq.QuadPart;
}

// 1. Bitwise & Math Crunch (100M iterations)
static uint64_t bench_compute(uint64_t iterations) {
    uint64_t state = 0x853c49e6748fea9bULL;
    for (uint64_t i = 0; i < iterations; i++) {
        state ^= (state << 13);
        state ^= (state >> 7);
        state ^= (state << 17);
        state = state * 6364136223846793005ULL + 1442695040888963407ULL;
    }
    return state;
}

// 2. High-throughput Allocation & Aggregation (5,000 batches x 20,000 elements)
static int64_t bench_memory(int batches, int elements_per_batch) {
    int64_t grand_total = 0;
    for (int b = 0; b < batches; b++) {
        int64_t* arr = (int64_t*)malloc(elements_per_batch * sizeof(int64_t));
        if (!arr) return 0;
        int64_t batch_sum = 0;
        for (int i = 0; i < elements_per_batch; i++) {
            arr[i] = (int64_t)(b * 31 + i * 17);
            batch_sum += arr[i];
        }
        grand_total += (batch_sum ^ (int64_t)arr[0]);
        free(arr);
    }
    return grand_total;
}

// 3. Deep Recursion (Fibonacci 42)
static int64_t fib(int64_t n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

int main(void) {
    printf("=== C Benchmark (GCC -O3) ===\n");

    // Bench 1
    double t0 = get_time_sec();
    uint64_t res1 = bench_compute(100000000ULL);
    double t1 = get_time_sec();
    printf("1. Compute (100M iter): %.2f ms (Hash: %llu)\n", (t1 - t0) * 1000.0, (unsigned long long)res1);

    // Bench 2
    t0 = get_time_sec();
    int64_t res2 = bench_memory(5000, 20000);
    t1 = get_time_sec();
    printf("2. Memory Churn (100M items): %.2f ms (Sum: %lld)\n", (t1 - t0) * 1000.0, (long long)res2);

    // Bench 3
    t0 = get_time_sec();
    int64_t res3 = fib(42);
    t1 = get_time_sec();
    printf("3. Recursion (fib 42): %.2f ms (Val: %lld)\n", (t1 - t0) * 1000.0, (long long)res3);

    return 0;
}
