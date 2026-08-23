# 📖 End Programming Language — Formal Language Specification

> **Official Systems Architecture & Language Grammar Reference**  
> *Compiler Target: `endc` v2.0.0 (LLVM 18 / Cranelift / Native C11 / WebAssembly / VM)*  
> *Platform: Windows, Linux, macOS, iOS, Android, WebAssembly, Bare-Metal Embedded*

---

## 1. Introduction & Language Overview

The **End Programming Language** is a modern, statically typed, compiled, general-purpose systems programming language designed from the ground up for high machine throughput, zero-overhead memory safety, and native AI-agent collaboration.

### Core Paradigms
- **Zero-Overhead Memory Safety:** 4-tier hybrid memory management (Tier 0: Ephemeral Leases, Tier 1: Bump-Allocated Regions, Tier 2: Static Borrow Checker, Tier 3: Hardware Raw Pointers).
- **First-Class Operation Algebra:** Operations (`operation` / `op`) are first-class values that support algebraic composition (`>>`, `&`, `|`, `*`, `retry`, `repeat`, `memoize`).
- **Reactive Event Topologies:** Native event streams, topologies (`topology`), and execution graphs with formal invariant verification.
- **Composition over Inheritance:** High-order modular architecture utilizing `capability`, `surface`, `shape`, `mixin`, and contractual `feature` declarations.
- **Autonomous Agent Contracts:** First-class machine-readable contracts (`intent`, `prove`, `assume`, `guarantee`, `invariant`, `task`, `skill`, `evidence`, `evolve`).
- **Multi-Target Native Compilation:** Direct lowering to LLVM 18 IR, Cranelift JIT, Optimized C11 with LTO, and WASM/WASI.

---

## 2. Lexical Structure & Grammar

### 2.1 Character Set & Comments
- **Encoding:** UTF-8 source files (`.end`).
- **Line Comments:** `// comment text`
- **Block Comments:** `/* multi-line comment */` (supports nesting)
- **Whitespace:** Spaces, tabs, and newlines are delimiters; whitespace is not significant for indentation blocks (braces `{}` are used).
- **Semicolons:** Semicolons `;` are statement terminators / separators. Optional after closing braces `}` of block declarations.

### 2.2 Identifiers & Morphic Identifiers
- **Standard Identifiers:** Starts with an ASCII letter or underscore `[a-zA-Z_]`, followed by alphanumeric characters or underscores `[a-zA-Z0-9_]*`.
- **Morphic Identifiers:** Segmental meta-identifiers in the format `{segment}_suffix` (e.g. `{platform}_send`, `{target}_Client`). Morphic identifiers allow writing single polymorphic functions and structs that specialize into platform-specific implementations.

### 2.3 Keywords Matrix
| Category | Keywords |
| :--- | :--- |
| **Variables & Storage** | `val`, `mut`, `var`, `val!`, `lease`, `borrow`, `during`, `owned`, `frozen`, `protect`, `mutable_by`, `handoff`, `return_to` |
| **Declarations** | `fn`, `st`, `struct`, `enum`, `class`, `trait`, `impl`, `mod`, `module`, `feature`, `operation`, `op`, `event`, `hub`, `contract`, `shape`, `surface`, `capability`, `mixin`, `syntax` |
| **Control Flow** | `if`, `else`, `while`, `for`, `in`, `parallel`, `choose`, `race`, `hedge`, `match`, `return`, `ret`, `defer`, `spawn`, `skip`, `guard`, `catch` |
| **Pipelining & Logic** | `when`, `then`, `where`, `is`, `as`, `with`, `by`, `through`, `into`, `from`, `toward`, `only`, `except`, `over`, `without` |
| **Capabilities & Surfaces** | `access`, `grant`, `deny`, `adopt`, `implement`, `attach`, `detach`, `provide`, `require`, `resolve`, `select`, `project`, `delegate`, `proxy`, `intercept`, `hook`, `enable`, `disable`, `scope`, `traitify`, `equip`, `fuse` |
| **Reactive Events** | `emit`, `on`, `once`, `every`, `after`, `before`, `state`, `derive`, `topology`, `debounce`, `throttle`, `sample`, `coalesce`, `window`, `ack`, `require_ack`, `replayable`, `durable`, `event_sourced`, `event_transaction`, `quarantine`, `circuit_breaker`, `retry_policy`, `dead_letter_queue` |
| **Inheritance & Polymorphism**| `inherits`, `inherit`, `abstract`, `super`, `superchain`, `conflict`, `virtual`, `share`, `delegates`, `delegation`, `contractually`, `inspect` |
| **Agent Contracts & Governance** | `skill`, `skills`, `satisfies`, `rules`, `constraints`, `requirement`, `requirements`, `implements`, `verifies`, `claim`, `complete`, `evidence`, `todo`, `knowledge`, `decision`, `approval`, `review`, `confidence`, `agent_boundary`, `agent_context`, `context_firewall`, `agent_api`, `agentability`, `regression_guard`, `tasks`, `profile`, `hard`, `soft`, `structural`, `semantic`, `behavioral`, `performance`, `security`, `testing`, `summary`, `risks`, `recommendation`, `notes` |
| **Extensibility DNA** | `partial`, `augment`, `extension_only`, `extension_point`, `replace`, `migration`, `overlay`, `open`, `closed`, `compiler_plugin`, `lint`, `analyzer`, `type_rule`, `optimizer`, `build_plugin`, `generator`, `reflect`, `lock`, `agent_extension`, `proposal`, `evolvable`, `owned_by`, `architecture_test`, `provides`, `guarantees`, `rename`, `use`, `snapshot`, `replace_with`, `api`, `needs`, `expose`, `replaceable`, `lifecycle`, `decorate`, `impact`, `must`, `reason`, `internal`, `private`, `public`, `pub`, `extends`, `extension`, `implementation`, `test`, `begin`, `commit`, `not` |
| **Autonomous Refactoring** | `refactor`, `decompose`, `conservation`, `inventory`, `audit_log`, `traceable`, `coupling`, `solid`, `unaccounted`, `repair`, `gravity`, `split`, `partition`, `extract`, `cluster`, `separate`, `merge`, `inline` |
| **Formal Proofs & Invariants**| `intent`, `prove`, `assume`, `guarantee`, `invariant`, `verify`, `because`, `why`, `explain`, `context`, `slice`, `patch`, `evolve`, `goal`, `preserve`, `allow`, `accept`, `reject`, `baseline`, `regression`, `order`, `deterministic`, `replay`, `checkpoint`, `rollback`, `transaction`, `speculative`, `fallback`, `budget`, `deadline`, `priority`, `quality`, `tradeoff`, `adapt`, `observe`, `watch`, `react`, `stream`, `flow`, `cancel_safe` |
| **Low-Level & Interop** | `inline_c`, `asm`, `target`, `import`, `alloc`, `null`, `true`, `false`, `extern`, `bridge`, `override` |

### 2.4 Operators & Delimiters
- **Arithmetic:** `+`, `-`, `*`, `/`, `%`, `**` (power)
- **Comparison:** `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical:** `&&`, `||`, `!`
- **Bitwise:** `&`, `|`, `^`, `~`, `<<`, `>>`
- **Assignment:** `=`, `:=` (walrus / destructure), `??=` (null-coalesce assign), `<+=` (atomic add-assign)
- **Pipelining & Nullability:**
  - `|>` (pipe operator: `x |> f` $\rightarrow$ `f(x)`)
  - `??` and `~>` (null-coalescing / null-collapse)
  - `?.` (null-aware member access)
  - `..` (cascade operator: `obj..set_x(10)..set_y(20)`)
  - `?..` (null-aware cascade operator)
  - `...` (spread operator: `[...list1, ...list2]`)
  - `...?` (null-aware spread operator)
  - `..` and `..<` (inclusive range `0..10`, exclusive range `0..<10`)
  - `<->` and `<~>` (bidirectional / reactive channel connectors)
  - `!->` (breaking transition / mutation arrow)
  - `->` (return type / single direction flow)
  - `=>` (fat arrow for lambdas / match arms / projections)

### 2.5 Literals
- **Integers:** Decimal `42`, Hexadecimal `0x2A`, Binary `0b101010` (type `i64` by default, coerces to target integer type).
- **Floats:** `3.14159`, `.5`, `100.0` (type `f64` by default, coerces to `f32`).
- **Unit Literals:** Numeric literal immediately followed by bracketed unit: `120[km/h]`, `500[ms]`, `1024[MB]`, `45[deg]`.
- **Strings & Interpolation:**
  - Standard String: `"Hello, World!\n"`
  - Interpolated String: `"User ID: {user.id}, Name: {user.name.to_uppercase()}"`
  - Raw Escape Sequences: `\n`, `\r`, `\t`, `\\`, `\"`
- **Booleans:** `true`, `false`
- **Null:** `null`

---

## 3. Type System

End is statically and strongly typed. Every expression has a deterministic type at compile time.

```
                                    Type
                                      │
     ┌───────────────┬────────────────┼────────────────┬───────────────┐
     ▼               ▼                ▼                ▼               ▼
 Primitive        Compound         Generic          Algebraic       System & Resource
 • void          • Struct (st)    • List<T>        • Result (!T)   • region<R>
 • bool          • Enum           • Map<K,V>       • Option (?T)   • Box<T>
 • i8..i64       • Tuple (A, B)   • Set<T>         • Operation     • Rc<T> / Arc<T>
 • u8..u64       • Slice ([]T)    • Channel<T>     • Event<E>      • Channel<T>
 • f32, f64      • Array ([N]T)                                    • Allocator
 • str           • SIMD (TxN)                                      • Raw Pointer (*T)
```

### 3.1 Primitive Types
| Type | Bit Width | Description |
| :--- | :--- | :--- |
| `void` | 0 | Unit type, represents no return value |
| `bool` | 8 | Boolean logical value (`true` or `false`) |
| `i8`, `i16`, `i32`, `i64` | 8, 16, 32, 64 | Two's-complement signed integers |
| `u8`, `u16`, `u32`, `u64` | 8, 16, 32, 64 | Unsigned integers |
| `f32` | 32 | IEEE 754 single-precision floating point |
| `f64` | 64 | IEEE 754 double-precision floating point |
| `str` | 128 | UTF-8 string slice (pointer + length) |

### 3.2 Compound & Collection Types
- **Pointers:** `*T` (raw bare-metal pointer), `*mut T` (mutable pointer).
- **Slices:** `[]T` (dynamically sized view into contiguous buffer).
- **Fixed-Size Arrays:** `[N]T` (e.g. `[16]u8`, `[3]f32`).
- **SIMD Vector Types:** `TxN` where `T` is a primitive and `N` is lane count: `f32x4`, `f32x8`, `i32x4`, `i32x8`, `u8x16`.
- **Tuples:** `(T1, T2, ...)` e.g. `(str, i64, bool)`.
- **Generics:** `Name<T1, T2, ...>` e.g. `List<User>`, `Map<str, i64>`.

### 3.3 Error & Algebraic Types
- **Fallible Result:** `!T` or `Result<T, E>` — represents either success value `T` or error `E`.
- **Nullable / Option:** Optional representation using null-coalescing and quantum unwrapping.
- **First-Class Operations:** `Operation<TIn, TOut>` — pure operation values that can be composed and transformed.
- **Events:** `Event<T>` — typed event payload channel.

### 3.4 Resource & Memory Allocation Types
- **Region Reference:** `region<Name>` — tracks lifetime bounded to an allocation arena.
- **Unique Heap Box:** `Box<T>` — Tier 2 heap allocation with single-owner move semantics.
- **Reference Counted:** `Rc<T>` (single-threaded) and `Arc<T>` (atomic thread-safe).
- **Message Channel:** `Channel<T>` — lock-free SPSC / MPSC ring-buffer channel.
- **Allocator:** `Allocator` — custom memory allocator trait interface.

---

## 4. Declarations & Program Structure

An End program consists of module files (`.end`). Modules organize structs, enums, functions, traits, operations, events, features, and agent contracts.

### 4.1 Variables & Storage
```end
// Immutable binding (Default)
val pi: f64 = 3.141592653589793;
val name = "Alice"; // Type inferred as str

// Mutable variable
mut counter: i64 = 0;
counter = counter + 1;

// Ephemeral Lease (automatically recycled at scope exit)
lease val buffer = alloc(1024) {
    // buffer is valid strictly within this scope
};

// Quantum Unwrap
val! result = fallible_op() ?? 0;
```

### 4.2 Functions
```end
// Standard Function
pub fn add(a: i64, b: i64) -> i64 {
    ret a + b;
}

// Function with Generic Parameters & Default / Named Arguments
pub fn find_or_default<T>(items: []T, predicate: fn(T) -> bool, default_val: T = null) -> T {
    for item in items {
        if predicate(item) {
            ret item;
        }
    }
    ret default_val;
}

// Morphic Function (Specializes for each platform)
pub fn {platform}_send(recipient: str, payload: str) -> str {
    ret "{platform}:" + recipient + ":" + payload;
}
```

### 4.3 Structs & Enums
```end
// Struct with field alignments and directives
@export(C, Flutter, WASM)
pub struct User {
    pub id: i64,
    pub name: str,
    pub email: str,
    pub is_active: bool,
}

// Algebraic Data Type (Enum with variant payloads)
pub enum UserStatus {
    Unverified,
    Active(i64), // Contains last login timestamp
    Suspended(str), // Contains suspension reason
}
```

### 4.4 Classes & Inheritance (Contractual Subtyping)
```end
pub class BaseController {
    pub base_url: str,
    
    pub fn init(base_url: str) -> Self {
        ret BaseController { base_url: base_url };
    }
    
    pub fn log_request(path: str) {
        println("Incoming request: " + path);
    }
}

pub class AuthController inherits BaseController {
    pub jwt_secret: str,
    
    pub fn login(user: str) -> bool {
        super.log_request("/auth/login");
        ret true;
    }
}
```

### 4.5 Traits, Mixins & Surfaces
```end
// Trait Definition
pub trait Serializable {
    fn serialize(&self) -> str;
}

// Struct Impl
impl Serializable for User {
    fn serialize(&self) -> str {
        ret "{\"id\":" + self.id + ",\"name\":\"" + self.name + "\"}";
    }
}

// Capability Surface Definition
surface User.PublicProfile {
    id,
    name,
    is_active
}

// Equip entity with capabilities
equip User with [Serializable, PublicProfile];
```

---

## 5. First-Class Operation Algebra

In End, operations are pure computational units that can be combined, repeated, retried, parallelized, and guarded using algebraic operators:

```end
// 1. Operation Definition
operation FetchUser(id: i64) -> User {
    requires: id > 0;
    guarantees: result.id == id;
    effects: [NetworkRead];
    emits: [UserFetched];
    
    val user = http_get("/users/" + id);
    ret user;
}

operation ValidateUser(user: User) -> bool {
    ret user.is_active;
}

// 2. Algebraic Composition Pipeline
// Sequential composition: FetchUser >> ValidateUser
val UserPipeline = compose {
    FetchUser >> ValidateUser;
};

// Retry combinator (retry up to 3 times on transient failure)
val ResilientFetch = retry FetchUser up to 3;

// Parallel branch combination
val ParallelTasks = FetchUser || FetchSettings;

// Alternative fallback selection
val FallbackFetch = PrimaryAPI / BackupAPI / CachedData;

// Memoization combinator
val CachedFetch = memoize FetchUser;
```

---

## 6. Reactive Events & Execution Topologies

End includes a native reactive graph engine with zero external dependencies:

```end
// Event Definition
pub event OrderCreated {
    order_id: i64,
    amount: f64,
    customer_id: str,
}

// Event Hub
pub hub OrderHub {
    owns: [OrderCreated, OrderPaid];
}

// Reactive Subscription
on OrderCreated when amount > 1000.0 {
    println("High value order created: " + order_id);
    emit OrderFlaggedForReview(order_id);
}

// Scheduled Reactive Timers
every 5s {
    heartbeat_ping();
}

// Stream Windowing & Backpressure Topologies
debounce 250ms on SearchInput {
    query_suggestions(input_text);
}

topology OrderPipeline {
    OrderCreated -> ValidateInventory -> ProcessPayment -> EmitReceipt;
}
```

---

## 7. Feature-Oriented Paradigm & Extensibility DNA

Features in End encapsulate entire architectural slices (API, Domain, Storage, Tests, Contracts):

```end
pub feature BillingSystem {
    version: "2.1.0",
    owner: "PaymentTeam",
    
    needs: [
        Authentication.api,
        DatabaseConnection
    ],
    
    exposes: [
        process_charge,
        issue_refund
    ],
    
    contracts: [
        "All transactions must be idempotent",
        "PCI-DSS compliance guaranteed"
    ],
    
    api {
        fn process_charge(amount: i64, token: str) -> !str;
    }
    
    implementation {
        fn process_charge(amount: i64, token: str) -> !str {
            prove amount > 0;
            ret "TX_SUCCESS";
        }
    }
    
    tests {
        fn test_charge_positive() {
            val res = process_charge(100, "tok_123");
            assert(res == "TX_SUCCESS");
        }
    }
}
```

---

## 8. Agent Contract System & AI Protocol

End is built from the ground up to be inspected, reasoned over, and safely mutated by autonomous AI pair programming agents:

```end
// Agent Charter Contract
agent BackendArchitect {
    scope: "services/billing",
    goal: "Maintain P99 latency < 10ms with zero memory leaks",
    constraints: [
        "Do not modify public API signatures without migration path",
        "Preserve double-entry balance invariants"
    ]
}

// Agent Task Lifecycle
task OptimizePaymentLoop {
    owner: "Agent-7",
    status: "In Progress",
    requirement: "REQ-409",
    skills: ["ZeroAllocPipes", "SIMDVectorization"],
    change_budget: ["max_files: 2", "max_loc: 150"],
    
    evidence {
        benchmark: "P99 reduced from 14.2ms to 6.1ms",
        safety: "100% tests passing in test_suite_250_syntaxes.end"
    }
}

// Claim & Complete Verification Gates
claim task OptimizePaymentLoop;

complete task OptimizePaymentLoop {
    result: "success",
    confidence: 0.99,
    summary: "Vectorized tax calculation using f32x8 SIMD",
    evidence: ["tests/perf_audit.log", "bin/bench_results.json"]
}
```

---

## 9. Autonomous Lossless Refactoring Engine

End compiler enforces the **Law of Source Conservation** to guarantee that automated refactoring never silently deletes, leaks, or corrupts code:

```end
refactor session ModuleDecomposition {
    agent: "CodeRefactorAgent",
    target: "monolith_server.end",
    scope: ["auth", "billing", "analytics"],
    forbid: ["modifying public api"],
    goals: ["reduce file size to < 500 lines per submodule"]
}

conservation audit {
    original_source: "monolith_server.end",
    original_loc: 2400,
    original_symbols: ["User", "AuthService", "PaymentEngine"],
    new_loc: 2390,
    accounted_symbols: ["User", "AuthService", "PaymentEngine"],
    unaccounted_count: 0,
    allow_semantic_deletion: false
}
```

---

## 10. Memory Model & Resource Management

### 10.1 Deterministic 4-Tier Memory Hierarchy
1. **Tier 0: Ephemeral Leases (`lease` / `borrow`)**  
   Zero nanosecond allocation and automatic deallocation at scope termination. Prevents memory leaks by construction.
2. **Tier 1: Bump-Allocated Regions (`region Name { ... }`)**  
   Ultra-fast $O(1)$ pointer bumping with bulk arena reset upon region scope exit.
3. **Tier 2: Static Borrow Checker (`&T`, `&mut T`, `Box<T>`)**  
   Compile-time single-mutable borrow exclusivity rule. Move semantics for unique heap boxes.
4. **Tier 3: Bare-Metal Pointers (`*T`, `*mut T`)**  
   Direct unmanaged hardware memory access for OS kernels, GPU buffers, and C FFI.

---

## 11. Foreign Function Interface (FFI) & Multi-Target Bindings

### 11.1 Native C Header Interoperability
```end
@import_c("<stdio.h>")
@import_c("<math.h>")

pub fn main() -> i32 {
    printf("Native C interop from End!\n");
    ret 0;
}
```

### 11.2 Multi-Ecosystem Bindgen (`endc bindgen`)
Using `@export(C, Flutter, WASM)`, the End compiler generates bindings for:
- **Flutter / Dart:** `dart:ffi` typed structs and method bindings.
- **Python:** Zero-copy C-extension and `ctypes` wrapper.
- **TypeScript / Node.js / Bun:** Native N-API and `bun:ffi` bindings.
- **C# / Unity / .NET:** Native P/Invoke DLL bindings.
- **WebAssembly:** `.wasm` binary with JavaScript WASI glue code.

---

## 12. Standard Library & Frameworks Overview

- **`std/core`**: `Option`, `Result`, primitive operations, memory allocations.
- **`std/net`**: HTTP/1.1, HTTP/2, WebSockets, RPC, socket abstractions (`EndNexus`).
- **`std/ui`**: 120 FPS declarative UI components, canvas painting, widget hierarchy (`EndUI`).
- **`std/db`**: Native PostgreSQL, SQLite, Redis client interfaces, type-safe SQL builder (`EndForge`).
- **`std/ai`**: Local tensor engine, GGUF model loader, intent parser, tokenizer.
- **`std/crypto`**: AES-256-GCM, SHA-256/512, Ed25519, JWT, TLS/ACME certificate automation (`EndCrypto`).
- **`std/collections`**: Lock-free SPSC queues, HashMaps, B-Trees, RingBuffers, Vector lists.
- **`std/async`**: Lightweight fibers, coroutines, channels, schedulers (`EndHyper`).

---

## 13. Compiler CLI & Tooling Ecosystem

```bash
# Compile and run
endc run src/main.end

# Compile native executable with -O3 optimizations
endc build src/main.end -o bin/app.exe

# Compile dynamic library (DLL / .so / .dylib)
endc build src/main.end -o bin/libcore.dll --dll

# Launch 120 FPS Declarative UI DevServer + AI Canvas Overlay
endc ui src/main.end --dev --serve --port 3000 --open

# Generate OpenAPI 3.1 & AI Project Passport
endc doc src/main.end -o docs/

# Generate Dart/Flutter FFI bindings
endc flutter bindgen src/main.end -o flutter_lib/

# AI Agent inspection tools
endc agent list
endc graph src/main.end --json
endc slice src/main.end --tokens 500 --json
endc patch src/main.end --patch-file patch.json
endc verify src/main.end --json
endc security src/main.end --level strict --json
```
