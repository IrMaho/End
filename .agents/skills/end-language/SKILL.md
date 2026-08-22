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

---

## 4. 🚀 50 First-Class Semantic & AI-Native Language Syntaxes

The End language features 50 revolutionary, first-class language syntaxes processed natively across all compiler stages (Lexer ➔ Parser ➔ AST ➔ Semantic Analyzer ➔ HIR/MIR ➔ C/LLVM Backend ➔ Interpreter VM ➔ LSP / Agent Protocol):

### A. Intent, Formal Verification & Rationale (1-10)
1. **`intent "<goal>" { goal: "...", preserve: [...] }`** - First-class declarative specification block defining optimization objectives and non-negotiable invariants.
2. **`intent diff { preserve: [...], change: [...] }`** - Explicit contract specifying boundaries between permitted refactoring and strictly preserved behaviour.
3. **`prove <condition>;`** - Formal proof obligation checked by the verifier / runtime contract engine.
4. **`assume <condition>;` / `assume { ... }`** - Optimization assumptions enabling dead code elimination and zero-cost branch elimination.
5. **`guarantee <condition>;`** - Formal postcondition assurance enforced on function exit.
6. **`invariant <condition>;`** - Continuous invariant that must hold true across all state transitions.
7. **`verify { <cond1>, <cond2>, ... }`** - Comprehensive multi-clause contract verification suite.
8. **`because "<rationale>";`** - In-language rationale anchoring for compiler optimizations and AI agent decisions.
9. **`why <target> { "<rationale>" }`** - Targeted architectural rationale binding.
10. **`explain { "<topic>": "<rationale>" }`** - Self-documenting, queryable semantic metadata block.

### B. Memory, Protection & Domain Ownership (11-16)
11. **`protect { ... }`** - Hardware-isolated memory region guarding critical state.
12. **`frozen <symbol>;`** - Immutable symbol locking preventing any subsequent mutations or agent overrides (`E0908`).
13. **`mutable_by <role1>, <role2>;`** - Role-based authorization policy governing who or what can mutate state.
14. **`owned <name>: <type> = <init>;`** - Single-owner linear memory declaration with compile-time borrow/move checking.
15. **`handoff <resource> -> <domain>;`** - Zero-copy resource ownership transfer across execution domains.
16. **`return_to <domain> <resource>;`** - Explicit ownership restitution back to the host domain.

### C. Heterogeneous Compute & Resilient Concurrency (17-30)
17. **`compute <target> { ... } fallback <target> { ... }`** - Hardware-agnostic compute dispatch across CPU, GPU, TPU, and FPGA.
18. **`race_free { ... }`** - Compiler-proven data-race-free concurrent execution region.
19. **`order: <mode>;`** - Memory ordering constraint declaration (`sequential`, `relaxed`, `acquire_release`, `stable`).
20. **`deterministic { ... }`** - Floating-point associative stability and deterministic thread scheduling lock.
21. **`replay { ... }`** - Deterministic execution trace logging for zero-overhead debug replay.
22. **`checkpoint <name>;`** - Instantaneous execution state snapshot.
23. **`rollback to <name>;`** - Reversible state rollback to a prior checkpoint.
24. **`transaction { ... }`** - ACID atomic state transaction block with automatic abort-on-error.
25. **`speculative { ... }`** - Isolated branch speculation committed only on successful completion.
26. **`fallback <target> { ... }`** - Explicit contingency execution pipeline.
27. **`parallel choose { <branch> => { ... }, ... }`** - First-available multi-branch parallel selection.
28. **`race { { ... }, { ... } }`** - Concurrent race execution where the fastest branch wins and cancels remainder.
29. **`hedge after <delay_ms> { ... } fallback { ... }`** - Latency-hedging request pattern for P99 optimization.
30. **`cancel_safe { ... }`** - Asynchronous region guaranteed to clean up and roll back on external cancellation.

### D. Budgets, Deadlines & Adaptive SLAs (31-36)
31. **`budget { cpu: "...", memory: "..." } { ... }`** - Hard runtime SLA ceiling with automatic yielding on budget exhaustion.
32. **`deadline "<duration>" { ... }`** - Strict latency boundary preventing frame drops and thread starvation.
33. **`priority <level> { ... }`** - Operating system and thread-pool scheduling priority hint (`realtime`, `high`, `normal`, `idle`).
34. **`quality { min: "...", max_latency: "..." } { ... }`** - Dynamic fidelity tuning under varying hardware load.
35. **`tradeoff { prefer: "...", sacrifice: "..." } { ... }`** - Explicit Pareto tradeoff directive for compiler optimization passes.
36. **`adapt { if <c1> => { ... }, ... }`** - Self-tuning multi-path adaptive dispatch based on live runtime telemetry.

### E. Observability & Reactive Data Flow (37-41)
37. **`observe <metric1>, <metric2>;`** - Zero-overhead runtime telemetry and telemetry extraction hook.
38. **`watch <target> { on <event> => { ... } }`** - Reactive state mutation observer with sub-microsecond event dispatch.
39. **`react to <condition> { ... }`** - Event-driven reactive execution trigger.
40. **`stream <source> { <op1>, <op2>, ... }`** - Zero-allocation fused reactive stream pipeline.
41. **`flow { <step1>, <step2>, ... }`** - Declarative directed acyclic data-flow pipeline.

### F. AI Agent Protocol & Code Evolution (42-50)
42. **`agent <Name> { scope: "...", goal: "...", constraints: [...] }`** - Machine-readable agent charter and boundary contract.
43. **`task <Name> { ... }`** - Atomic units of work assignable and verifiable by AI agents.
44. **`accept { [...] }`** - Formal acceptance criterion required before agent patches are applied.
45. **`reject if { [...] }`** - Automated rejection guardrail preventing performance regressions or breaking changes.
46. **`baseline { <metric>: "<val>", ... }`** - Immutable historical performance baseline anchor.
47. **`regression { "<condition>" }`** - Automated regression assertion evaluated against established baselines.
48. **`context <Name> { include: [...], exclude: [...] }`** - AI Agent prompt window context filter for token-efficient reasoning.
49. **`slice <Name> { from: ..., include: [...], exclude: [...] }`** - Semantic AST code slicing for focused module editing.
50. **`patch <Target> { ... }` & `evolve <Target> { ... }`** - Automated, formal-contract-backed codebase evolution and AST patching engine.

