---
name: lossless-modular-refactor
description: Comprehensive workflow for autonomous, lossless modularization and SOLID refactoring of monolithic source files into cohesive submodules strictly under 500 lines.
---

# Lossless Modular Refactoring Workflow

## Step 1: Pre-Refactoring Baseline & Backup
1. Record original line count and create an immutable backup (`src/<module>.rs.bak`).
2. Map all top-level symbols (structs, enums, functions, traits, impl blocks, match arms).

## Step 2: Subsystem Architectural Decomposition
1. Group symbols by cohesive responsibility:
   - `types.rs` / `tokens.rs`: Structs, Enums, and data definitions.
   - `facets/` / `decl/`: Domain-specific declaration groups.
   - `engine/` / `driver.rs`: Orchestration, lifecycle, and drivers.
   - `reporter.rs` / `format.rs`: Output formatting, UI, and diagnostics.
2. Ensure every planned submodule will be strictly ≤ 500 lines.

## Step 3: Generation & Symbol Extraction
1. Preserve all `#[derive(...)]` attributes, docstrings, and comments.
2. Resolve internal cross-submodule imports with `use super::...;` or `use crate::...;`.
3. Create the facade `mod.rs` re-exporting all public symbols.

## Step 4: Verification & Conservation Audit
1. Run `cargo check` and resolve any missing imports or visibility errors.
2. Run `cargo test` to confirm 100% test pass rate with zero regressions.
3. Execute line-count audit across all files in the subsystem to guarantee `lines <= 500`.
4. Run symbol conservation script comparing original backup symbols against new module symbols.
5. Clean up backup and generator files, then stage and commit to git.
