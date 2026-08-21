# ??? End Language — Formal Security Threat Model (THREAT_MODEL.md)

This document establishes the threat vectors, trust boundaries, exploit mitigations, and residual risks for the **End Language** toolchain (`endc`), runtime, package ecosystem, and AI cognitive engines.

---

## ?? System Boundaries & Threat Matrix

| Asset / Boundary | Attack Vector | Impact | Mitigation Strategy | Verification / Test |
| :--- | :--- | :--- | :--- | :--- |
| **Compiler Lexer/Parser** | Malformed / recursive source payload | Stack overflow, infinite loop, compiler denial of service | Non-recursive bounded parsing, panic-free tokenization, fuzzing corpus | `end fuzz`, `tests/` |
| **Affine Memory System** | Use-after-move, loan invalidation | Memory corruption, dangling pointers | Compile-time affine move checker (`E0906`), active loan borrowck (`E0907`) | `cargo test` (ir::tests) |
| **Region Allocation** | Pointer escape from arena frame | Access to deallocated stack/arena frame | Strict compile-time Region Escape Analyzer (`E0903`) | Unit & enterprise tests |
| **AI Evaluation Engine** | Malicious expression injection | Sandbox escape, process hijacking | Sandboxed isolated execution environment, memory/time bounds | `end eval` micro-isolation |
| **Structured AST Patch** | Corrupt/malicious node modification | Broken semantics, silent backdoor injection | Pre-patch AST validation, semantic typecheck, automated rollback | `end patch` validation |
| **Concurrency Primitives** | Data race on shared memory channels | Race conditions, memory corruption | C11 Atomic Arc (`__atomic_fetch_add/sub`), Spinlock mutex channels | `c_backend.rs` atomic tests |
| **Package Supply Chain** | Tampered packages, dependency confusion | Malicious dependency execution | Cryptographic SHA-256 validation, SAT solver lockfiles | `SatDependencySolver` |
| **Release Artifacts** | Man-in-the-middle binary tampering | Compromised developer toolchain | SHA256SUMS manifests, GitHub Release TLS asset verification | `release.yml` SHA256 pipeline |

---

## ?? Residual Risk & Ongoing Hardening

1. **Bare-Metal Freestanding Targets:** Current production target is hosted OS (Linux/Windows/macOS) with standard C runtime library. Embedded freestanding kernels are on the roadmap.
2. **Dynamic Unsafe C FFI:** Foreign C functions invoked via `@import_c` are marked unsafe and must be wrapped in safe End contracts.
