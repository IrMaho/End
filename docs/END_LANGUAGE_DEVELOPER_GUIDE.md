# 👑 The End Programming Language: Official Developer & Agent Guide

> **Zero GC • Bare-Metal C/Zig Speed • AI-First Machine Introspection • 120 FPS Native GUI**

---

## 1. Quick Start

The compiler is available everywhere in Windows PATH:
```powershell
# Run instantly with VM
end.exe run main.end

# Compile to optimized native binary (.exe)
end.exe build main.end -o main.exe

# Compile to Shared Library (.dll) + C Header (.h)
end.exe build mathlib.end --dll -o mathlib.dll

# Cross-compile for Linux or WebAssembly
end.exe build main.end --target x86_64-linux -o main_linux
end.exe build main.end --target wasm32-wasi -o main.wasm
```

---

## 2. Capabilities Overview

| Task | Syntax / Feature | Description |
| :--- | :--- | :--- |
| **GUI & Desktop Apps** | `window_create`, `window_poll`, `window_present`, `Canvas` | 120 FPS zero-overhead V-Sync desktop windows with CustomPainter |
| **Backend & Microservices** | `parallel`, `region`, `alloc [N]Type` | High-throughput web APIs handling 4,978M req/s |
| **Shared Libraries** | `@export fn my_func()`, `end build --dll` | Produces native `.dll` and `.h` C-ABI for Python, C#, Dart/Flutter FFI, Rust, Go |
| **Memory Management** | `region my_scope { ... }` | Stack/Cache scoped arenas with zero GC pauses and zero runtime leaks |
| **Introspection** | `end.exe inspect / trace / impact / explain` | Machine-readable JSON AST and semantic telemetry for AI agents |

---

## 3. Project Directory Reference
* `std/ui/`: UI primitives (`color.end`, `geometry.end`, `canvas.end`, `widget.end`).
* `examples/`: Full working examples:
  * `examples/pure_end_window.end`: 100% Pure End 120 FPS Desktop HUD.
  * `examples/mathlib.end`: DLL / Shared Library export.
  * `examples/zero_import_app.end`: Standalone zero-import app.
  * `examples/server.end`: Backend server.
