<div align="center">

<p align="center">
  <img src="assets/logo.png" alt="The End Programming Language" width="280" style="border-radius: 16px; box-shadow: 0 8px 32px rgba(0,0,0,0.3);" />
</p>

# 👑 The End Programming Language
### One Language. Every Domain.
**Native Performance • Memory Safety • Modularity • AI-Native Development**

[![CI Status](https://github.com/IrMaho/End/workflows/End%20CI/badge.svg)](https://github.com/IrMaho/End/actions)
[![Release](https://img.shields.io/github/v/release/IrMaho/End?color=58a6ff&label=Release)](https://github.com/IrMaho/End/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-3fb950.svg)](LICENSE)
[![Memory](https://img.shields.io/badge/Memory-4--Tier%20Deterministic-56d4dd.svg)](docs/MEMORY_MODEL.md)
[![Agent-Native](https://img.shields.io/badge/AI--Native-50%20Contracts%20%26%20Proof%20of%20Work-f0883e.svg)](docs/AGENT_CONTRACT_SYSTEM.md)
[![FPS](https://img.shields.io/badge/V--Sync-120%20FPS%20Canvas-bc8cff.svg)](std/ui)

<p align="center">
  <a href="#-what-is-end">What is End?</a> •
  <a href="#-why-end">Why End?</a> •
  <a href="#-one-language-every-domain">Every Domain</a> •
  <a href="#-core-pillars">Core Pillars</a> •
  <a href="#-code-examples">Code Examples</a> •
  <a href="#-feature-maturity-matrix">Features</a> •
  <a href="#-12-challenge-performance-matrix">Benchmarks</a> •
  <a href="#-quick-install">Install</a> •
  <a href="docs/VISION.md">Vision</a>
</p>

---

</div>

> **«End is a high-performance, memory-safe, modular, general-purpose programming language designed from the ground up for AI-assisted and agentic software development.»**  
> *From embedded systems and native applications to games, backends, tooling, graphics, distributed systems, and AI infrastructure — End aims to give developers one coherent language and toolchain instead of forcing them to switch languages for every domain.*

---

## 💡 What is End?

**End is a modern general-purpose programming language engineered for the era of AI pair programming and high-performance computing.**

Historically, software engineering has forced developers to switch programming languages whenever their application domain changes: C/C++/Rust for systems and game engines, Go/Java/C# for cloud backends, Swift/Kotlin for mobile, Python for AI orchestration, and TypeScript for web applications.

**End eliminates this artificial fragmentation.** It combines bare-metal execution speed, a 4-tier deterministic memory model with zero mandatory garbage collection pauses, algebraic composability, and a native compiler toolchain specifically designed to be machine-readable and machine-verifiable by AI coding agents.

---

## 🎯 Why End?

Modern developers are constantly forced to choose between rigid trade-offs:
- Want raw performance and hardware control? *Sacrifice memory safety and developer velocity (C/C++).*
- Want memory safety without garbage collection? *Sacrifice compilation speed and accept steep cognitive lifetime gymnastics (Rust).*
- Want simplicity and fast builds? *Sacrifice expressive immutability and accept runtime GC pauses (Go/Zig).*
- Want rapid development velocity? *Sacrifice execution speed and accept massive runtime bloat (Python/Node.js).*

### The End Thesis:

> **«Why choose one trade-off when a language can be designed around all of them? By selectively adopting the strongest ideas across programming history and redesigning them around one coherent model, End reduces these historical compromises into a single, unified language.»**

---

## 🚫 What End Is NOT

To maintain absolute technical clarity:

| What End Is NOT | What End Actually IS |
| :--- | :--- |
| ❌ **NOT only a game programming language** | ✅ **A general-purpose language** equally suited for cloud backends, microservices, databases, CLI tools, and OS components. |
| ❌ **NOT only a systems programming language** | ✅ **A full-spectrum platform** providing high-level ergonomics (`EndHyper`, `EndNexus`), declarative routing, and rich frameworks. |
| ❌ **NOT a Rust or Zig clone** | ✅ **A unique language** featuring a 4-Tier Memory Model (Regions + Ephemeral Leases + Borrowing) and native AI Agent Contracts. |
| ❌ **NOT an AI wrapper or chatbot toy** | ✅ **A compiled, statically-typed native language** with its own lexer, recursive-descent parser, AST, semantic analyzer, and C11/LLVM backends. |
| ❌ **NOT a compiler experiment** | ✅ **A production-tracked toolchain** with 41+ passing unit test suites, 12 reproducible benchmarks, and stripped 40 KB native binaries. |

---

## 🌐 One Language. Every Domain.

End is explicitly designed to cover the complete software development spectrum without requiring domain-specific language switches:

| Domain | End Target Goal | Status | Key Architectural Mechanism |
| :--- | :--- | :---: | :--- |
| **Backend & APIs** | Native high-performance web services and microservices | 🟢 Stable | `EndHyper`, zero-copy HTTP routing, connection pooling |
| **Web Full-Stack** | Full-stack services, binary WebSocket streams | 🟢 Stable | `EndForge`, low-latency binary framing, typed DTOs |
| **Memory-Safe Systems** | OS components, embedded engines, low-level drivers | 🟢 Stable | Deterministic Region Memory, compile-time borrow checking |
| **Real-Time & Games** | 120 FPS game loops, vector math, physics simulation | 🟢 Stable | `std/ui/canvas.end`, SIMD intrinsics, linear memory arenas |
| **Graphics & GPU** | Real-time raymarching, shader pipelines, compute | 🟣 Research | Unified CPU + GPU kernel execution & compute borrowing |
| **AI Infrastructure** | Tensor computing, model inference, agent runtimes | 🟢 Stable | `AI_TENSORS`, GGUF engine integration, tensor primitives |
| **Developer Tools** | Compilers, linters, analyzers, instant CLIs | 🟢 Stable | Stripped 40 KB native binaries, instant sub-ms startup |
| **Data Systems & DBs** | In-memory key-value engines, disk storage layers | 🔵 Beta | `EndKV`, 64-byte aligned arena tables, WAL logging |
| **Distributed Systems** | Raft consensus, message-passing nodes, replication | 🟡 Experimental | `RAFT_DISTRIBUTED_CONSENSUS`, cluster state machines |
| **Desktop Applications** | Cross-platform native GUI with high-refresh rendering | 🟡 Experimental | Native C canvas bindings, GLFW/DirectX bridges |
| **Mobile Applications** | Android (NDK) and iOS native binaries | ⚪ Planned | Cross-compilation toolchain targets |
| **WebAssembly (WASM)** | Browser-side client execution without JS glue | ⚪ Planned | WASM standalone backend target generator |

---

## 🏛️ Core Pillars of End

```text
                               ┌────────────────────────────────────────────────────────┐
                               │                    The End Language                    │
                               └───────────────────────────┬────────────────────────────┘
                                                           │
       ┌───────────────────────┬───────────────────────────┼───────────────────────────┬───────────────────────┐
       ▼                       ▼                           ▼                           ▼                       ▼
┌──────────────┐        ┌──────────────┐            ┌──────────────┐            ┌──────────────┐        ┌──────────────┐
│ Performance  │        │ Memory &     │            │ Modularity & │            │ Agent-Native │        │ Developer    │
│  & Native    │        │ Resource     │            │ Packages &   │            │ Contracts &  │        │ Experience & │
│ Compilation  │        │ Safety       │            │ Frameworks   │            │ Verification │        │ Tooling      │
└──────────────┘        └──────────────┘            └──────────────┘            └──────────────┘        └──────────────┘
```

### 1. ⚡ High Performance & Native Compilation
- **Bare-Metal Speed:** Compiles directly to clean, optimized C11 and native machine code with zero runtime overhead.
- **Predictable Latency:** Eliminates stop-the-world garbage collection pauses via linear $O(1)$ bump-allocated region arenas.
- **Ultra-Small Binaries:** Generates standalone, stripped native executables as small as **40.0 KB** with zero external dynamic runtime dependencies.

### 2. 🛡️ 4-Tier Memory & Resource Safety
- **Tier 0: Ephemeral Leases (`lease` & `during`):** Memory buffers and hardware resources bound to scoped blocks with instant zero-cost reclamation.
- **Tier 1: Deterministic Region Arenas (`region`):** Bulk linear allocations resetting in 0 ns upon scope exit with compile-time escape analysis (`E0302`).
- **Tier 2: Static Borrow Checker (`&T` / `&mut T`):** Exclusive mutable references preventing data races and pointer aliasing at build time (`E0382`).
- **Tier 3: Isolated Unsafe Pointers (`*T`):** Raw memory access strictly confined to low-level FFI and device driver routines.
- **Resource-Oriented Programming:** Memory, CPU cores, GPU queues, sockets, files, and DB connections are treated as first-class statically managed resources.

### 3. 🧩 Modularity, Packages & Frameworks
- **First-Class Packages:** Self-contained, capability-bounded packages (`end.config.toml`) discoverable by both humans and AI agents.
- **Integrated Standard Frameworks:**
  - `EndHyper`: High-throughput web server with declarative routing and connection pooling.
  - `EndForge`: Sub-millisecond binary WebSockets for multiplayer gaming and financial feeds.
  - `EndNexus`: 2D/3D physics simulation and thermal CPU load throttling.
  - `EndCrypto`: Constant-time side-channel-resistant cryptographic engine (AES, SHA-256, Ed25519).
  - `EndKV`: Embedded in-memory key-value store with sub-microsecond latency (P99 < 15 ns).

### 4. 🤖 AI-Native Development & 50-Feature Agent Contracts
- **Semantic Cognitive Toolchain:**
  - `end graph`: Sub-millisecond machine knowledge graph of project hierarchy.
  - `end impact`: Blast-radius analyzer computing upstream/downstream dependencies before edits.
  - `end slice`: Compresses 50,000-line codebases into 50-line interfaces (**95% LLM token savings**).
  - `end patch`: Deterministic AST auto-patcher applying node-level modifications safely.
  - `end eval`: Micro-isolated sandbox evaluating expressions in < 50 µs.
  - `end arch`: Enforces architectural invariants defined in `Architecture.toml`.
- **The Agent Contract System (`Intent → Task → Skill → Evidence → Verify`):**
  - **Feature Declarations (`feature`):** Project-level requirements and task checklists.
  - **First-Class Skills (`skill`):** Scoped architectural rules with `hard` and `soft` constraints.
  - **Task State Machine (`task`):** Strict transitions: `planned → claimed → implementing → verified → accepted`.
  - **Requirement Traceability:** Unbroken proof chain: `requirement` → `implements` → `verifies`.
  - **Executable TODOs (`todo`):** Compiler-enforced TODOs that fail the build if left incomplete.
  - **Proof-of-Work Verification:** Replaces unverified AI claims with machine-audited evidence.

---

## 💻 Code Examples

### 1. High-Performance Web Endpoint with Region Scoping (`EndHyper`)
```end
import "std/hyper/router.end"
import "std/hyper/response.end"

st PaymentDto {
    customer_id: i64,
    amount: i64,
    currency: str,
}

@post("/api/v1/payments")
@capability(net = true, disk = false, memory = "ArenaScoped")
pub fn process_payment_handler(req: RequestContext, payment: PaymentDto) HyperResponse {
    region request_scope {
        val confirmation_code = "TX-99482";
        val response_body = "{\"status\": \"approved\", \"tx\": \"" + confirmation_code + "\"}";
        ret hyper_response_json(200, response_body);
    } // Memory reset instantly in 0 ns at scope exit!
}
```

### 2. First-Class Operation Values with Resilient Algebra
```end
operation validate_order(items: i64) -> i64 {
    requires: items > 0;
    guarantee: result == items;
    ret items;
}

operation charge_card(items: i64) -> i64 {
    requires: items > 0;
    val price_per_item = 50;
    ret items * price_per_item;
}

pub fn main() void {
    // Pipeline composition with retry and memoization
    val checkout_pipeline = (validate_order >> charge_card).retry(3, 50ms).memoize();
    val result = checkout_pipeline(4);
    
    println("Order total: $" + result.output);
    println("Latency: " + result.duration_ms + " ms");
}
```

### 3. Agent Contract with Proof-of-Work Verification
```end
feature PaymentSecurity {
    requirement: "Transactions must be idempotent and audited";
    skills: ["Idempotent", "TransactionSafe"];
    tasks: ["SEC_01"];
}

skill Idempotent for function {
    rules: ["deterministic_hash", "atomic_deduplication"];
    hard: ["no_double_charge"];
}

task implement_idempotency {
    owner: "autonomous_agent_01";
    status: "planned";
    requirement: "REQ_IDEMP_001";
    implementation: "process_idempotent_tx";
    skills: ["Idempotent"];
    evidence {
        test: "test_replay_attack_prevention";
        prove: "guarantee_single_deduction";
    }
}

claim task implement_idempotency;
complete task implement_idempotency {
    result: "success";
    confidence: 0.99;
    summary: "Idempotency key check implemented with atomic CAS";
    evidence: ["test_replay_attack_prevention", "guarantee_single_deduction"];
    notes: "Zero regressions detected";
}
verify implement_idempotency;
```

---

## 📊 12-Challenge Performance Matrix

*Measurements: Median Latency (P50) across 5 statistical evaluation runs with 2 warmup passes per language on native Windows x86_64.*  
*Data source: [`benchmarks/suite12/suite12_results.json`](benchmarks/suite12/suite12_results.json) — generated by [`run_suite12.py`](benchmarks/suite12/run_suite12.py)*

| Benchmark Challenge | 👑 **End (C11)** | ⚡ **Zig (0.16.0)** | ⚡ **Rust (1.89.0)** | ⚡ **C (GCC 15.2)** | ⚡ **Go (1.25.1)** | Checksum Verification |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **1. 3D SDF Raymarcher (250K Rays)** | **42.55 ms** | 56.50 ms | 56.97 ms | 51.34 ms | 33.54 ms 🥇 | ⚠️ FP Variant ¹ |
| **2. Dynamic Binary Trees (Depth 16 Dynamic)** | **40.97 ms** 🥇 | 339.94 ms | 580.61 ms | 554.73 ms | 447.87 ms | `407713` ✅ |
| **3. HFT Limit Order Engine (1M Orders)** | **27.13 ms** | 25.74 ms | 24.77 ms 🥇 | 29.19 ms | 29.53 ms | `552829538` ✅ |
| **4. SHA-256 Crypto Hashing (500K Blocks)** | **95.33 ms** | 94.99 ms 🥇 | 102.51 ms | 97.05 ms | 116.21 ms | `-4721506799343634759` ✅ |
| **5. N-Body Gravity Orbit (1M Pairwise)** | **700.44 ms** 🥇 | 2221.10 ms | 2287.63 ms | 1878.68 ms | 3275.29 ms | ⚠️ FP Variant ² |
| **6. SPSC Ring Buffer Queue (10M Items)** | **3.80 ms** | 2.60 ms | 1.53 ms | 0.00 ms 🥇 | 8.73 ms | ⚠️ Impl Variant ³ |
| **7. DNA Levenshtein Matrix (1M Cells)** | **1047.25 ms** 🥇 | 2167.75 ms | 2236.03 ms | 1201.59 ms | 2196.40 ms | `525912` ✅ |
| **8. JSON Microservice Serializer (100K)** | **2.12 ms** 🥇 | 8.03 ms | 22.53 ms | 54.81 ms | 41.33 ms | `5588438541400559045` ✅ |
| **9. FSM Lexer Stream (10M Chars)** | **9.87 ms** | 11.81 ms | 13.42 ms | 8.88 ms 🥇 | 18.28 ms | ⚠️ Hash Variant ⁴ |
| **10. GEMM Matrix Multiplication (512x512)** | **10.57 ms** 🥇 | 63.98 ms | 37.00 ms | 10.88 ms | 83.10 ms | `6422836` ✅ |
| **11. Monte Carlo Black-Scholes (2M Paths)** | **41.62 ms** | 29.86 ms 🥇 | 36.67 ms | 44.02 ms | 46.24 ms | `10440246` ✅ |
| **12. Super-Scalar ALU Reduction (10M)** | **658.63 ms** | 797.74 ms | 148.01 ms 🥇 | 638.15 ms | 766.68 ms | `3370198876750320971` ✅ |
| **📦 Binary Footprint (Executable Size)** | 🥇 **40.0 KB** | 835.0 KB | 193.0 KB | 77.4 KB | 1592.5 KB *(1.6MB)* | Stripped Native |

> 📁 **Detailed Benchmark Notes & Reproduction:** Full source code across all 5 languages, compiler flags, and mathematical variance footnotes (¹ ² ³ ⁴) are fully documented in [BENCHMARKS.md](BENCHMARKS.md).

---

## 🏷️ Feature Maturity Matrix

| Capability Category | Feature Name | Status | Documentation Reference |
| :--- | :--- | :---: | :--- |
| **Memory Architecture** | Deterministic Region Arenas (`region`) | 🟢 Stable | [`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md) |
| **Memory Architecture** | Static Borrow Checker & Exclusivity (`&mut`) | 🟢 Stable | [`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md) |
| **Memory Architecture** | Ephemeral Leases (`lease` & `during`) | 🟢 Stable | [`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md) |
| **AI Toolchain** | Semantic Knowledge Graph (`end graph`) | 🟢 Stable | [`docs/AGENT_NATIVE.md`](docs/AGENT_NATIVE.md) |
| **AI Toolchain** | Semantic Slicing & Blast-Radius (`end slice/impact`)| 🟢 Stable | [`docs/AGENT_NATIVE.md`](docs/AGENT_NATIVE.md) |
| **AI Toolchain** | 50-Feature Agent Contract System | 🟢 Stable | [`docs/AGENT_CONTRACT_SYSTEM.md`](docs/AGENT_CONTRACT_SYSTEM.md) |
| **Language Features** | First-Class Operation Values & Algebra (`>>`, `&`)| 🟢 Stable | [`docs/OPERATION_VALUES_AND_ALGEBRA.md`](docs/OPERATION_VALUES_AND_ALGEBRA.md) |
| **Language Features** | Event Hubs, Dispatch (`emit`), and Telemetry | 🟢 Stable | [`docs/OPERATION_VALUES_AND_ALGEBRA.md`](docs/OPERATION_VALUES_AND_ALGEBRA.md) |
| **Frameworks** | EndHyper, EndForge, EndNexus, EndCrypto | 🟢 Stable | [`docs/FRAMEWORKS.md`](docs/FRAMEWORKS.md) |
| **Frameworks** | EndKV In-Memory Key-Value Store | 🔵 Beta | [`docs/FRAMEWORKS.md`](docs/FRAMEWORKS.md) |
| **Tooling & IDE** | VS Code Extension with 120 FPS Sandbox | 🟢 Stable | [`editors/vscode/`](editors/vscode/) |
| **Compiler Backend** | Direct LLVM IR Code Generation Backend | 🟠 Alpha | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) |
| **Distributed** | Raft Consensus & Cluster State Machine | 🟡 Experimental | [`docs/RAFT_DISTRIBUTED_CONSENSUS.md`](docs/RAFT_DISTRIBUTED_CONSENSUS.md) |
| **Compute Model** | Borrowed Processor & GPU Resources (`borrow gpu`)| 🟣 Research | [`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md) |
| **Ecosystem** | Global Decentralized Package Registry | ⚪ Planned | [`docs/PACKAGES.md`](docs/PACKAGES.md) |
| **Target Platform** | WebAssembly (WASM) Standalone Backend | ⚪ Planned | [`docs/ROADMAP.md`](docs/ROADMAP.md) |

---

## 🚀 Quick Install & Getting Started

### Installation on Windows / Linux / macOS:

```bash
# 1. Clone the repository
git clone https://github.com/IrMaho/End.git
cd End

# 2. Build the End Compiler (endc)
cargo build --release --manifest-path endc/Cargo.toml

# 3. Add to system PATH (or run installer script)
# Windows:
.\install.bat

# Linux / macOS:
./install.sh
```

### Create and Run Your First End Project:

```bash
# Create a new project
end new my_project
cd my_project

# Run directly in development mode
end run src/main.end

# Build a stripped native release binary
end build src/main.end --release -o my_app.exe
```

---

## 📚 Complete Documentation Hierarchy

| Topic | Document Reference | Description |
| :--- | :--- | :--- |
| **Canonical Vision** | [`docs/VISION.md`](docs/VISION.md) | The philosophy, thesis, and full-spectrum domain vision of End. |
| **Feature Taxonomy** | [`docs/FEATURES.md`](docs/FEATURES.md) | Complete feature maturity matrix, status classifications, and capabilities. |
| **AI-Native Protocol** | [`docs/AGENT_NATIVE.md`](docs/AGENT_NATIVE.md) | Semantic tools, slicing, vibecoding, and machine-verifiable agent workflows. |
| **Agent Contracts** | [`docs/AGENT_CONTRACT_SYSTEM.md`](docs/AGENT_CONTRACT_SYSTEM.md) | Full 50-feature specification for `Intent → Task → Skill → Evidence → Verify`. |
| **Operation Values** | [`docs/OPERATION_VALUES_AND_ALGEBRA.md`](docs/OPERATION_VALUES_AND_ALGEBRA.md) | First-class operations, algebraic composition, event hubs, and refactoring. |
| **Memory Architecture**| [`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md) | 4-Tier memory model, regions, borrow checking, and resource safety. |
| **Compiler Specs** | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Lowering pipeline, AST, HIR/MIR, C11 backend, and runtime architecture. |
| **Packages & Modules** | [`docs/PACKAGES.md`](docs/PACKAGES.md) | Package manifests, dependency management, capabilities, and module firewalls. |
| **Frameworks** | [`docs/FRAMEWORKS.md`](docs/FRAMEWORKS.md) | EndHyper, EndForge, EndNexus, EndCrypto, EndKV, and UI Canvas. |
| **Roadmap & Gaps** | [`docs/ROADMAP.md`](docs/ROADMAP.md) | Milestone schedule, gap analysis, and prioritized engineering targets. |
| **Project Status** | [`docs/STATUS.md`](docs/STATUS.md) | Truth classification table, evidence inventory, and platform support. |

---

## 🤝 Contributing

We welcome contributions from compiler engineers, language researchers, systems programmers, and AI tool builders!
Please see [CONTRIBUTING.md](CONTRIBUTING.md) for development environment setup, coding guidelines, and pull request procedures.

---

## 📄 License

End is distributed under the [MIT License](LICENSE).  
Copyright © 2026 [Mohammad Javad (IrMaho)](https://github.com/IrMaho) & The End Language Community.
