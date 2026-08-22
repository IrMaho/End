# 🏛️ End Language — Compiler Architecture & Implementation Specification

> **Multi-Tier Lowering Pipeline, Semantic Analysis Engine, C11/LLVM Backends, and Native Runtime.**  
> *A comprehensive technical breakdown of the End Compiler (`endc`), intermediate representations, and native code generation pipeline.*

---

## 1. The Multi-Tier Compiler Pipeline

The End compiler is built in Rust for memory safety and uncompromising compilation speed. It transforms high-level End source code into optimized machine binaries through a structured multi-phase pipeline:

```text
               ┌────────────────────────────────────────┐
               │         End Source Code (.end)         │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │           Lexer & Tokenizer            │
               │         (endc/src/lexer.rs)            │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │        Recursive-Descent Parser        │
               │         (endc/src/parser.rs)           │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │         Abstract Syntax Tree (AST)     │
               │          (endc/src/ast.rs)             │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │       Semantic Analysis Engine         │
               │      • Region Escape Analysis (E0302)  │
               │      • Static Borrow Checker (E0382)   │
               │      • Capability & Invariant Auditor  │
               │      • Agent Contract Verifier         │
               │      • End Intelligence Platform (EIP) │
               │        (DNA, ImpactGuard, DEC_v2, DRM, │
               │         SemanticGit, SkillVerifier)    │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │ High-Level & Mid-Level IR (HIR / MIR)  │
               │         (endc/src/ir/mod.rs)           │
               └───────────────────┬────────────────────┘
                                   │
                   ┌───────────────┴───────────────┐
                   ▼                               ▼
       ┌────────────────────────┐      ┌────────────────────────┐
       │   C11 Native Backend   │      │  LLVM Direct Backend   │
       │(endc/src/codegen/c_...)│      │      (Alpha 🟠)        │
       └───────────┬────────────┘      └───────────┬────────────┘
                   │                               │
                   └───────────────┬───────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │ Native Linker (zig cc / clang / gcc)   │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │ Stripped Native Binary (40 KB exe/dll) │
               └────────────────────────────────────────┘
```

---

## 2. Compiler Subsystems & Source Organization

| Subsystem | Source Location | Responsibility |
| :--- | :--- | :--- |
| **Lexer & Tokens** | [`endc/src/lexer.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/lexer.rs) | Tokenizes Unicode keywords, time literals (`50ms`), operators (`>>`, `&`, `\|`), and contract tokens. |
| **Parser** | [`endc/src/parser.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/parser.rs) | Parses statements, operations, event hubs, expressions, and agent contracts without ambiguity. |
| **AST Definitions** | [`endc/src/ast.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/ast.rs) | Typed representation of modules, functions, structs, enums, statements, and contracts. |
| **Semantic Analyzer** | [`endc/src/semantic/analyzer.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/semantic/analyzer.rs) | Type resolution, ownership tracking, region lifetime checks, and invariant validation. |
| **IR & Lowering** | [`endc/src/ir/mod.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/ir/mod.rs) | SSA form generation, constant propagation, dead-code elimination, and control-flow graphs. |
| **Interpreter VM** | [`endc/src/codegen/interpreter.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/codegen/interpreter.rs) | Fast development execution, REPL sandbox, and contract verification engine. |
| **C11 Code Generator** | [`endc/src/codegen/c_backend.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/codegen/c_backend.rs) | Transpiles AST/MIR directly to ultra-portable, optimized C11 code. |
| **Unit Test Suite** | [`endc/src/ir/tests.rs`](file:///c:/Users/ASUS/Desktop/flutter_project/end/endc/src/ir/tests.rs) | 41 automated test suites validating language semantics, algebraic operations, and contracts. |

---

## 3. C11 Bare-Metal Backend & FFI Architecture

End compiles natively to clean, standards-compliant **C11 source code** which is compiled with `-O3 -march=native -flto -funroll-loops -fomit-frame-pointer` via `zig cc`, `clang`, or `gcc`.

### C-ABI Native Export:
```end
@c_export
pub fn calculate_vector_distance(x1: f64, y1: f64, x2: f64, y2: f64) f64 {
    val dx = x2 - x1
    val dy = y2 - y1
    ret sqrt_f64((dx * dx) + (dy * dy))
}
```

Compiling with `end build math.end --dll` produces:
1. `math.dll` / `math.so` / `math.dylib` (native shared object)
2. `math.h` (C-compatible header file with automatic `END_API` exports)

---

## 4. Hardware Yielding & CPU Watchdog Runtime

To guarantee that high-throughput network engines (`EndHyper`) and real-time game loops (`EndNexus`) never spin-lock or overheat the CPU, the End runtime injects native platform yield instructions:

```c
#if defined(_WIN32)
  #define END_CPU_YIELD() SwitchToThread()
  #define END_CPU_RELAX() YieldProcessor()
#elif defined(__linux__) || defined(__APPLE__)
  #define END_CPU_YIELD() sched_yield()
  #define END_CPU_RELAX() __builtin_ia32_pause()
#else
  #define END_CPU_YIELD() ((void)0)
  #define END_CPU_RELAX() ((void)0)
#endif
```

---

## 5. Architectural Invariant Engine (`Architecture.toml`)

End features a native architectural boundary validator. Projects define boundary rules in `Architecture.toml`:

```toml
[invariants]
"std/nexus/**"     = { pure_math_only = false, allowed_effects = ["net", "io", "alloc"] }
"std/simulation/**"= { cannot_import = ["std/nexus/socket_guard.end"], pure_math_only = true }
"std/crypto/**"    = { cannot_import = ["std/ui/**", "std/forge/**", "std/net/**"] }
"std/math/**"      = { pure_math_only = true, cannot_import = ["std/net/**", "std/db/**"] }
```

Running `end arch check --json` validates all layer boundaries in < 5 ms, preventing architectural degradation during automated AI pair programming.
