# 📖 End Programming Language — Formal Language Specification

> **The 5 Pillars: Type System, Memory Model, Compiler Architecture, Concurrency, and Foreign Function Interface (FFI).**  
> *Version 1.0.0 — Apex Systems & Formal Specification Edition.*

---

## 1. Pillar A: Type System & Formal Semantics

End features a statically checked, strongly typed, and Hindley-Milner influenced type system designed for high machine throughput and cognitive clarity.

### 1.1 Primitive Types
- **Signed Integers:** `i8`, `i16`, `i32`, `i64`, `i128`
- **Unsigned Integers:** `u8`, `u16`, `u32`, `u64`, `u128`
- **Floating-Point:** `f32` (IEEE 754 single precision), `f64` (double precision)
- **Boolean & Text:** `bool` (`true`/`false`), `str` (UTF-8 string slice)
- **Unit / None:** `void`

### 1.2 User-Defined Structs (`st`)
Structs have explicit field alignments and C-compatible memory layouts:
```rust
st Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

st HttpResponse {
    status_code: i32,
    body: str,
    headers_count: i32,
}
```

### 1.3 Immutability by Default (`val` vs `mut`)
Variables are strictly immutable by default (`val`), requiring explicit `mut` for mutable storage:
```rust
val pi: f64 = 3.141592653589793; // Immutable
mut counter: i64 = 0;             // Mutable
counter = counter + 1;
```

---

## 2. Pillar B: Memory Model & Safety Architecture

End uses a **Deterministic 3-Tier Memory Model**:

### 2.1 Region-Based Bump Arenas (`region name { ... }`)
- Dynamic allocations inside a region are handled by ultra-fast pointer bumping ($O(1)$).
- When a region scope closes, all internal memory is instantly reset with zero scanning overhead.
- Escape analysis prevents inner-region references from escaping to outer lifetimes.

### 2.2 Compile-Time Static Borrow Checker
- Linear ownership transfer (Move semantics) for heap and region buffers.
- Single mutable reference (`&mut T`) exclusivity rule: at any time, a value can have either multiple read-only borrows (`&T`) or exactly one mutable borrow (`&mut T`), preventing data races at compile time.

### 2.3 Bare-Metal Raw Pointers (`*T`)
- Direct hardware and OS pointer access for kernel drivers and C FFI (`*u8`, `*TreeNode`).

---

## 3. Pillar C: Compiler Architecture & Codegen Pipeline

The End compiler (`endc`) is constructed as a multi-stage, zero-copy compilation pipeline:

```
[Source .end] 
      │
      ▼
1. Zero-Allocation Lexer (endc/src/lexer.rs)
      │
      ▼
2. Recursive Descent Parser & Concrete AST (endc/src/parser.rs)
      │
      ▼
3. Semantic Analyzer, Borrowck & Effects (endc/src/semantic/)
      │
      ▼
4. High-Level Intermediate Representation (HIR) (endc/src/ir/hir.rs)
      │
      ▼
5. Mid-Level Control Flow Graph (MIR) (endc/src/ir/mir.rs)
      │
      ▼
6. Multi-Target Codegen Engine:
   ├── High-Performance C11 Backend with LTO & Fast-Math (endc/src/codegen/c_backend.rs)
   ├── Direct LLVM 18 IR Emitter (endc/src/codegen/llvm_backend.rs)
   ├── Sub-Millisecond JIT & Interpreter (endc/src/codegen/interpreter.rs)
   └── Cranelift JIT Backend (endc/src/codegen/cranelift_backend.rs)
      │
      ▼
[Ultra-Optimized Stripped Native Binary / DLL / AAR / XCFramework]
```

---

## 4. Pillar D: High-Performance Concurrency

### 4.1 Lock-Free Ring Buffers (SPSC Queues)
End includes hardware-optimized Single-Producer Single-Consumer (SPSC) lock-free ring buffers with cache-line padding (64 bytes) to eliminate false sharing.

### 4.2 Hardware Atomics & Memory Barriers
Direct access to CPU atomic primitives (`atomic_load`, `atomic_store`, `atomic_fetch_add`, `compare_and_swap`) with Sequential Consistency, Acquire-Release, and Relaxed memory orderings.

### 4.3 Lightweight Fiber Coroutines & Event Loops
Non-blocking I/O event loops for ultra-fast networking services (e.g. `EndNexus` HTTP server handling 100,000+ requests/sec).

---

## 5. Pillar E: Seamless Foreign Function Interface (FFI) & Multi-Target Bindings

### 5.1 Direct C ABI Header Imports (`@import_c`)
End can call any C library, OS API, or GPU driver directly without boilerplate wrappers:
```rust
@import_c("<stdio.h>")
@import_c("<math.h>")
@import_c("<windows.h>")

pub fn main() i32 {
    printf("Hello from End with native C ABI!\n")
    ret 0
}
```

### 5.2 Automatic Multi-Language FFI Bindings (`end bindgen`)
The compiler generates idiomatic, zero-copy FFI bindings for:
- **Python:** `ctypes` & C-extension modules.
- **TypeScript / Node.js:** Native N-API and `bun:ffi` bindings.
- **Dart / Flutter:** `dart:ffi` bindings for mobile and desktop apps.
- **C# / Unity:** `P/Invoke` native shared libraries (.dll/.dylib/.so).
