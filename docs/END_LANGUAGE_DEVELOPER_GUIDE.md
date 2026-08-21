# 👑 The End Programming Language: Master Developer & Agent Guide

> **Hybrid 3-Tier Memory • Bare-Metal Speed • Zero GC • Direct @import_c • Fiber Concurrency • AI Self-Healing**

---

## 1. Quick Start & Ecosystem Commands

The compiler is available everywhere in Windows PATH (`end.exe` & `endc.exe`):

```powershell
# Project Management
end.exe new my_service                # Scaffold complete project with end.toml
end.exe init                          # Initialize end.toml in current folder
end.exe add <package_name>            # Add dependency to manifest

# Execution & Native Compilation
end.exe run main.end                  # Run immediately via VM
end.exe build main.end -o app.exe     # Compile to native binary (.exe)
end.exe build mathlib.end --dll -o mathlib.dll  # Compile to DLL + C Header (.h)

# AI Self-Healing & Semantic Introspection
end.exe fix app.end --apply           # AI Self-Healing compiler auto-patching
end.exe inspect app.end --line 10     # Deep semantic introspection
end.exe explain app.end --line 10     # Natural language explanation
end.exe trace app.end --symbol x      # Symbol lifecycle data-flow
end.exe impact app.end --symbol fn    # Blast-radius analysis
```

---

## 2. Master Feature Capabilities

| Feature | Syntax / Command | Description |
| :--- | :--- | :--- |
| **Direct C Ingestion** | `@import_c("<sqlite3.h>")` | Zero-glue direct C/C++ library bridging |
| **3-Tier Memory Model** | `region`, `alloc`, `promote`, `RC` | Scoped arena + heap box + ref-counted sharing |
| **Fiber Concurrency** | `spawn worker()`, `channel_*` | Virtual green threads with lock-free MPSC channels |
| **Desktop GUI Engine** | `window_create`, `Canvas` | 120 FPS native V-Sync vector CustomPainter |
| **Standard Library** | `std/net`, `std/json`, `std/crypto`, `std/db` | Production-ready network, JSON, SHA256 & DB modules |
| **Package Manager** | `end.toml`, `end new`, `end add` | Modern project configuration and dependency management |
| **AI Self-Healing** | `end fix --apply` | Automated compiler semantic bug fixing and patching |

---

## 3. Working Example Directory
* `examples/test_import_c.end`: Direct `@import_c("<math.h>")` demonstration.
* `examples/test_memory_3tier.end`: 3-Tier memory model & region promotion demonstration.
* `examples/test_concurrency_fibers.end`: Concurrency fibers and lock-free channels.
* `examples/pure_end_window.end`: 100% Pure End 120 FPS native desktop HUD application.
* `examples/mathlib.end`: DLL / Shared Library export with C Header generation.
