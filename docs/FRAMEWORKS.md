# 📦 End Language Exclusive Framework Ecosystem

The End standard library provides 5 specialized, high-performance framework modules:

---

## 1. ⚡ EndHyper (High-Throughput Web Framework)
- Declarative routing with `@get`, `@post`, `@put`, `@delete`.
- Zero-reflection compile-time DTO validation.
- Built-in HTTP connection pooling and keep-alive.

```end
import "std/hyper/router.end"

@post("/api/v1/checkout")
pub fn handle_checkout(req: RequestContext) HyperResponse {
    ret hyper_response_ok("{\"status\": \"paid\"}")
}
```

---

## 2. 🎮 EndForge (120 FPS Binary WebSockets)
- Low-latency binary framing for real-time multiplayer games and financial streaming.
- Sub-millisecond message broadcasting with 0 memory copies.

```end
import "std/forge/websocket.end"

@ws("/ws/arena/{room_id}")
pub fn handle_arena_socket(client: WebSocketClient, room: str) void {
    val state = ws_pack_state(1001, 240.5, 310.0, 120)
    ws_broadcast(client, state)
}
```

---

## 3. 🛡️ EndNexus (Game Physics & Hardware Watchdog)
- 2D/3D physics collision matrix.
- `SocketGuard` dual-check validation with exponential backoff.
- Thermal circuit breaker with automatic CPU load throttling (> 85%).

---

## 4. 🔒 EndCrypto (Constant-Time Cryptography)
- Hardware-accelerated AES-GCM, ChaCha20-Poly1305, SHA-256, Ed25519.
- Constant-time memory comparisons immune to timing side-channel attacks.

---

## 5. 💾 EndKV (Zero-Allocation Key-Value Store)
- High-concurrency in-memory storage engine backed by 64-byte aligned arena tables.
- Sub-microsecond read/write operations (P99 < 15 ns).
