# ⚡ End Language — Cranelift JIT Backend Specification
## Real-Time CLIF Lowering, Sub-Millisecond JIT Execution, and REPL Integration

---

## 🌟 Overview & Architecture

The End Cranelift Backend (`endc/src/codegen/cranelift_backend.rs`) provides sub-millisecond compilation and execution by translating End AST into Cranelift Intermediate Format (CLIF) and invoking functions directly in memory.

```text
               ┌────────────────────────────────────────┐
               │         End Source Code (.end)         │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │        Cranelift Backend Engine        │
               │   • Fast SSA Construction              │
               │   • CLIF Translation (iadd, brif, etc) │
               │   • Stack Slot & Variable Binding      │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │           Cranelift IR (CLIF)          │
               │  target x86_64 has_sse2 has_avx2       │
               └───────────────────┬────────────────────┘
                                   │
                                   ▼
               ┌────────────────────────────────────────┐
               │          Native Executable Memory      │
               │   Instant JIT Execution (< 1,000 µs)   │
               └────────────────────────────────────────┘
```

---

## 🔍 CLI Usage & Flags

```bash
# Run file using Cranelift JIT engine:
$ end run main.end --backend cranelift
⚡ Cranelift JIT: JIT compiled 4 functions in 420 µs (Zero C Dependency)
  ├─ Status: JIT_READY
  ├─ Entry: 0x000000007fff4000
  └─ Note: Module compiled to native Cranelift JIT memory space with sub-millisecond execution.

# Dump Cranelift CLIF IR to .clif file:
$ end build main.end --dump-cranelift-clif
✔ Dumped Cranelift CLIF IR at "main.clif"
```
