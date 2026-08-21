# 👑 The End Programming Language: Master Developer & Enterprise Guide

> **Hybrid 3-Tier Memory • Bare-Metal Speed • Zero GC • Direct @import_c • Fiber Concurrency • 120 FPS WebSockets • FastAPI-like Web Routing**

---

## 1. Quick Start & Package Manager Commands

The compiler and package manager are available everywhere in Windows PATH (`end.exe`, `endc.exe`, `endpkg`):

```powershell
# Project & Package Management (end / endpkg)
end.exe new my_enterprise_app         # Scaffold complete project with end.toml
end.exe init                          # Initialize end.toml in current directory
end.exe add end_hyper                 # Add dependency to manifest
end.exe install                       # Install and lock all dependencies
end.exe publish                       # Publish package to End Central Registry

# Execution & Native Compilation
end.exe run main.end                  # Run immediately via VM
end.exe build main.end -o app.exe     # Compile to native ultra-optimized binary (.exe)
end.exe build mathlib.end --dll -o mathlib.dll  # Compile to DLL + C Header (.h)

# AI Self-Healing & Semantic Introspection
end.exe fix app.end --apply           # AI Self-Healing compiler auto-patching
end.exe inspect app.end --line 10     # Deep semantic introspection
end.exe explain app.end --line 10     # Natural language explanation
end.exe trace app.end --symbol x      # Symbol lifecycle data-flow
end.exe impact app.end --symbol fn    # Blast-radius analysis
```

---

## 2. The 5 Official First-Party Enterprise Frameworks

| Framework | Import Path | Key Capabilities |
| :--- | :--- | :--- |
| **🚀 EndHyper** | `std/hyper/hyper.end` | FastAPI/Actix-Web-like declarative web framework with `@get`, `@post`, `@put`, `@delete`, auto-validation, OpenAPI/Swagger generation. |
| **🎮 EndForge** | `std/forge/forge.end` | RFC 6455 Native WebSocket engine, 120 FPS binary game state sync, broadcast channels, sub-microsecond latency. |
| **⚡ EndStream** | `std/stream/stream.end` | Zero-copy chunked file streaming, multipart form uploads, Server-Sent Events (SSE). |
| **🔒 EndCrypto** | `std/crypto/jwt.end` | JWT tokens (`jwt_sign`, `jwt_verify`), Token Bucket Rate Limiter, SHA-256 security. |
| **💾 EndKV & EndORM** | `std/kv/kv.end` | 64-Byte cache-aligned in-memory key-value database, WAL persistence, zero-reflection ORM. |

---

## 3. Real-World Enterprise Example: Web + 120 FPS WebSocket Game Server

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
