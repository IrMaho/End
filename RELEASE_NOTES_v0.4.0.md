# 👑 End Language v0.4.0-alpha Official Global Release Notes

We are thrilled to announce the official release of **The End Programming Language (v0.4.0-alpha)** — the AI-First, Zero-GC Systems & Game Programming Language!

---

## 🚀 What's New in v0.4.0-alpha

### 1. ⚡ Extreme Bare-Metal Performance & Zero-GC Regions
- **Zero-GC Memory Scopes:** Instant pointer bump allocation with 0 ns deallocation cost at frame and scope boundaries.
- **High-Throughput Performance:** Competitive with C and Zig across 12-challenge benchmark suite (see [BENCHMARKS.md](BENCHMARKS.md) for reproducible results).
- **C11 Bare-Metal Code Generator:** Ultra-optimized native binary compilation (`-O3 -funroll-loops -fomit-frame-pointer`).

### 2. 🛡️ Hardware Watchdog & Thermal Safety Guardrails
- **Socket Guard (`std/nexus/socket_guard.end`):** Dual-check socket validation with exponential backoff reducing idle CPU load to < 0.5%.
- **Thermal Circuit Breaker (`std/nexus/circuit_breaker.end`):** Automatic CPU quota load throttling above 85%.

### 3. 🧠 7-Pillar AI-First Cognitive Engine
- **Machine Knowledge Graph (`end graph` / `end query --callers`):** 1ms caller/callee query returning in < 50 tokens.
- **Blast-Radius & Impact Map (`end impact`):** Predictive risk mapping before making codebase edits.
- **Side-Effects & Capability Contracts (`@capability` / `end effects`):** Deterministic purity tracking.
- **Semantic Code Slicing (`end slice`):** 99% token compression for massive files.
- **Structured AST Patches (`end patch`):** Node-level modifications without text breakage.
- **Micro-Isolated Evaluator (`end eval`):** In-memory expression execution in < 50 µs.
- **Architecture Invariants (`end arch check`):** Layer boundary enforcement via `Architecture.toml`.

### 4. 🎮 120 FPS Native Canvas & Framework Ecosystem
- **`EndHyper`:** High-throughput declarative HTTP server framework.
- **`EndForge`:** 120 FPS binary WebSocket multiplayer framework.
- **`EndNexus`:** Game physics and thermal stability engine.
- **`EndCrypto` & `EndKV`:** Constant-time cryptography & zero-alloc key-value tables.

### 5. 🔌 IDE Ecosystem & Extension Suite
- TextMate grammar for decorators (`@get`, `@post`, `@ws`, `@test`, `@capability`).
- Interactive CodeLens (`▶ Run Test`, `🔬 Simulate Mutation`, `⚡ Bench 1M Ops`).
- Visual Studio Webview with 120 FPS interactive particle canvas & What-If sliders.

---

## 📦 Downloads & Verification

| Platform / Asset | File Name | Format |
| :--- | :--- | :---: |
| **Windows x86_64** | `end-v0.4.0-alpha-windows-x64.zip` | `.zip` |
| **Linux x86_64** | `end-v0.4.0-alpha-linux-x64.tar.gz` | `.tar.gz` |
| **macOS Apple Silicon** | `end-v0.4.0-alpha-macos-arm64.tar.gz` | `.tar.gz` |
| **macOS Intel** | `end-v0.4.0-alpha-macos-x64.tar.gz` | `.tar.gz` |
| **SHA256 Checksums** | `SHA256SUMS.txt` | Plain Text |

### ⚡ One-Line Install:
- **Windows:** `irm https://github.com/IrMaho/End/releases/latest/download/install.ps1 | iex`
- **Unix:** `curl -sSf https://github.com/IrMaho/End/releases/latest/download/install.sh | sh`

