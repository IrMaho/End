# ?? End Language — Benchmark Methodology & Specification

This document details the exact methodology, hardware parameters, statistical metrics, and anti-optimization validations used to evaluate the **End Language** compiler toolchain (`endc`) across diverse systems workloads.

---

## ?? Benchmark Principles

1. **Identical Algorithms:** All implementations (End, C, Rust, Zig, Go) execute strictly equivalent algorithms with matching inputs.
2. **Output Verification:** Every benchmark run calculates and verifies an anti-optimization checksum to prevent compiler dead-code elimination.
3. **Warmup Cycles:** Every measurement performs 1,000 warmup iterations prior to recording timings.
4. **Statistical Distribution:** Metrics report Min, Max, Mean, Median, P50, P90, P95, and P99 latency percentiles alongside throughput.

---

## ?? Workload Suites

| Workload | Description | Core Operations Measured |
| :--- | :--- | :--- |
| **SimdReduction** | 10,000,000 element numeric reduction | AVX2/NEON vector math, memory bandwidth |
| **RingBufferThroughput** | 1,000,000 lock-free ring buffer writes | Cache line contention, branch prediction |
| **AstTreeTraversal** | 100,000 node recursive AST visitor | Pointer chasing, stack frame allocation |
| **RegionArenaAllocation** | 50,000 region block allocations & resets | Zero-GC memory lifecycle, bump-allocator |

---

## ?? Running the Benchmarks

```powershell
# Run the complete reproducible benchmark suite
powershell -ExecutionPolicy Bypass -File .\benchmark\run_benchmarks.ps1

# Run the 10-Million Stress Suite
powershell -ExecutionPolicy Bypass -File .\benchmark\stress_10m\run_10m_suite.ps1
```
