# ?? End Language — Agent Engineering Principles & Definition of Done (DoD)

This document establishes the binding rules of development, verification, and task completion across the entire **End Language** repository and compiler toolchain (`endc`).

---

## ?? Fundamental Agent Law

A task or checkbox may **ONLY** be marked as complete `[x]` when all 8 completion criteria are satisfied with verifiable evidence:

1. **Implementation Exists:** Real, sound code is written in the compiler, runtime, or standard library. No stubs, mocks, or placeholders.
2. **Regression Tests Exist:** Unit, integration, and compilation test cases are in place.
3. **Adversarial / Negative Tests Exist:** Negative cases (e.g. `compile_fail` tests for `E0906`, `E0907`, `E0903`, `E0904`) actively test error detection.
4. **CI Matrix Passes:** `cargo build --release`, `cargo test`, and `end test` pass cleanly with zero failures.
5. **Reproducible Behavior:** Every performance or functional claim can be reproduced with a single command on standard hardware.
6. **Documentation Matches Reality:** All READMEs, docs, and specs accurately describe the current implementation state (`0.4.0-alpha`).
7. **Zero Silent TODOs/FIXMEs:** No silent `todo!()`, `unimplemented!()`, or unhandled placeholders remain in user-facing code paths.
8. **Evidence Recorded:** Changed files, executed commands, and test outputs are documented in reports and commit logs.

---

## ??? Versioning & Truth Policy

- **Current Version:** `0.4.0-alpha`
- **Compiler Channel:** Alpha Toolchain (High-Performance C11 / LLVM IR / Instant Interpreter)
- No unproven marketing claims (such as "v1.0" or fabricated throughput numbers) may be presented as verified fact without a reproducible benchmark suite.

---

## ?? Verification Matrix

| Area | Verification Command | Expected Output |
| :--- | :--- | :--- |
| **Compiler Units** | `cargo test` | 5/5 tests pass (`HIR`, `MIR`, `Move`, `Borrow`, `Purity`) |
| **Compiler Build** | `cargo build --release` | Zero errors, valid release binary |
| **Standard Suites** | `end test <suite>` | 8 enterprise suites pass (34+ scenarios) |
| **Standard Library** | `end check <module>` | 0 errors, 0 warnings across all `std/` modules |
| **Security Fuzzing** | `end fuzz <module>` | Real byte mutations, memory safety verified |
