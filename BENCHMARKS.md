# 👑 End Language — Official 12-Challenge Grandmaster Benchmark Suite

> **100% Transparent, Reproducible, and Formally Verified Bare-Metal Benchmark Specifications.**  
> Every benchmark in this repository executes mathematically identical algorithms with **identical checksum verifications** across C, Zig, Rust, Go, and End.

---

## 🎯 Quick Navigation & Source Code References

| Benchmark ID & Domain | Description & Workload | End Source | Zig Source | Rust Source | C Source | Go Source |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **1. 3D SDF Raymarcher** | Real-Time Vector Optics (250K Rays) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **2. Dynamic Binary Trees** | Memory Torture (Depth 16 Bump Arena) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **3. HFT Order Engine** | Fintech Matching (1M Limit Orders) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **4. SHA-256 Hashing** | Cryptography (500K Blocks / 32MB) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **5. N-Body Gravity Orbit** | Astrophysics Simulation (1M Pairwise) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **6. SPSC Ring Buffer** | Lock-Free Concurrency (10M Items) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **7. DNA Levenshtein Matrix**| Bioinformatics DP (1,000 pairs × 1K bp)| [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **8. JSON Microservice** | Web Serialization (100K Payloads) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **9. FSM Lexer Stream** | Compiler Infrastructure (10M Chars) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **10. GEMM Matrix Mul** | AI / Linear Algebra (512x512 Blocked) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **11. Monte Carlo Pricing** | Quant Finance (2M Option Contracts) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |
| **12. Super-Scalar ALU** | CPU Pipeline Crunch (10M / 500M Ops) | [`suite12_end.end`](benchmarks/suite12/suite12_end.end) | [`suite12_zig.zig`](benchmarks/suite12/suite12_zig.zig) | [`suite12_rust.rs`](benchmarks/suite12/suite12_rust.rs) | [`suite12_c.c`](benchmarks/suite12/suite12_c.c) | [`suite12_go.go`](benchmarks/suite12/suite12_go.go) |

---

## 📊 Grandmaster Performance Results (12 Challenging Real-World Tests)

*Measurements: Median Latency (P50) across 5 statistical evaluation runs with 2 warmup passes per language on native Windows x86_64.*

| Benchmark Challenge | 👑 **End (C11)** | ⚡ **Zig (0.16.0)** | ⚡ **Rust (1.89.0)** | ⚡ **C (GCC 15.2)** | ⚡ **Go (1.25.1)** | Checksum Verification |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **1. 3D SDF Raymarcher (250K Rays)** | **47.67 ms** | 60.21 ms | 63.12 ms | 53.30 ms | 30.76 ms 🥇 | `14694880` ✅ |
| **2. Binary Trees (Depth 16 Dynamic)** | **32.42 ms** 🥇 | 360.76 ms | 592.85 ms | 511.16 ms | 462.63 ms | `407713` ✅ |
| **3. HFT Limit Order Engine (1M Orders)** | **26.54 ms** | 23.38 ms 🥇 | 23.91 ms | 28.28 ms | 24.90 ms | `552829538` ✅ |
| **4. SHA-256 Crypto Hashing (500K Blocks)** | **91.58 ms** | 105.10 ms | 111.43 ms | 90.84 ms 🥇 | 104.89 ms | `-4721506799343634759` ✅ |
| **5. N-Body Gravity Orbit (1M Pairwise)** | **1943.91 ms** 🥇 | 2174.58 ms | 2141.09 ms | 2536.55 ms | 3828.78 ms | `1656141296` ✅ |
| **6. SPSC Ring Buffer Queue (10M Items)** | **4.83 ms** | 2.83 ms | 2.01 ms | 0.00 ms 🥇 | 12.17 ms | `1550000015000000` ✅ |
| **7. DNA Levenshtein Matrix (1M Cells)** | **1131.36 ms** 🥇 | 2306.27 ms | 2391.26 ms | 1283.13 ms | 2443.96 ms | `525912` ✅ |
| **8. JSON Microservice Serializer (100K)** | **64.42 ms** | 8.13 ms 🥇 | 23.89 ms | 57.67 ms | 41.57 ms | `5588438541400559045` ✅ |
| **9. FSM Lexer Stream (10M Chars)** | **10.24 ms** | 17.09 ms | 17.26 ms | 9.30 ms 🥇 | 24.16 ms | `-6471218147204355511` ✅ |
| **10. GEMM Matrix Multiplication (512x512)** | **18.59 ms** | 75.31 ms | 44.81 ms | 16.55 ms 🥇 | 79.31 ms | `6422836` ✅ |
| **11. Monte Carlo Black-Scholes (2M Paths)** | **63.66 ms** | 37.64 ms 🥇 | 44.92 ms | 52.41 ms | 48.02 ms | `10440247` ✅ |
| **12. Super-Scalar ALU Reduction (10M)** | **212.06 ms** | 0.01 ms | 0.00 ms 🥇 | 209.01 ms | 419.61 ms | `-6815960706871662336` ✅ |
| **📦 Binary Footprint (Executable Size)** | 🥇 **38.5 KB** | 834.0 KB | 184.5 KB | 76.9 KB | 1592.5 KB *(1.6MB)* | Smallest native binary |

---

## 🚀 How to Reproduce All 12 Benchmarks

Anyone can verify every single number locally on their machine:

```bash
# 1. Clone repository
git clone https://github.com/IrMaho/End.git
cd End

# 2. Run the automated 12-benchmark suite across all 5 languages
python benchmarks/suite12/run_suite12.py
```

### Exact Compilation Commands Used:
```bash
# 1. End Language
end.exe build benchmarks/suite12/suite12_end.end --strip -o suite12_end.exe

# 2. Zig
zig build-exe -O ReleaseFast -lc benchmarks/suite12/suite12_zig.zig -femit-bin=suite12_zig.exe

# 3. Rust
rustc -O -C target-cpu=native -C lto=fat -C codegen-units=1 benchmarks/suite12/suite12_rust.rs -o suite12_rust.exe

# 4. C (GCC)
gcc -O3 -march=native -flto -funroll-loops -fomit-frame-pointer -finline-functions benchmarks/suite12/suite12_c.c -o suite12_c.exe

# 5. Go
go build -ldflags="-s -w" -o suite12_go.exe benchmarks/suite12/suite12_go.go
```
