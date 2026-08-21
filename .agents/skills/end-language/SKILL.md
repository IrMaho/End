---
name: end-language
description: Comprehensive expert guide for developing applications, web servers, 120 FPS native GUI applications, shared libraries (DLL), and high-performance algorithms in the End Programming Language.
---

# End Programming Language: Master Agent Reference & Architecture Guide

> **The End Programming Language (`end.exe` / `endc.exe`)** is an ultra-fast, zero-overhead, AI-first systems language that combines 3-Tier Hybrid Memory (Zero GC Region + Box + Rc/Arc), bare-metal execution beating Zig/C/Rust/Go, direct `@import_c` header bridging, lightweight fiber concurrency (`spawn` & Channels), integrated Package Manager (`endpkg`), 5 official first-party enterprise frameworks (`EndHyper`, `EndForge`, `EndCrypto`, `EndStream`, `EndKV`), and deep semantic machine introspection (`inspect`, `explain`, `trace`, `effects`, `impact`, `fix`).

---

## 1. CLI Commands & Toolchain Ecosystem

The toolchain is globally installed in system PATH as `end.exe` and `endc.exe`:

```bash
# Package Manager & Scaffolding (end / endpkg)
end.exe new my_project                # Scaffold complete End project with end.toml
end.exe init                          # Initialize end.toml in current folder
end.exe add <package_name>            # Add dependency to end.toml
end.exe install                       # Install and lock all dependencies
end.exe publish                       # Publish package to End Central Registry

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

## 2. The 5 Official First-Party Enterprise Frameworks

| Framework | Import Path | Purpose |
| :--- | :--- | :--- |
| **🚀 EndHyper** | `std/hyper/hyper.end` | FastAPI/Actix-Web-like declarative web framework with `@get`, `@post`, `@put`, `@delete`, auto-validation, OpenAPI/Swagger JSON. |
| **🎮 EndForge** | `std/forge/forge.end` | RFC 6455 Native WebSocket engine, 120 FPS binary game state sync, broadcast channels. |
| **⚡ EndStream** | `std/stream/stream.end` | Zero-copy chunked file streaming, multipart uploads, Server-Sent Events (SSE). |
| **🔒 EndCrypto** | `std/crypto/jwt.end` | JWT tokens (`jwt_sign`, `jwt_verify`), Token Bucket Rate Limiter, SHA-256 security. |
| **💾 EndKV & EndORM** | `std/kv/kv.end` | 64-Byte cache-aligned in-memory key-value database, WAL persistence, zero-reflection ORM. |

---

## 3. Real-World Architecture Example

```end
import "std/hyper/hyper.end"
import "std/forge/forge.end"
import "std/crypto/jwt.end"
import "std/kv/kv.end"
import "std/time/time.end"

st UserLoginDto {
    username: str,
    password_hash: u64,
    device_id: i64,
}

// 1. Declarative Route Attribute (FastAPI Style)
@post("/api/v1/auth/login")
pub fn handle_login(req: RequestContext, body: UserLoginDto) HyperResponse {
    if body.username == "admin" {
        val jwt = jwt_sign(body.username, 3600)
        ret hyper_response_ok("{\"status\": \"authenticated\", \"token\": \"JWT_OK\"}")
    }
    ret hyper_response_unauthorized("Invalid Credentials")
}

// 2. Real-Time 120 FPS WebSocket Game Synchronizer
@ws("/ws/game/{room_id}")
pub fn game_socket_handler(client: WebSocketClient, room: str) void {
    val state = ws_pack_game_state(1001, 145.5, 320.0, 12.4, 60)
    ws_broadcast_state(client, state)
}

fn main() void {
    val start = instant_now()
    val db = kv_database_open("game_db")
    kv_set(db, "player:1001:session", "ACTIVE")

    val req = RequestContext { method: "POST", path: "/api/v1/auth/login", body: "", client_ip: "127.0.0.1" }
    val dto = UserLoginDto { username: "admin", password_hash: 12345, device_id: 1 }
    val res = handle_login(req, dto)
    
    println(res.body)
    println(instant_elapsed_micros(start))
}
```
