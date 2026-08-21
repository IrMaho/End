// Ultra-Heavy Benchmark 1: Real Memory Allocation & Region Scoping Stress Test
// 10,000,000 Dynamic Object Allocations & Lifecycle Management
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <windows.h>
#include <psapi.h>

#define NUM_ALLOCS 10000000

typedef struct {
    char* buffer;
    size_t capacity;
    size_t offset;
} EndRegion;

static inline __attribute__((always_inline)) EndRegion* end_region_create(size_t cap) {
    EndRegion* r = (EndRegion*)malloc(sizeof(EndRegion));
    r->buffer = (char*)malloc(cap);
    r->capacity = cap;
    r->offset = 0;
    return r;
}

static inline __attribute__((always_inline)) void* end_region_alloc(EndRegion* r, size_t size) {
    if (r->offset + size > r->capacity) return NULL;
    void* ptr = (void*)(r->buffer + r->offset);
    r->offset += (size + 15) & ~15; // 16-byte alignment
    return ptr;
}

static inline __attribute__((always_inline)) void end_region_reset(EndRegion* r) {
    r->offset = 0;
}

static inline __attribute__((always_inline)) void end_region_destroy(EndRegion* r) {
    if (r) { free(r->buffer); free(r); }
}

typedef struct {
    uint64_t request_id;
    uint32_t payload_length;
    uint32_t status_code;
    uint64_t checksum;
    char data[32];
} ServerRequest;

static size_t get_peak_memory_kb() {
    PROCESS_MEMORY_COUNTERS pmc;
    if (GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof(pmc))) {
        return pmc.PeakWorkingSetSize / 1024;
    }
    return 0;
}

int main(void) {
    LARGE_INTEGER freq, start, end;
    QueryPerformanceFrequency(&freq);

    printf("========================================================================\n");
    printf("🔥 ULTRA-HEAVY TEST 1: 10,000,000 Dynamic Object Allocation Stress Test\n");
    printf("========================================================================\n");

    uint64_t volatile sink = 0;

    // --- Mode A: Traditional Heap Malloc/Free (C/C++ Baseline) ---
    printf("[1/2] Running Traditional Heap Malloc/Free Baseline (10,000,000 allocs)...\n");
    QueryPerformanceCounter(&start);
    for (int i = 0; i < NUM_ALLOCS; i++) {
        ServerRequest* req = (ServerRequest*)malloc(sizeof(ServerRequest));
        req->request_id = i;
        req->status_code = 200;
        req->checksum = (uint64_t)i * 31;
        sink += req->checksum;
        free(req);
    }
    QueryPerformanceCounter(&end);
    double malloc_ms = (double)(end.QuadPart - start.QuadPart) * 1000.0 / freq.QuadPart;

    // --- Mode B: End Deterministic Region Scoping ---
    printf("[2/2] Running End Deterministic Region Allocator (10,000,000 allocs)...\n");
    EndRegion* region = end_region_create(1024 * 1024); // 1MB reusable region chunk

    QueryPerformanceCounter(&start);
    for (int i = 0; i < NUM_ALLOCS; i++) {
        ServerRequest* req = (ServerRequest*)end_region_alloc(region, sizeof(ServerRequest));
        if (!req) {
            end_region_reset(region);
            req = (ServerRequest*)end_region_alloc(region, sizeof(ServerRequest));
        }
        req->request_id = i;
        req->status_code = 200;
        req->checksum = (uint64_t)i * 31;
        sink += req->checksum;
    }
    end_region_destroy(region);
    QueryPerformanceCounter(&end);
    double end_region_ms = (double)(end.QuadPart - start.QuadPart) * 1000.0 / freq.QuadPart;

    printf("\n📊 MEMORY STRESS TEST RESULTS (10,000,000 Allocations):\n");
    printf("  - Traditional Heap Malloc:  %.2f ms (%.2fM allocs/sec)\n", malloc_ms, (NUM_ALLOCS / malloc_ms) / 1000.0);
    printf("  - End Region-Based Engine:   %.2f ms (%.2fM allocs/sec)\n", end_region_ms, (NUM_ALLOCS / end_region_ms) / 1000.0);
    printf("  - Speedup of End vs Malloc:  %.2fx FASTER\n", malloc_ms / end_region_ms);
    printf("  - Peak Working Set Memory:   %zu KB\n", get_peak_memory_kb());
    printf("  - Verification Sink:         %llu\n", (unsigned long long)sink);

    return 0;
}
