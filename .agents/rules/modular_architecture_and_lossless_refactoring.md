# Modular Architecture & Lossless Refactoring Standards

## 1. Hard File Size Invariant (≤ 500 Lines)
- **Hard Limit**: No source file may exceed 500 lines under any circumstances.
- **Target Size**: Prefer focused, cohesive submodules between 50 and 350 lines.
- **Proactive Modularization**: When adding new features or expanding existing code, if a file approaches ~400 lines, immediately decompose it into a cohesive subsystem folder.

## 2. Lossless Refactoring & Code Conservation
- **Zero Deletion Policy**: Never delete or drop functions, structs, enums, variants, fields, match arms, error handling, or docstrings during restructuring.
- **Derives & Attributes**: Ensure all `#[derive(...)]`, `#[allow(...)]`, and macro annotations are strictly preserved on extracted types.
- **Line Conservation**: Account for all lines of code and verify that line differences are strictly due to layout/formatting, not dropped logic.

## 3. Public API & Facade Contract
- **Seamless Re-exports**: In `mod.rs`, re-export all public symbols (`pub use submodule::*;`) so that external modules import types identically without broken paths.
- **Visibility Scoping**: Use `pub(crate)` for internal helpers across sibling submodules, and `pub` for public API contracts.

## 4. SOLID & Clean Architecture
- **Single Responsibility (SRP)**: Divide modules by architectural concern (e.g. types/definitions, drivers/runners, formatters/reporters, core algorithms).
- **Open/Closed (OCP)**: Factor tables, matchers, and handlers into dedicated extensible modules.
- **Interface Segregation (ISP)**: Keep interfaces and traits focused and cohesive.

## 5. Mandatory Verification Gate
- **Compiler Check**: Run `cargo check` to verify zero compilation errors.
- **Test Suite**: Run `cargo test` to verify 100% passing tests without any regression.
- **Line Count Audit**: Automatically verify that all resulting files are strictly ≤ 500 lines before committing.
