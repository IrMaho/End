<div align="center">

<p align="center">
  <img src="assets/logo.png" alt="The End Programming Language" width="280" style="border-radius: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.3);" />
</p>

# 👑 The End Programming Language

**The AI-First, Zero-GC, Bare-Metal Systems & Game Programming Language**

[![CI Status](https://github.com/IrMaho/End/workflows/End%20CI/badge.svg)](https://github.com/IrMaho/End/actions)
[![Release](https://img.shields.io/github/v/release/IrMaho/End?color=58a6ff&label=Release)](https://github.com/IrMaho/End/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-3fb950.svg)](LICENSE)
[![FPS](https://img.shields.io/badge/V--Sync-120%20FPS%20Canvas-bc8cff.svg)](std/ui)
[![Memory](https://img.shields.io/badge/Memory-Zero--GC%20Regions-56d4dd.svg)](docs/ARCHITECTURE.md)
[![AI-First](https://img.shields.io/badge/AI--First-Cognitive%20Engine-f0883e.svg)](docs/AI_AGENT_PROTOCOL.md)

<p align="center">
  <a href="#-quick-install">Quick Install</a> •
  <a href="#-key-features">Features</a> •
  <a href="#-benchmark-matrix">Benchmarks</a> •
  <a href="#-code-examples">Code Examples</a> •
  <a href="#-ai-first-cognitive-engine">AI Protocol</a> •
  <a href="docs/FRAMEWORKS.md">Frameworks</a> •
  <a href="docs/ARCHITECTURE.md">Architecture</a>
</p>

---

</div>

## 📊 12-Challenge Performance Matrix

Performance measurements conform to the 12-benchmark specification in [BENCHMARKS.md](BENCHMARKS.md) and can be reproduced locally via `python benchmarks/suite12/run_suite12.py`.

| Benchmark Challenge | 👑 **End (C11)** | ⚡ **Zig (0.16.0)** | ⚡ **Rust (1.89.0)** | ⚡ **C (GCC 15.2)** | ⚡ **Go (1.25.1)** | Verification |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **1. 3D SDF Raymarcher (250K Rays)** | **42.55 ms** | 56.50 ms | 56.97 ms | 51.34 ms | 33.54 ms 🥇 | ⚠️ FP Variant |
| **2. Binary Trees (Depth 16 Dynamic)** | **40.97 ms** 🥇 | 339.94 ms | 580.61 ms | 554.73 ms | 447.87 ms | `407713` ✅ |
| **3. HFT Limit Order Engine (1M Orders)** | **27.13 ms** | 25.74 ms | 24.77 ms 🥇 | 29.19 ms | 29.53 ms | `552829538` ✅ |
| **4. SHA-256 Crypto Hashing (500K Blocks)** | **95.33 ms** | 94.99 ms 🥇 | 102.51 ms | 97.05 ms | 116.21 ms | `-4721506799343634759` ✅ |
| **5. N-Body Gravity Orbit (1M Pairwise)** | **700.44 ms** 🥇 | 2221.10 ms | 2287.63 ms | 1878.68 ms | 3275.29 ms | ⚠️ FP Variant |
| **6. SPSC Ring Buffer Queue (10M Items)** | **3.80 ms** | 2.60 ms | 1.53 ms | 0.00 ms 🥇 | 8.73 ms | ⚠️ Impl Variant |
| **7. DNA Levenshtein Matrix (1M Cells)** | **1047.25 ms** 🥇 | 2167.75 ms | 2236.03 ms | 1201.59 ms | 2196.40 ms | `525912` ✅ |
| **8. JSON Microservice Serializer (100K)** | **2.12 ms** 🥇 | 8.03 ms | 22.53 ms | 54.81 ms | 41.33 ms | `5588438541400559045` ✅ |
| **9. FSM Lexer Stream (10M Chars)** | **9.87 ms** | 11.81 ms | 13.42 ms | 8.88 ms 🥇 | 18.28 ms | ⚠️ Hash Variant |
| **10. GEMM Matrix Multiplication (512x512)** | **10.57 ms** 🥇 | 63.98 ms | 37.00 ms | 10.88 ms | 83.10 ms | `6422836` ✅ |
| **11. Monte Carlo Black-Scholes (2M Paths)** | **41.62 ms** | 29.86 ms 🥇 | 36.67 ms | 44.02 ms | 46.24 ms | `10440246` ✅ |
| **12. Super-Scalar ALU Reduction (10M)** | **658.63 ms** | 797.74 ms | 148.01 ms 🥇 | 638.15 ms | 766.68 ms | `3370198876750320971` ✅ |
| **📦 Executable Binary Size** | 🥇 **40.0 KB** | 835.0 KB | 193.0 KB | 77.4 KB | 1592.5 KB | Stripped Native |

> 📁 **Verification Datasets & Specifications:** Raw machine-readable outputs, execution percentiles, and hardware metadata are saved in [benchmarks/suite12/suite12_results.json](benchmarks/suite12/suite12_results.json). Full source codes and reproduction instructions are documented in [BENCHMARKS.md](BENCHMARKS.md). Benchmarks marked ⚠️ have implementation-specific divergences explained in detail in BENCHMARKS.md footnotes.


---

## 🏛️ Key Features

- 🛡️ **Zero-GC Deterministic Region Memory:** Memory scopes reset instantly at frame boundaries or request lifecycles without any pause or leak.
- 🔒 **Hardware Watchdog & Thermal Fuse:** Native `SwitchToThread` yield budgeting and exponential socket backoff preventing CPU spin-locking.
- ⚡ **Zero-Downtime Hot-Reload (`end dev`):** Dynamic reload preserving active session pools and counters in persistent RAM arenas.
- 🎮 **120 FPS Native Canvas (`std/ui/canvas.end`):** Hardware SIMD-accelerated canvas for real-time game physics and glassmorphism UIs.
- 🧠 **AI-First Cognitive Toolchain:** Semantic Knowledge Graph (`end graph`), Blast-Radius analysis (`end impact`), Code Slicing (`end slice`), and Micro-Evaluator (`end eval`).
- 🔌 **Official VS Code / IDE Extension:** CodeLens inline testing, Inlay Hints, and 120 FPS Visual Studio Webview sandbox.

---

## 💻 Code Examples

### 1. Hello World with High-Resolution Timing
```end
import "std/time/time.end"

pub fn main() void {
    val start = instant_now()
    println("👑 Hello, World from End Programming Language!")
    val elapsed = instant_elapsed_nanos(start)
    println("  Execution duration: " + elapsed + " ns")
}
```

### 2. Declarative High-Speed Web Server (`EndHyper`)
```end
import "std/nexus/socket_guard.end"
import "std/nexus/circuit_breaker.end"

st UserDto {
    id: i64,
    username: str,
}

@post("/api/v1/users")
@capability(net = true, disk = false, memory = "ArenaScoped")
pub fn create_user(user: UserDto) str {
    ret "{\"status\": \"created\", \"user_id\": " + user.id + "}"
}
```

### 3. Zero-GC Region Frame Scope & Simulation
```end
import "std/simulation/what_if.end"

pub fn process_physics_frame() void {
    region frame_scope {
        val baseline = 142.5
        val mutated = 118.2
        val diff = simulation_compare_outputs("SIMD Physics", baseline, mutated)
        println("Diff Gain: " + diff.delta_pct + "%")
    } // Memory reset instantly in 0 ns at scope exit!
}
```

---

## 🧠 AI-First Cognitive Engine

The End compiler includes dedicated subcommands engineered specifically for AI Agents navigating multi-million line codebases:

```powershell
# 1. Machine Knowledge Graph (Sub-millisecond token-efficient index)
end graph server.end --json

# 2. Blast-Radius & Impact Analysis
end impact server.end calculate_physics --json

# 3. Side-Effect & Capability Contracts
end effects server.end pure_physics --json

# 4. Semantic Code Slicing (Compress 50,000-line file into 50-line interface)
end slice server.end --interface-only

# 5. Structured AST Auto-Patches
end patch server.end --ast-patch patch.json --apply

# 6. Micro-Isolated Expression Evaluator (< 50 µs)
end eval "val x = 100 * 31; (x ^ 0x5AA5) % 50"

# 7. Architecture Invariant Validation
end arch check --json
```

---

## 📚 Documentation & Guides

- 📖 [Master Developer Platform Guide](docs/END_LANGUAGE_DEVELOPER_GUIDE.md)
- 🚀 [Multi-Platform Installation Guide](INSTALL.md)
- 🏛️ [Memory & Architecture Specification](docs/ARCHITECTURE.md)
- 📦 [Exclusive Frameworks Guide (Hyper, Forge, Nexus, Crypto, KV)](docs/FRAMEWORKS.md)
- 🤖 [AI-Agent Cognitive Protocol Guide](docs/AI_AGENT_PROTOCOL.md)

---

## 📄 License

End is distributed under the [MIT License](LICENSE).
Copyright © 2026 [Mohammad Javad (IrMaho)](https://github.com/IrMaho) & The End Language Community.




