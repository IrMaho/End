<div align="center">

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

## 📊 12-Challenge Grandmaster Performance Matrix

Performance measurements conform to the 12-benchmark specification in [BENCHMARKS.md](BENCHMARKS.md) and can be reproduced locally via `python benchmarks/suite12/run_suite12.py`.

| Benchmark Challenge | 👑 **End (C11)** | ⚡ **Zig (0.16.0)** | ⚡ **Rust (1.89.0)** | ⚡ **C (GCC 15.2)** | ⚡ **Go (1.25.1)** | Verification |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **1. 3D SDF Raymarcher (250K Rays)** | **68.23 ms** | 59.79 ms | 60.18 ms | 50.10 ms | 41.30 ms 🥇 | `17840942` ✅ |
| **2. Binary Trees (Depth 16 Dynamic)** | **487.85 ms** | 351.88 ms 🥇 | 561.47 ms | 493.55 ms | 455.79 ms | `407713` ✅ |
| **3. HFT Limit Order Engine (1M Orders)** | **26.14 ms** | 22.50 ms 🥇 | 23.72 ms | 27.57 ms | 25.85 ms | `552829538` ✅ |
| **4. SHA-256 Crypto Hashing (500K Blocks)** | **95.97 ms** | 92.63 ms | 102.03 ms | 90.61 ms 🥇 | 102.45 ms | `-4721506799343634759` ✅ |
| **5. N-Body Gravity Orbit (1M Pairwise)** | **1858.70 ms** | 2164.25 ms | 2180.32 ms | 1790.46 ms 🥇 | 3050.80 ms | `1656141296` ✅ |
| **6. SPSC Ring Buffer Queue (10M Items)** | **3.38 ms** | 2.57 ms | 1.46 ms | 0.00 ms 🥇 | 8.09 ms | `1550000015000000` ✅ |
| **7. DNA Levenshtein Matrix (1M Cells)** | **970.42 ms** 🥇 | 2188.07 ms | 2194.66 ms | 1203.28 ms | 2266.70 ms | `525912` ✅ |
| **8. JSON Microservice Serializer (100K)** | **59.05 ms** | 12.38 ms 🥇 | 25.05 ms | 55.05 ms | 39.07 ms | `5588438541400559045` ✅ |
| **9. FSM Lexer Stream (10M Chars)** | **8.13 ms** 🥇 | 12.52 ms | 16.67 ms | 8.17 ms | 20.54 ms | `-6471218147204355511` ✅ |
| **10. GEMM Matrix Multiplication (512x512)** | **62.65 ms** | 73.59 ms | 58.50 ms | 23.99 ms 🥇 | 109.85 ms | `6422836` ✅ |
| **11. Monte Carlo Black-Scholes (2M Paths)** | **51.31 ms** | 32.88 ms 🥇 | 41.72 ms | 43.75 ms | 49.31 ms | `10440247` ✅ |
| **12. Super-Scalar ALU Reduction (10M)** | **208.19 ms** | 0.01 ms | 0.00 ms 🥇 | 212.95 ms | 454.54 ms | `-6815960706871662336` ✅ |
| **📦 Executable Binary Size** | 🥇 **41.0 KB** | 834.0 KB | 184.5 KB | 76.9 KB | 1592.5 KB | Stripped Native |

> 📁 **Verification Datasets & Specifications:** Raw machine-readable outputs, execution percentiles, and hardware metadata are saved in [benchmarks/suite12/suite12_results.json](benchmarks/suite12/suite12_results.json). Full source codes and reproduction instructions are documented in [BENCHMARKS.md](BENCHMARKS.md).

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




