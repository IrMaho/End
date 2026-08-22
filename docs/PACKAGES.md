# 📦 End Language — Package & Module System Architecture

> **First-Class Packages for Humans and AI Agents.**  
> *Deterministic Dependency Resolution, Machine-Readable Manifests, Capability Contracts, and Composable Modularity.*

---

## 1. The Core Philosophy

In End, a package is not merely a collection of source files; it is a **semantically bounded unit of capability, architecture, and verification**.

Both human developers and AI coding agents can discover:
- Package public API and exported types
- Side-effect and system capabilities (`net`, `io`, `ffi`, `alloc`)
- Memory and latency contracts
- Target platform constraints (Windows, Linux, macOS, Embedded)
- Formal test and verification suites

```text
               ┌────────────────────────────────────────┐
               │           End Package Manifest         │
               │            (end.config.toml)           │
               └───────────────────┬────────────────────┘
                                   │
       ┌───────────────────────────┼───────────────────────────┐
       ▼                           ▼                           ▼
┌──────────────┐           ┌──────────────┐           ┌──────────────────┐
│ Module Graph │           │ Capability   │           │ Agent Context    │
│  & Exports   │           │ Constraints  │           │ & Firewalls      │
└──────────────┘           └──────────────┘           └──────────────────┘
```

---

## 2. Package Manifest Specification (`end.config.toml`)

Every End project contains an `end.config.toml` at its root:

```toml
[package]
name = "hyper_service"
version = "0.4.0"
description = "High-performance microservice engine built with End"
authors = ["Mohammad Javad <contact@endlang.org>"]
license = "MIT"
edition = "2026"

[dependencies]
# Local standard library modules
"std/time" = { path = "std/time" }
"std/crypto" = { path = "std/crypto" }
"std/hyper" = { path = "std/hyper", features = ["websocket", "tls"] }

# External git dependencies
"end_tensor" = { git = "https://github.com/IrMaho/end_tensor.git", branch = "main" }

[targets]
bin = "src/main.end"
output = "dist/hyper_service.exe"
optimization = "ReleaseFast" # Debug, ReleaseFast, ReleaseSmall, ReleaseSafe

[capabilities]
# Statically audited by 'end effects' and 'end arch check'
allow_net = true
allow_disk_io = false
allow_raw_ffi = false
max_memory_mb = 64

[agent]
# Context limits for AI coding assistants
max_context_tokens = 16000
agent_boundary = true
verification_level = "formal_and_evidence"
```

---

## 3. Module System & Imports

### A. Importing Modules
Modules are imported using explicit, unambiguous path specifiers:
```end
import "std/time/time.end"
import "std/crypto/sha256.end"
import "services/payment_service.end"

pub fn main() void {
    val hash = sha256_hash("hello world");
}
```

### B. Visibility & Exports
Symbols are private by default. The `pub` keyword explicitly exports functions, structs, and constants:
```end
// Private to this file
fn internal_helper() i64 {
    ret 42
}

// Exported public API
pub fn calculate_price(base: i64) i64 {
    ret base + internal_helper()
}
```

### C. Agent Contexts & Module Firewalls
To prevent AI models from hallucinating against private internals or leaking security tokens, modules can specify explicit agent visibility:
```end
agent_context PaymentGateway {
    expose: ["charge", "refund", "get_status"];
    hide: ["raw_api_secrets", "internal_retry_state"];
}

context_firewall SecurityVault {
    deny: ["direct_memory_access", "unverified_ffi"];
    expose: ["sanitized_crypto_interface"];
}
```

---

## 4. Feature Flags & Target Architecture

End supports zero-overhead compile-time feature toggling:

```end
@cfg(feature = "websocket")
pub fn init_websocket_server() void {
    // Compiled only when 'websocket' feature is enabled in manifest
}

@cfg(target_os = "windows")
pub fn get_platform_handle() i64 {
    // Windows-specific implementation
}

@cfg(target_os = "linux")
pub fn get_platform_handle() i64 {
    // Linux-specific implementation
}
```

---

## 5. Global Decentralized Package Registry *(Planned ⚪)*

The long-term roadmap for End includes a cryptographically-verified, decentralized package repository:
- **`end package install <name>`**: Fast, deterministic package fetching with checksum lockfiles (`end.lock`).
- **Reproducible Builds**: Bit-for-bit identical binary output across all operating systems.
- **Supply-Chain Verification**: Mandatory cryptographic signing of all published packages with vulnerability scanning.
