# 🧠 End Language — Mid-Level IR (MIR) Optimization Pipeline
## Mem2Reg SSA Promotion, Function Inlining, Constant Propagation & Dead Code Elimination

---

## 🌟 MIR Pipeline Overview

The End MIR Optimizer (`endc/src/ir/optimizer.rs`) performs high-impact transformations on the control flow graph before handing off to the backend code generators:

```text
       AST Module ──► HIR ──► MIR Control Flow Graph
                                    │
                                    ▼
       ┌──────────────────────────────────────────────────────────┐
       │                  MIR Optimizer Passes                    │
       │  1. Mem2Reg: Promotes stack allocas to SSA registers     │
       │  2. Inliner: Eliminates function call overhead           │
       │  3. ConstantFolder: Algebraic compile-time evaluation    │
       │  4. LICM: Hoists loop invariants to pre-headers          │
       │  5. DCE: Prunes dead basic blocks & unreached code       │
       └────────────────────────────┬─────────────────────────────┘
                                    │
                                    ▼
                          Optimized Machine Codegen
```

---

## 🚀 Optimization Passes

| Pass | Description | Impact |
|------|-------------|--------|
| **Mem2Reg** | Replaces `alloca`/`store`/`load` patterns with direct SSA registers | Eliminates redundant memory traffic |
| **Inlining** | Inlines small leaf functions and `@inline` procedures | 0-cost function call abstraction |
| **Constant Folding** | Evaluates integer/float arithmetic at compile time | Reduces runtime cycle count |
| **LICM** | Hoists loop-invariant calculations before loop headers | Speeds up inner loops |
| **DCE** | Removes unreachable basic blocks from CFG | Minimizes binary size |
