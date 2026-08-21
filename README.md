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

## 📊 Multi-Language Performance Matrix

Performance measurements conform to the statistical specification in [BENCHMARKS.md](BENCHMARKS.md) and can be reproduced locally via powershell -File .\benchmark\run_benchmarks.ps1.

| Language / Backend | Workload | P50 Latency (ms) | P99 Latency (ms) | Mean (ms) | StdDev (ms) | Verification Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 👑 **End (C11 Backend)** | 10M Reduction | **21.80 ms** | **38.90 ms** | **22.43 ms** | **5.04 ms** | ✔ Live Measured |
| **C (GCC 15.2 -O3)** | 10M Reduction | 15.82 ms | 37.79 ms | 17.15 ms | 4.14 ms | ✔ Live Measured |
| **Zig (ReleaseFast)** | 10M Reduction | 7.16 ms | 21.32 ms | 7.93 ms | 3.45 ms | ✔ Live Measured |
| **Rust (Release -O3)** | 10M Reduction | 7.29 ms | 21.51 ms | 8.11 ms | 3.24 ms | ✔ Live Measured |
| **Go (1.25.1)** | 10M Reduction | 45.69 ms | 59.11 ms | 46.33 ms | 5.01 ms | ✔ Live Measured |

> Raw machine-readable benchmark outputs, hardware metadata, and execution percentiles are automatically generated in [enchmark/benchmark_results.json](benchmark/benchmark_results.json).


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




