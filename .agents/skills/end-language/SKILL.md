---
name: end-language
description: Comprehensive expert guide for developing applications, web servers, 120 FPS native GUI applications, shared libraries (DLL), and high-performance algorithms in the End Programming Language.
---

# End Programming Language: Master Agent Reference & Architecture Guide

> **The End Programming Language (`end.exe` / `endc.exe`)** is an ultra-fast, zero-overhead, AI-first systems language that combines 3-Tier Hybrid Memory (Zero GC Region + Box + Rc/Arc), bare-metal execution beating Zig/C/Rust/Go, direct `@import_c` header bridging, lightweight fiber concurrency (`spawn` & Channels), integrated Package Manager (`endpm`), and deep semantic machine introspection (`inspect`, `explain`, `trace`, `effects`, `impact`, `fix`).

---

## 1. CLI Commands & Toolchain Ecosystem

The toolchain is globally installed in system PATH as `end.exe` and `endc.exe`:

```bash
# Package Manager & Scaffolding
end.exe new my_project                # Scaffold complete End project with end.toml
end.exe init                          # Initialize end.toml in current folder
end.exe add <package_name>            # Add dependency to end.toml

# Compilation & Execution
end.exe run main.end                  # Run immediately via VM Interpreter
end.exe build main.end -o app.exe     # Compile to ultra-optimized native binary (ReleaseFast)
end.exe build lib.end --dll -o lib.dll # Compile to Shared Library / DLL + auto-generated C Header (.h)

# Multi-Target Cross-Compilation
end.exe build app.end --target x86_64-linux -o app_linux
end.exe build app.end --target aarch64-macos -o app_macos
end.exe build app.end --target wasm32-wasi -o app.wasm

# AI Self-Healing & Semantic Introspection
end.exe fix app.end --apply           # AI Self-Healing: analyze bugs/typos and auto-patch source
end.exe inspect app.end --line 15     # JSON AST & lifetime telemetry
end.exe explain app.end --line 15     # Natural language explanation
end.exe trace app.end --symbol my_var # Symbol lifecycle timeline
end.exe impact app.end --symbol my_fn # Refactoring blast-radius analysis
end.exe check app.end                 # Fast semantic diagnostics
```

---

## 2. Direct C Header Ingestion (`@import_c`)

Zero glue code, zero wrapper overhead — directly include and call C libraries:

```end
// Directly bridge native C APIs (SQLite, OpenSSL, Math, Win32, POSIX)
@import_c("<math.h>")
@import_c("<stdio.h>")

fn main() void {
    val result = sqrt(64.0)
    println(result)
}
```

---

## 3. Hybrid 3-Tier Memory Model & Region Promotion

```
+-------------------------------------------------------------------------+
|                  HYBRID THREE-TIER MEMORY MODEL                         |
+-------------------------------------------------------------------------+
| Tier 1 (Default):   region name { ... }   -> 0-GC 64-byte Cache Arena   |
| Tier 2 (Heap Box):  val b: *Type = alloc  -> Owned heap lifecycle       |
| Tier 3 (Sharing):   end_rc_new / end_rc_clone / end_rc_drop             |
+-------------------------------------------------------------------------+
```

```end
region outer_scope {
    mut session: *UserSession = null

    region inner_temp {
        val temp: *UserSession = alloc UserSession
        temp.id = 1001
        // Promote lifetime from inner to outer region without copying bytes:
        session = promote(temp, outer_scope)
    }
}
```

---

## 4. Lightweight Concurrency & Lock-Free Channels

```end
fn worker(id: i32) void {
    println(id)
}

fn main() void {
    // 1. Lightweight asynchronous worker fiber
    spawn worker(42)

    // 2. High-performance MPSC Channel
    val ch: *EndChannel = channel_create(16)
    channel_send(ch, "Task Payload")
    val msg = channel_recv(ch)
}
```

---

## 5. 120 FPS Native Desktop Windowing & CustomPainter GUI

```end
st Color { r: i32, g: i32, b: i32, a: i32 }
st Canvas { width: i32, height: i32, pixels: *u32 }

fn main() void {
    val win = window_create("👑 End Native GUI", 800, 600)
    val raw_buffer: *u32 = alloc [480000]u32
    val canvas = Canvas { width: 800, height: 600, pixels: raw_buffer }

    while window_poll(win) {
        region frame_scope {
            // Render vector UI to canvas.pixels ...
            window_present(win, canvas.pixels)
            window_sleep(8) // ~120 FPS timing
        }
    }
}
```

---

## 6. Standard Library Modules (`std/`)

* `std/net/http.end`: High-throughput asynchronous HTTP server and client.
* `std/json/json.end`: SIMD-accelerated zero-copy JSON encoder and parser.
* `std/crypto/sha256.end`: Hardware-accelerated SHA-256 and cryptographic primitives.
* `std/db/sqlite.end`: Ultra-fast native database connection engine.
* `std/ui/`: Reactive CustomPainter and Desktop Windowing components.
