---
name: end-language
description: Comprehensive expert guide for developing applications, web servers, 120 FPS native GUI applications, shared libraries (DLL), and high-performance algorithms in the End Programming Language.
---

# End Programming Language: Developer Reference & Agent Handbook

> **The End Programming Language (`end.exe` / `endc.exe`)** is an ultra-fast, zero-overhead, AI-first systems language that combines Region memory scoping (Zero GC), bare-metal speed beating Zig/C/Rust/Go, and deep semantic machine introspection (`inspect`, `explain`, `trace`, `effects`, `impact`).

---

## 1. CLI Commands & Global Execution

The compiler is globally accessible on system PATH as `end.exe` or `endc.exe`:

```bash
# Run source file directly (Interpreter VM)
end.exe run app.end

# Compile to ultra-optimized native binary (ReleaseFast)
end.exe build app.end -o app.exe

# Compile to Shared Library / DLL with automatic C Header (.h) generation
end.exe build mathlib.end --dll -o mathlib.dll

# Cross-compile for any target (Linux, macOS Apple Silicon, WASM)
end.exe build app.end --target x86_64-linux -o app_linux
end.exe build app.end --target aarch64-macos -o app_macos
end.exe build app.end --target wasm32-wasi -o app.wasm

# Agent Semantic Introspection & Tooling
end.exe inspect app.end --line 15
end.exe explain app.end --line 15
end.exe trace app.end --symbol my_var
end.exe impact app.end --symbol compute_total
end.exe check app.end
```

---

## 2. Core Language Syntax & Grammar

### Variables & Types
```end
val constant_val: i32 = 100
mut mutable_counter: i64 = 0
val is_active: bool = true
val pi_float: f32 = 3.14159
val name_str: str = "Cyberpunk HUD"
```

### Structs & Tagged Enums (Pattern Matching)
```end
st User {
    id: u64,
    score: i32,
    active: bool,
}

enum Status {
    Idle,
    Running(i32),
    Error,
}

fn handle_status(s: Status) i32 {
    ret match s {
        .Idle => 0,
        .Running(progress) => progress,
        .Error => -1,
        _ => 0,
    }
}
```

### Zero-GC Region Scoping & Continuous Memory
```end
fn process_workload() void {
    // Arena allocated in hardware cache, automatically wiped at scope exit with 0 GC overhead
    region frame_scope {
        val buffer: *u32 = alloc [256000]u32
        for i in 256000 {
            buffer[i] = (255 << 24) | (0 << 16) | (245 << 8) | 255
        }
    }
}
```

---

## 3. Building 120 FPS Native Desktop GUI & CustomPainter Apps

To build high-performance GUI applications in pure End:

```end
// 100% Pure End Native Desktop Application
st Color { r: i32, g: i32, b: i32, a: i32 }
fn color_rgba(r: i32, g: i32, b: i32, a: i32) Color { ret Color { r: r, g: g, b: b, a: a } }

st Canvas { width: i32, height: i32, pixels: *u32 }

fn canvas_clear(c: Canvas, col: Color) void {
    val pv: u32 = (col.a << 24) | (col.r << 16) | (col.g << 8) | col.b
    val total = c.width * c.height
    for i in total {
        c.pixels[i] = pv
    }
}

fn main() void {
    val win = window_create("👑 My End GUI App", 800, 600)
    val raw_buffer: *u32 = alloc [480000]u32
    val canvas = Canvas { width: 800, height: 600, pixels: raw_buffer }
    mut frame: i32 = 0

    while window_poll(win) {
        region frame_region {
            canvas_clear(canvas, color_rgba(15, 18, 25, 255))
            // Render custom vector UI...
            window_present(win, canvas.pixels)
            window_sleep(8)
            frame = frame + 1
        }
    }
}
```

---

## 4. Exporting Shared Libraries (`.dll` / `.so`) for Other Languages

To create a library callable by Python (`ctypes`), Dart/Flutter (`dart:ffi`), C#, C++, Rust:

```end
// mathlib.end
@export
fn end_add(a: i64, b: i64) i64 {
    ret a + b
}

@export
fn end_process_batch(count: i32) i64 {
    mut total: i64 = 0
    region batch_scope {
        for i in count {
            total = total + (i * 31)
        }
    }
    ret total
}
```

Compile with:
```bash
end.exe build mathlib.end --dll -o mathlib.dll
# Generates mathlib.dll and C Header mathlib.h!
```
