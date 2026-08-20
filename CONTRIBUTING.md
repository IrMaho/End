# Contributing to End Language 👑

Thank you for your interest in contributing to **End**, the native programming language for Autonomous AI Agents and Software Architects!

---

## 🌟 The 12 Immutable Principles of End Core

All contributions to the End language compiler (`endc`) and standard library must adhere to these 12 core design principles:

1. **Native:** Compiles to zero-overhead machine code (no forced Garbage Collection).
2. **Fast:** Millisecond compilation and check turnaround times.
3. **Readable:** Syntax optimized for humans as reviewers/architects.
4. **Explicit:** No hidden control flow or implicit type conversions.
5. **Deterministic:** 100% predictable execution behavior.
6. **Modular:** "Everything is a Module" with micro-modularity support.
7. **Composable:** Single mental model across UI, systems, and backend.
8. **Zero-Cost Abstraction:** What isn't used isn't paid for at runtime.
9. **Compile-Time Programmability:** First-class metaprogramming.
10. **Machine/Agent Inspectable:** Machine-first introspection (END-AIP).
11. **Hardware Accessible:** First-class CPU/GPU intrinsics, assembly, and register-level control.
12. **Extensible:** Allows custom domain profiles without breaking the core language semantics.

---

## 🛠️ Development Setup

### Prerequisites
- **Rust Toolchain:** `rustc` and `cargo` (edition 2024 / latest stable)
- **C Compiler (Optional for native build):** `zig`, `clang`, or `gcc`

### Building from Source
```bash
cd endc
cargo build
```

### Running Tests and Verification
```bash
cargo run -- run ../examples/hello.end
cargo run -- inspect ../examples/server.end:18
cargo run -- check ../examples/server.end --json
```

---

## 🤝 How to Contribute

1. **Fork the Repository** on GitHub.
2. **Create a Feature Branch:** `git checkout -b feature/amazing-feature`
3. **Commit your Changes:** `git commit -m 'feat: Add new AST node for compile-time constants'`
4. **Push to the Branch:** `git push origin feature/amazing-feature`
5. **Open a Pull Request!**
