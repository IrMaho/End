---
name: end-language
description: Comprehensive expert guide and definitive reference manual for writing 100% accurate, error-free End Programming Language code across Pythonic Fluid scripting, feature-oriented architecture, native AI agent/task contracts, modern 120 FPS declarative UI, operation algebra, and bare-metal assembly architectures.
---

# End Programming Language: Master Agent Reference & Architecture Guide

> **The End Programming Language (`endc` / `end`)** is an ultra-fast, zero-overhead, memory-safe, statically typed, general-purpose systems programming language providing **The Full Spectrum of Control**: from **Python-like fluid scripting** for high-velocity coding, to **Feature-Oriented Systems, Native AI Agents & 120 FPS Declarative UI**, down to **Bare-Metal Hardware & Assembly Control**.

---

## 1. 🌟 The 3-Tier Spectrum of Control

When writing code in End (`.end` files), developers and AI agents can choose the exact level of abstraction needed for the task:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                    👑 END: THE FULL SPECTRUM OF CONTROL                     │
├─────────────────────────────────────────────────────────────────────────────┤
│  🐍 LEVEL 1: Pythonic Fluid Mode (Rapid Scripting & High-Velocity Logic)     │
│     • def, let / val, var / mut, pass / skip, and, or, not, None, True, False│
│     • Optional type annotations, string interpolation "{var}", comprehensions │
│     • Pattern matching match, walrus :=, multi-line strings """..."""       │
├─────────────────────────────────────────────────────────────────────────────┤
│  🏛️ LEVEL 2: Feature-Oriented Systems, AI Agents & Enterprise Architecture    │
│     • Feature-Oriented Paradigm: feature X { ... }, module Y { ... }         │
│     • Modern use syntax: use namespace.feature; (NO legacy imports!)         │
│     • Native First-Class AI Agents: agent, task, skill, intent, prove, gate  │
│     • Project Manifest: End.toml / end.toml                                  │
│     • Strict static typing (i64, f64, str, !T, Box<T>, Arc<T>, Channel<T>)   │
│     • 120 FPS Declarative UI (@widget), Web Services (EndNexus)             │
│     • First-Class Operation Algebra (compose, retry, memoize, >>, ||)       │
│     • Reactive Event Topologies (event, hub, derive, topology, debounce)    │
│     • Capability Surfaces (surface, capability, grant, deny, attach, mixin) │
├─────────────────────────────────────────────────────────────────────────────┤
│  ⚡ LEVEL 3: Bare-Metal, Systems & Assembly Control                           │
│     • Ephemeral Resource Leases: lease val buf = alloc(size) { ... }       │
│     • Bump-Allocated Zero-GC Arenas: region FrameArena { ... }              │
│     • Hardware SIMD Vector Registers: f32x8, i32x8, u8x16 (AVX-512 / NEON) │
│     • Direct Inline Assembly: asm "x86_64" { "mov rax, rdi; ret" }          │
│     • Direct Inline C & C ABI Headers: inline_c { ... }, @import_c("<.h>") │
│     • Hardware Atomic Memory Barriers & Raw Pointers: *T, *mut T, <+=       │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 🛑 The Golden Rules of End Code Generation

1. **Use `use` Exclusively:** NEVER use legacy `import "..."`. Always use `use namespace.module;` or `use "relative/path.end"`.
2. **Feature-Oriented Architecture:** Encapsulate domain features using `feature FeatureName { ... }` and modules using `module ModuleName { ... }`.
3. **First-Class AI Agents & Tasks:** Use native `agent AgentName { ... }` and `task TaskName { ... }` blocks for AI-driven workflows.
4. **Project Manifest (`End.toml`):** Every project must have an `End.toml` or `end.toml` specifying package metadata, compiler targets, features, and agent permissions.
5. **Flexible Variable Declarations:** Use `val` or `let` for immutable bindings (default); use `mut` or `var` for mutable variables.
6. **Flexible Functions:** Use `pub fn` for exportable typed functions or `def` for quick Pythonic functions.
7. **Clean String Formatting:** Use native interpolation `"{variable}"` or multiline raw strings `"""..."""` instead of manual string concatenations.
8. **Pattern Matching First:** Use `match expr { ... }` instead of deeply nested, repetitive `if-else` chains.
9. **Zero Memory Leaks:** Wrap scratch buffers in `lease val buf = alloc(...) { ... }` or `region Temp { ... }`.
10. **Declarative UI:** Annotate UI functions with `@widget` and use declarative trees (`Container`, `Column`, `Row`, `Text`, `Button`, `Card`).

---

## 3. Syntax Reference & Templates

### 3.1 🤖 Native AI Agent & Task Contracts
```end
// First-Class AI Agent Contract
agent DevOpsArchitect {
    scope: "src/compiler",
    goal: "Lossless Codebase Evolution & Verification",
    constraints: ["symbol_conservation == 100%", "zero_breaking_changes", "smt_verified"]
}

// First-Class AI Agent Task
task SurgicalRefactorTask {
    owner: "agent-devops-01",
    status: "in_progress",
    requirement: "Lossless decomposition of monolithic parser into cohesive submodules under 500 lines",
    target: "src/compiler/parser.end",
    skills: ["lossless-modular-refactor", "smt-formal-verifier"],
    change_budget: ["max_files: 5", "max_line_delta: 200"]
}
```

### 3.2 🏛️ Feature-Oriented Architecture & `use`
```end
use app.domain.user;
use app.domain.project;
use app.presentation.components;

pub feature ProjectDashboardFeature @version("2.0.0") @owner("core_team") {
    depends: [UserDomain, ProjectDomain];
}

pub fn render_dashboard(p: Project) -> str {
    val card = create_metric_card("Health", "99%", "1.2%", true);
    ret "{card.render()}\n\nActive Project: {p.name}";
}
```

### 3.3 🐍 Pythonic Fluid Mode (Rapid Scripting)
```end
def process_data(items, multiplier = 2):
    val filtered = [x * multiplier for x in items if x > 0];
    
    if not filtered:
        ret {"status": "empty", "total": 0};
        
    var sum = 0;
    for x in filtered:
        sum = sum + x;
        
    ret {
        "status": "ok",
        "count": filtered.len(),
        "total": sum
    };

def generate_summary(name, count, total):
    ret """
    ========================================
    Report for: {name}
    Processed Items: {count}
    Total Value: {total}
    ========================================
    """;
```

### 3.4 ⚡ Bare-Metal Systems & Assembly Control
```end
// Ephemeral Memory Leases (Recycled in 0 ns without GC)
pub fn process_scratch_buffer(size: usize) {
    lease val buffer = alloc(size) {
        buffer[0] = 0x5A;
    }; // Memory is reclaimed instantly here
}

// Bump-Allocated Region Arena
pub fn render_frame() {
    region FrameArena {
        val node = alloc_node();
        paint_node(node);
    }; // Entire FrameArena is bulk-reset instantly
}

// Direct Inline Assembly
pub fn fast_cpu_clock() -> u64 {
    asm "x86_64" {
        "rdtsc"
        "shl rdx, 32"
        "or rax, rdx"
        "ret"
    }
}
```

---

## 4. Project Configuration (`End.toml` / `end.toml`)

```toml
[package]
name = "my_app"
version = "1.0.0"
edition = "2026"

[compiler]
target = "native"
opt_level = 3
strip = true
memory_model = "ephemeral_lease"
smt_proof_gates = true

[architecture]
paradigm = "feature-oriented"
enforce_boundaries = true
cycle_free = true
max_file_lines = 500

[features]
default = ["dashboard", "ai_agent"]
dashboard = { enabled = true, level = "stable" }
ai_agent = { enabled = true, level = "production" }

[agents.DevOpsArchitect]
role = "Autonomous System Architect"
capabilities = ["read_ast", "analyze_impact", "dispatch_events"]
cannot = ["unsealed_declassification"]

[skills]
"lossless-modular-refactor" = { version = "1.0.0", verified = true }
```
