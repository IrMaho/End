---
name: end-language
description: Comprehensive expert guide for developing applications, web servers, 120 FPS native GUI applications, shared libraries (DLL), and high-performance algorithms in the End Programming Language.
---

# End Programming Language: Master Agent Reference & Architecture Guide

> **The End Programming Language (`end.exe` / `endc.exe`)** is an ultra-fast, zero-overhead, AI-first systems language combining:
> 1. **50 Revolutionary Syntactic Innovations:** Morphic segmental functions (`{platform}_send()`), quantum unwrapping (`val!`), scoped invariants (`@invariant`), declarative UI (`@widget`), and multi-export bindgen (`@export`).
> 2. **Modular Vibe-Coding & Polymorphism:** Zero-boilerplate module contracts (`derives`), invariant validation, and cross-module bridges.
> 3. **Native Declarative UI (`EndUI`):** 120 FPS declarative GUI syntax (`@widget`), zero external latency, and interactive DevMode overlay.
> 4. **Bidirectional AI Agent Feedback Protocol:** Direct gallery mockup upload, visual pin annotations, and CLI reply bridge (`end agent list` / `end agent reply`).
> 5. **Universal Technical Passport (`end doc`):** Auto-generated OpenAPI 3.1 (`openapi.json`), AI Technical Passport (`project_passport.json`), and Swagger documentation.
> 6. **Native Flutter / Dart FFI Bridge (`end flutter`):** Zero-overhead Dart FFI bindgen for mobile/cross-platform deployment with 0 LSP diagnostics.
> 7. **3-Tier Hybrid Memory:** Bare-metal execution (Arena + Zero GC Region + Box + Rc/Arc) beating Zig/C/Rust/Go.

---

## 1. 🛑 The Golden Law of End Architecture (Where Code Lives)

To prevent developer and AI Agent confusion, always adhere to strict separation between **Source Code** and **Compiler Build Artifacts**:

```text
my_end_project/
│
├── 🌟 src/                             <=== ONLY WRITE SOURCE CODE HERE (100% .end files)
│   ├── main.end                        <--- Domain logic, structs, invariants, and entry point
│   └── ui_dashboard.end                <--- Declarative UI widgets (@widget) in End syntax
│
├── 📦 bin/                             <--- [AUTO-GENERATED] Native machine binaries (.exe, .dll)
├── 🌐 ui/                              <--- [AUTO-GENERATED] 120 FPS Web/Desktop DOM (end ui)
├── 📱 flutter_lib/                     <--- [AUTO-GENERATED] Dart FFI bridge for Flutter (end flutter)
├── 📄 docs/                            <--- [AUTO-GENERATED] OpenAPI & AI Passport (end doc)
└── 🤖 .end/agent_feedback/             <--- [PERSISTENT] DevMode pin annotations & mockup images
```

---

## 2. The 50 Revolutionary Syntactic Innovations Overview

| Domain | Key Syntaxes | Advantage Over C++/Rust/Python/Go |
| :--- | :--- | :--- |
| **1. 🏗️ Morphic Meta-Identifiers** | `fn {platform}_send()`, `struct {target}_Client` | Write 1 function/struct, auto-morph into infinite platform specializations. |
| **2. ⚡ Quantum State & Invariants** | `val! x = fallible() ?? fallback`, `@invariant(...)` | Zero-branch unwrapping and compile-time verified mathematical invariants. |
| **3. 🌐 Universal Protocol Synthesis** | `interaction Bot { state ... }`, `stream ||> sink` | Platform-agnostic bot and stream processing without hardcoded APIs. |
| **4. 🧠 AI & Agent Native Coding** | `@intent("...")`, `@ai_parse(...)`, `healable { ... }` | Direct semantic contracts and self-healing execution inspected by AI agents. |
| **5. 📊 Math & Financial Precision** | `10355.00d`, `120[km/h]`, `ledger_entry { ... }` | Zero-drift 128-bit decimal arithmetic and compile-time unit checking. |
| **6. 🔀 Concurrency & Fibers** | `spawn_group { fork t1(); }`, `counter <+= 1` | Structured concurrency with cancellation and bare-metal atomic operators. |
| **7. 🎨 Declarative UI & DevMode** | `@widget`, `@pin(id, task)`, `state count = 0` | 120 FPS reactive DOM generation with visual canvas annotation overlay. |
| **8. 🔒 Security & Zero-Trust** | `tainted str`, `secret password`, `quarantine { ... }` | Compile-time SQL/XSS prevention and auto-redacted secret variables. |
| **9. 🔄 Polymorphic Metaprogramming** | `@comptime`, `derives [Json, Eq]`, `shape HasId` | Zero-runtime-cost compile-time evaluation and structural duck contracts. |
| **10. 🚀 Seamless Multi-Target FFI** | `@export(C, Flutter, WASM)`, `simd<f32, 8>` | Single-codebase automated multi-ecosystem SDK generation and AVX-512 SIMD. |

---

## 3. CLI Toolchain Command Reference

```bash
# 1. Compilation & Native Execution
end.exe run src/main.end                          # Execute immediately via VM Interpreter
end.exe build src/main.end -o bin/app.exe         # Compile to ultra-optimized machine code (-O3)

# 2. Native Declarative UI & DevServer (EndUI)
end.exe ui src/main.end -o ui/ --dev --serve --port 8765 --open  # Launch live 120 FPS DevServer + AI Overlay

# 3. Bidirectional AI Agent Feedback Protocol
end.exe agent list                                # List all open developer pins, tasks, and attached mockups
end.exe agent reply --id <id> --message "<msg>" --status Resolved  # AI Agent replies and marks task resolved

# 4. Universal Documentation & AI Technical Passport
end.exe doc src/main.end -o docs/                 # Generates openapi.json, project_passport.json, Swagger UI

# 5. Flutter / Dart FFI Bridge Generator
end.exe flutter bindgen src/main.end -o flutter_lib/  # Generates end_flutter_bridge.dart
```
