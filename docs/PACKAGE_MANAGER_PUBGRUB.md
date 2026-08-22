# 📦 End Language — PubGrub SAT Package Manager & Workspace System
## Next-Gen Dependency Resolution, Git Sources, Cryptographic Signatures, and Monorepos

---

## 🌟 PubGrub SAT Dependency Solver

The End Package Manager (`endc/src/package/solver.rs`) implements the state-of-the-art PubGrub SAT dependency resolution algorithm:

- **Semantic Version Range Solving**: Correctly resolves `^1.2.0`, `~2.0.0`, `>=1.0.0, <3.0.0`.
- **Git Dependencies**: Pulls direct repository dependencies via `git+https://github.com/user/repo`.
- **SHA-256 Checksums**: Deterministic, tamper-proof lockfiles (`end.lock`).
- **Monorepo Workspaces**: Unified dependency management via `end.work.toml`.

---

## 🛠️ Configuration Example (`end.config.toml`)

```toml
[package]
name = "my_service"
version = "1.0.0"

[dependencies]
http_server = "^2.1.0"
jwt_auth = "~1.4.0"
crypto_vault = { git = "https://github.com/end-lang/crypto_vault", tag = "v1.0" }

[workspace]
members = [
    "crates/core",
    "crates/gateway",
    "crates/db"
]
```
