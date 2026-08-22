# 📜 Technical Documentation & Architectural Truth Classification Rule

Whenever writing, modifying, or refactoring code or documentation in the End Programming Language repository, always strictly follow these non-negotiable rules:

---

## 1. Continuous Documentation Synchronization
Whenever a new feature, statement keyword, AST node, semantic pass, or standard library module is added or updated in `endc`:
- All 12 documents in the documentation hierarchy (`README.md`, `docs/VISION.md`, `docs/FEATURES.md`, `docs/AGENT_NATIVE.md`, `docs/AGENT_CONTRACT_SYSTEM.md`, `docs/OPERATION_VALUES_AND_ALGEBRA.md`, `docs/MEMORY_MODEL.md`, `docs/ARCHITECTURE.md`, `docs/PACKAGES.md`, `docs/FRAMEWORKS.md`, `docs/ROADMAP.md`, `docs/STATUS.md`) must be reviewed and updated concurrently.
- No code feature is considered complete until its formal specification, code examples, maturity badge, and verification status are documented.

---

## 2. Canonical Vision & Positioning Invariant (Interpretation C)
- Always position End as: **«A high-performance, memory-safe, modular, general-purpose programming language designed from the ground up for AI-assisted and agentic software development.»**
- Explicitly reinforce:
  - **«One Language. Every Domain.»** (Full spectrum: systems, backends, web, mobile, desktop, games, graphics, AI infrastructure, embedded).
  - **«All the good parts, without the unnecessary complexity.»** (Selective synthesis of C, Rust, Zig, Go, Swift, Kotlin, Python, and functional paradigms).
- Explicitly reject reductive positioning: End is **not** only a game language, systems language, Rust/Zig alternative, or AI wrapper.

---

## 3. Strict 6-Tier Feature Maturity Classification
Every documented feature must carry a formal maturity status:
- 🟢 **Stable:** Implemented, verified in `endc/src/ir/tests.rs` (41+ tests passing), benchmarked.
- 🔵 **Beta:** Core implementation complete; API stabilization in progress.
- 🟡 **Experimental:** Functionally working in compiler/VM; semantics subject to refinement.
- 🟠 **Alpha:** Initial prototype; foundational AST/lowering tests passing.
- 🟣 **Research:** Formal specification and conceptual design under active exploration.
- ⚪ **Planned:** On official roadmap with design RFC in progress.

---

## 4. Truth Classification & Anti-Marketing Integrity
- Never claim a roadmap item or research design is implemented without verifiable source code in the repository.
- Avoid unsupported superlatives ("fastest language in the world", "zero bugs", "better than all languages").
- Maintain transparent, reproducible benchmark reports with hardware parameters, OS, compilation flags, checksums, and source code links.
