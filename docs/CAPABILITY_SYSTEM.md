# 🔒 End Language — Declarative Capability-Based Security Model

> **Compile-Time Fine-Grained Sandboxing for Safe AI-Generated Code and Zero-Trust Execution.**  
> *How End enforces hardware and OS boundaries at the compiler level without runtime overhead.*

---

## 1. The Threat Model of AI-Generated Code

When an AI coding agent writes or modifies systems code, there is a risk of unauthorized side effects:
- Accessing the disk or sensitive configuration files without permission.
- Spawning network sockets or making external outbound calls.
- Allocating unbounded heap memory that leaks system resources.

---

## 2. Declarative Capabilities (`@capability`)

In End, functions can declare explicit capability contracts:

```rust
@capability(net = true, disk = false, memory = "ArenaScoped")
pub fn handle_api_request(req: *HttpRequest) *HttpResponse {
    // ✔ Allowed: Network sockets, arena allocations
    // ❌ Forbidden at compile-time: Direct disk reads/writes
}
```

### Supported Capability Matrix:
- `net: bool` — Grants access to network socket creation and I/O.
- `disk: bool` — Grants access to filesystem read/write operations.
- `memory: "StackOnly" | "ArenaScoped" | "GlobalHeap"` — Restricts memory allocation scope.
- `ffi: bool` — Grants permission to invoke foreign C functions (`@import_c`).
- `process: bool` — Grants permission to spawn child processes.

---

## 3. Compile-Time Enforcement Protocol

When `endc` analyzes a function marked with `@capability`:
1. The compiler traverses the function's Call Graph and Abstract Syntax Tree.
2. If any callee or operation violates the declared capability (e.g. calling `fopen` when `disk = false`), compilation immediately halts with diagnostic error:
   ```text
   ❌ Error E0403: Capability Violation in function `handle_api_request`
      --> src/server.end:42:5
      |
   42 |     val file = fopen("secret.key", "r");
      |                ^^^^^^^^^^^^^^^^^^^^^^^^ Direct disk I/O attempted
      |
      = note: Function declares capability `@capability(disk = false)`. Disk operations are strictly forbidden.
   ```
