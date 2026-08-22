# 📦 End Language — Standard Framework & Library Ecosystem

> **High-Performance Native Frameworks for Web, Real-Time Networking, Cryptography, Gaming, and In-Memory Storage.**  
> *End includes an integrated suite of domain-specific frameworks designed for extreme performance, memory safety, and seamless AI agent pair programming.*

---

## 1. ⚡ EndHyper — High-Throughput Web Framework

**EndHyper** is a zero-reflection, declarative web and microservices framework engineered for high-concurrency cloud backends:

- **Declarative Route Annotations:** `@get`, `@post`, `@put`, `@delete`, `@patch`.
- **Zero-Reflection DTOs:** Compile-time serialization without runtime reflection overhead.
- **Connection Pooling & Keep-Alive:** Built-in HTTP connection pools with non-blocking I/O.
- **Capability Contracts:** Explicitly audits network and memory permissions per handler.

```end
import "std/hyper/router.end"
import "std/hyper/response.end"

st OrderRequest {
    item_id: i64,
    quantity: i64,
    customer_email: str,
}

@post("/api/v1/orders")
@capability(net = true, disk = false, memory = "ArenaScoped")
pub fn handle_create_order(req: RequestContext, order: OrderRequest) HyperResponse {
    region order_scope {
        val confirmation_id = 99841;
        val json_response = "{\"status\": \"created\", \"order_id\": " + confirmation_id + "}";
        ret hyper_response_json(201, json_response);
    }
}
```

---

## 2. 🎮 EndForge — Sub-Millisecond Binary WebSockets

**EndForge** provides low-latency binary framing for real-time multiplayer games, financial order feeds, and live collaborative applications:

- **Zero-Copy Message Broadcasting:** Transmits binary state buffers across thousands of clients without duplicate memory copies.
- **Microsecond Latency:** Handles frame updates at 120 FPS with P99 broadcast latency < 1 ms.

```end
import "std/forge/websocket.end"

@ws("/ws/arena/{room_id}")
pub fn handle_arena_stream(client: WebSocketClient, room_id: str) void {
    val state = ws_pack_player_state(1001, 240.5, 310.0, 120);
    ws_broadcast_binary(room_id, state);
}
```

---

## 3. 🛡️ EndNexus — Game Physics & Hardware Protection

**EndNexus** combines real-time 2D/3D physics simulation with enterprise-grade hardware protection:

- **2D/3D Collision Matrix:** Fast SIMD-accelerated raymarching, bounding box checks, and rigid body dynamics.
- **SocketGuard:** Dual-check socket validation with exponential backoff against network flapping.
- **Thermal Circuit Breaker:** Automatically throttles CPU load when sustained execution exceeds safety thresholds (> 85% core saturation).

```end
import "std/nexus/socket_guard.end"
import "std/nexus/circuit_breaker.end"
import "std/nexus/physics2d.end"

pub fn update_game_loop(entities: *EntityList, delta_time: f64) void {
    region frame_scope {
        val collisions = physics_solve_collisions(frame_scope, entities, delta_time);
        nexus_thermal_guard(); // Enforces CPU yield if core temperature spikes
    }
}
```

---

## 4. 🔒 EndCrypto — Constant-Time Cryptographic Suite

**EndCrypto** provides side-channel-resistant cryptographic primitives implemented strictly in constant-time:

- **Hardware Acceleration:** Native AES-GCM (AES-NI), SHA-256, ChaCha20-Poly1305, and Ed25519 signatures.
- **Constant-Time Verification:** Immune to timing attacks and cache-leakage vulnerabilities.

```end
import "std/crypto/sha256.end"
import "std/crypto/ed25519.end"

pub fn verify_signature(public_key: *u8, message: str, signature: *u8) bool {
    ret ed25519_verify_constant_time(public_key, message, signature);
}
```

---

## 5. 💾 EndKV — Zero-Allocation Key-Value Store

**EndKV** is an in-memory embedded storage engine backed by 64-byte cache-line aligned arena tables:

- **Sub-Microsecond Operations:** Read/write operations with P99 latency < 15 nanoseconds.
- **Lock-Free Concurrency:** Lock-free Single-Producer Single-Consumer (SPSC) ring buffers for concurrent event streaming.

```end
import "std/kv/store.end"

pub fn cache_session(session_id: str, user_id: i64) void {
    endkv_put_fast(session_id, user_id, 3600); // 1-hour TTL
}
```

---

## 6. 🎨 EndUI / Canvas — 120 FPS Native Graphics

**EndUI** provides a hardware-accelerated 120 FPS graphics pipeline (`std/ui/canvas.end`):

- **Hardware SIMD Vector Graphics:** Anti-aliased line rendering, glassmorphic blur effects, and real-time shader pipelines.
- **VS Code Extension Integration:** Live interactive 120 FPS visual preview in VS Code webviews.

---

## 7. Ecosystem Framework Roadmap

| Framework | Domain | Status | Planned Release |
| :--- | :--- | :---: | :---: |
| **EndHyper** | Cloud Web & REST APIs | 🟢 Stable | v0.4.0 |
| **EndForge** | Real-time WebSockets & Game Feeds | 🟢 Stable | v0.4.0 |
| **EndNexus** | Physics & Hardware Protection | 🟢 Stable | v0.4.0 |
| **EndCrypto** | Cryptography & Signatures | 🟢 Stable | v0.4.0 |
| **EndKV** | In-Memory Key-Value Engine | 🔵 Beta | v0.4.5 |
| **EndUI / Canvas** | 120 FPS Native GUI & Games | 🟢 Stable | v0.4.0 |
| **EndRaft** | Distributed Cluster Consensus | 🟡 Experimental | v0.5.0 |
| **EndTensor** | Machine Learning & Tensor Engine | 🟡 Experimental | v0.5.0 |
| **EndMobile** | Native iOS & Android Framework | ⚪ Planned | v0.6.0 |
