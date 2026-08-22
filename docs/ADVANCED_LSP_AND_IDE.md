# 🛠️ End Language — Advanced IDE & Language Server Protocol (LSP 3.17)
## Semantic Token Highlighting, Inlay Hints, AutoHeal Code Actions & Incremental Sync

---

## 🌟 LSP 3.17 Implementation

The End Language Server (`endc/src/lsp/mod.rs`) implements full modern Language Server Protocol specifications:

- **Semantic Tokens**: Rich token legend encoding for types, contracts, invariants, keywords, and mutable variables.
- **Inlay Hints**: Automatic type annotation hints for inferred variables (`val total /*: i64*/ = 100`).
- **Code Actions & Quick-Fixes**: Seamless integration with the compiler's `AutoHeal` engine to suggest and apply code fixes with 1 click.
- **Project-Wide Rename**: Deterministic AST-aware symbol renaming across all source files.
