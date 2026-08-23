---
name: end-language
description: Comprehensive expert guide and definitive reference manual for writing 100% accurate, error-free End Programming Language code across Pythonic Fluid scripting, modern enterprise systems, 120 FPS declarative UI, operation algebra, and bare-metal assembly architectures.
---

# End Programming Language: Master Agent Reference & Architecture Guide

> **The End Programming Language (`endc` / `end`)** is an ultra-fast, zero-overhead, memory-safe, statically typed, general-purpose systems programming language providing **The Full Spectrum of Control**: from **Python-like fluid scripting** for high-velocity coding, to **Modern Systems & 120 FPS Declarative UI**, down to **Bare-Metal Hardware & Assembly Control**.

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
│  🏛️ LEVEL 2: Modern Systems & Enterprise Architecture                         │
│     • Strict static typing (i64, f64, str, !T, Box<T>, Arc<T>, Channel<T>)   │
│     • 120 FPS Declarative UI (@widget), Web Services (EndNexus)             │
│     • First-Class Operation Algebra (compose, retry, memoize, >>, ||)       │
│     • Reactive Event Topologies (event, hub, derive, topology, debounce)    │
│     • Capability Surfaces (surface, capability, grant, deny, attach, mixin) │
│     • AI Agent Contracts (agent, task, intent, prove, assume, guarantee)   │
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

1. **Flexible Variable Declarations:** Use `val` or `let` for immutable bindings (default); use `mut` or `var` for mutable variables.
2. **Flexible Functions:** Use `pub fn` for exportable typed functions or `def` for quick Pythonic functions.
3. **Statement Semicolons:** Terminate statements with `;`.
4. **Clean String Formatting:** Use native interpolation `"{variable}"` or multiline raw strings `"""..."""` instead of ugly manual string concatenations (`"a" + "b" + "\n"`).
5. **Pattern Matching First:** Use `match expr { ... }` instead of deeply nested, repetitive `if-else` chains.
6. **No Memory Leaks:** Wrap heavy scratch buffers in `lease val buf = alloc(...) { ... }` or `region Temp { ... }`.
7. **Declarative UI:** Annotate UI functions with `@widget` and use declarative trees (`Container`, `Column`, `Row`, `Text`, `Button`, `Card`).

---

## 3. Syntax Reference & Templates

### 3.1 🐍 Level 1: Pythonic Fluid Mode (Rapid Scripting)
```end
// Pythonic function with optional types and default arguments
def process_data(items, multiplier = 2):
    val filtered = [x * multiplier for x in items if x > 0];
    
    if not filtered:
        ret {"status": "empty", "total": 0};
        
    mut sum = 0;
    for x in filtered:
        sum = sum + x;
        
    ret {
        "status": "ok",
        "count": filtered.len(),
        "total": sum
    };

// Pythonic multi-line formatted reports
def generate_summary(name, count, total):
    ret """
    ========================================
    Report for: {name}
    Processed Items: {count}
    Total Value: {total}
    ========================================
    """;

// Pythonic control flow & walrus assignment
def check_status(queue):
    if (size := queue.len()) > 0 and not queue.is_empty():
        println("Queue has {size} active items.");
    else:
        pass;
```

### 3.2 🏛️ Level 2: Modern Systems & Enterprise Architecture
```end
// 1. Strongly Typed Domain Models
@export(C, Flutter, WASM)
pub struct User {
    pub id: i64,
    pub name: str,
    pub email: str,
    pub balance: f64,
    pub is_active: bool,
}

// 2. Pattern Matching with Expression Return
pub fn evaluate_user_status(user: User) -> str {
    ret match user.is_active {
        true if user.balance > 1000.0 => "VIP Active",
        true => "Standard Active",
        false => "Inactive Account",
        _ => "Unknown"
    };
}

// 3. Dart-Style Cascades (.. and ?..)
pub fn setup_theme() -> PaintConfig {
    val config = PaintConfig()
        ..color = Color.Blue
        ..stroke_width = 3
        ..anti_alias = true;
    ret config;
}

// 4. First-Class Operation Algebra
operation FetchBalance(user_id: i64) -> f64 {
    requires: user_id > 0;
    guarantees: result >= 0.0;
    effects: [DatabaseRead];
    
    ret http_get("/users/{user_id}/balance").parse_f64();
}

val ResilientBalancePipeline = compose {
    retry FetchBalance up to 3;
};

// 5. 120 FPS Declarative UI Widget
@widget
pub fn UserDashboard(user: User) {
    Container {
        padding: 16,
        background: Color.from_hex("#1E1E2E"),
        
        Column {
            Text {
                content: "Welcome, {user.name}",
                font_size: 24,
                color: Color.White,
                font_weight: "Bold"
            },
            Spacer { height: 12 },
            Row {
                Card {
                    title: "Balance",
                    value: "${user.balance}",
                    color: Color.Green
                }
            }
        }
    }
}
```

### 3.3 ⚡ Level 3: Bare-Metal Systems & Assembly Control
```end
// 1. Ephemeral Memory Leases (Recycled in 0 ns without GC)
pub fn process_scratch_buffer(size: usize) {
    lease val buffer = alloc(size) {
        buffer[0] = 0x5A;
    }; // Memory is reclaimed instantly here
}

// 2. Bump-Allocated Region Arena
pub fn render_frame() {
    region FrameArena {
        val node = alloc_node();
        paint_node(node);
    }; // Entire FrameArena is bulk-reset instantly
}

// 3. Vectorized Hardware SIMD Registers (AVX-512 / NEON)
pub fn dot_product_simd(a: []f32x8, b: []f32x8, count: usize) -> f32 {
    mut acc = f32x8.zero();
    for i in 0..<count {
        acc = acc + (a[i] * b[i]);
    }
    ret acc.horizontal_sum();
}

// 4. Direct Inline Assembly
pub fn fast_cpu_clock() -> u64 {
    asm "x86_64" {
        "rdtsc"
        "shl rdx, 32"
        "or rax, rdx"
        "ret"
    }
}

// 5. Direct C Header Imports
@import_c("<math.h>")
pub fn compute_fast_sqrt(val: f64) -> f64 {
    ret sqrt(val);
}
```

---

## 4. Standard Library Cheatsheet

| Module | Purpose | Common Functions / Classes |
| :--- | :--- | :--- |
| `std.net.http` | HTTP/1.1 & HTTP/2 Server & Client | `HttpServer.bind()`, `HttpResponse.ok()`, `HttpResponse.json()` |
| `std.db.postgres` | Native PostgreSQL Client | `PostgresPool.connect()`, `client.query()` |
| `std.db.sqlite` | Embedded SQLite Client | `SqliteDb.open()`, `db.execute()` |
| `std.db.redis` | Redis RESP3 Client | `RedisClient.connect()`, `redis.set_ex()`, `redis.get()` |
| `std.ai.gguf` | Local GGUF LLM Parser | `GgufModel.load_file()` |
| `std.ai.tensor` | Hardware-Accelerated SIMD Tensors | `Tensor.matmul_simd()`, `Tensor.softmax()` |
| `std.ui` | 120 FPS Declarative Widgets | `Container`, `Column`, `Row`, `Text`, `Button`, `Card` |
| `std.crypto.jwt` | JSON Web Tokens | `Jwt.sign()`, `Jwt.verify()` |
| `std.crypto.argon2` | Secure Password Hashing | `Argon2id.hash()`, `Argon2id.verify()` |
| `std.async` | Lightweight Fiber Concurrency | `FiberGroup.create()`, `group.spawn()`, `group.wait_all()` |

---

## 5. Toolchain CLI Commands

```bash
# Instant execution via VM
endc run src/main.end

# Compile native stripped binary with -O3
endc build src/main.end -o bin/app.exe --strip

# Compile dynamic library (.dll / .so / .dylib)
endc build src/main.end --dll -o bin/libcore.dll

# Launch 120 FPS UI DevServer + AI Canvas
endc ui src/main.end --dev --serve --port 3000 --open

# Generate OpenAPI 3.1 & AI Project Passport
endc doc src/main.end -o docs/

# Generate Dart/Flutter FFI bindings
endc flutter bindgen src/main.end -o flutter_lib/
```
