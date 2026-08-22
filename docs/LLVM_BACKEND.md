# ⚡ End Language — Production-Grade LLVM Backend Specification
## Native IR Generation, 4-Tier Memory Model Lowering, and Direct Executable Synthesis

---

## 🌟 Overview & Architecture

The End LLVM Backend (`endc/src/codegen/llvm_backend.rs`) compiles End AST directly into standardized, optimized LLVM Intermediate Representation (LLVM IR `.ll` and bitcode `.bc`), eliminating any intermediate C compiler dependency.

```text
               ┌────────────────────────────────────────┐
               │         End Source Code (.end)         │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │        Semantic & Type Analysis        │
               │   • Static Borrow Checker (noalias)    │
               │   • Region Escape Auditor (Arenas)     │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │          LLVM Backend Engine           │
               │  • SSA Form with PHI & Merge Blocks    │
               │  • 4-Tier Deterministic Memory Lowering│
               │  • Tagged Union Enum Switch Tables     │
               │  • SIMD Vector Intrinsics (<4 x float>)│
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │            LLVM IR (.ll)               │
               │        target triple = "..."           │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │         llc / opt / lld-link           │
               │     Optimized Standalone Binary        │
               └────────────────────────────────────────┘
```

---

## 🛡️ 4-Tier Memory Model Lowering to LLVM

| End Memory Tier | LLVM IR Lowering Pattern |
|-----------------|--------------------------|
| **Tier 0: Ephemeral Leases (`lease`)** | `alloca` inside scoped block + `@llvm.lifetime.start.p0i8` & `@llvm.lifetime.end.p0i8` intrinsics for deterministic 0-cost stack reuse. |
| **Tier 1: Region Arenas (`region`)** | `%struct.EndArena*` + `@end_arena_create(65536)` / `@end_arena_destroy` with $O(1)$ linear bump allocation GEPs. |
| **Tier 2: Static Borrows (`&T` / `&mut T`)** | Exclusive pointer attributes: `noalias`, `nonnull`, `align 8`. |
| **Tier 3: Unsafe Pointers (`*T`)** | Direct unmanaged `i8*` pointers for FFI and kernel driver routines. |

---

## 🔍 CLI Usage & Flags

```bash
# Build standalone binary with LLVM backend:
$ end build main.end --backend llvm -o app.exe

# Dump generated LLVM IR text (.ll):
$ end build main.end --dump-llvm-ir
✔ Generated direct LLVM IR at "main.ll"
```
