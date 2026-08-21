# 👑 End Language — Compiler Configuration & Architectural Linter Guardrails

> **Deterministic Architectural Integrity, Code Modularity Limits, and Vibe-Coding AI Guardrails.**  
> *Configure project-wide architectural invariants in `end.config.toml` and let the End compiler guarantee code quality at compile-time.*

---

## 1. Overview: The Vibe-Coding Challenge

When developing applications rapidly with AI Coding Agents ("Vibe Coding"), two major failure modes occur:
1. **Mega-Files & Monolithic Sprawl:** AI agents append code indefinitely, resulting in 2,000+ line unmaintainable files.
2. **Architectural Drift:** Inconsistent naming conventions, broken layer boundaries (e.g. UI directly querying SQL), chaotic commenting styles, and complex 15-level nested `if/else` ladders.

**The End Language solves this fundamentally at the compiler level.**  
By configuring `end.config.toml`, the compiler acts as a strict architectural enforcer. Any violation is caught at build-time with actionable auto-repair instructions.

---

## 2. Configuration Schema: `end.config.toml`

Initialize a new config file in your project with:
```bash
end config-init
```

### Full Configuration Reference:

```toml
# 👑 End Language Compiler & Architecture Configuration

[architecture]
pattern = "clean"                    # Architectural pattern: clean | mvc | mvvm | hexagonal | none
enforce_layers = true                # Strict layer isolation (Domain cannot depend on Presentation)
layers = ["domain", "data", "presentation"]

[files]
max_lines = 200                      # Maximum allowed lines per file (forces modularization)
max_functions_per_file = 10          # Maximum functions allowed per single file
max_function_lines = 50              # Maximum statements/lines per function
max_params = 5                       # Maximum parameters per function

[comments]
allowed = true                       # Allow comments in codebase (false = zero-comment policy)
language = "en"                      # Comment language: en | fa | any
require_doc_comments = true          # Require documentation comments on pub functions

[naming]
struct_style = "PascalCase"          # Struct naming convention: PascalCase | snake_case
function_style = "snake_case"        # Function naming convention: snake_case | camelCase
variable_style = "snake_case"        # Variable naming convention: snake_case | camelCase

[quality]
max_cyclomatic_complexity = 10       # Maximum cyclomatic complexity per function (1-20)
no_dead_code = true                  # Warn/error on unused functions
no_unused_imports = true             # Warn/error on unused imports
```

---

## 3. CLI Subcommands

### A. Initialize Configuration
```bash
end config-init
```
Generates a pre-configured, production-ready `end.config.toml` in the current root directory.

### B. Project-Wide Linter Inspection
```bash
end lint .
# Or lint a specific module:
end lint src/features/auth.end
```

**Human-Readable Terminal Output:**
```
╔══════════════════════════════════════════════════════════════╗
║         ⚙️  END COMPILER CONFIG VIOLATIONS                  ║
╚══════════════════════════════════════════════════════════════╝
  ❌ ERROR [files.max_params] auth.end:15 — Function 'handle_login' has 6 parameters, exceeding maximum of 5.
         💡 Fix: Group parameters of 'handle_login' into a struct or config object.
  ❌ ERROR [naming.function_style] auth.end:15 — Function 'handleLogin' violates naming convention: expected snake_case.
         💡 Fix: Rename to 'handle_login'.
  ❌ ERROR [naming.struct_style] models.end:10 — Struct 'user_record' violates naming convention: expected PascalCase.
         💡 Fix: Rename to 'UserRecord'.

  📊 Total: 3 error(s), 0 warning(s)
```

### C. Machine-Readable JSON for AI Coding Agents
```bash
end lint . --json
```

**Structured JSON Output:**
```json
[
  {
    "file": "src/features/auth.end",
    "line": 15,
    "rule": "files.max_params",
    "severity": "error",
    "message": "Function 'handle_login' has 6 parameters, exceeding maximum of 5.",
    "suggestion": "Group parameters of 'handle_login' into a struct or config object."
  },
  {
    "file": "src/features/auth.end",
    "line": 15,
    "rule": "naming.function_style",
    "severity": "error",
    "message": "Function 'handleLogin' violates naming convention: expected snake_case.",
    "suggestion": "Rename to 'handle_login'."
  }
]
```

---

## 4. Automatic Build-Time Enforcement

When `end.config.toml` exists in the workspace root, all standard compilation commands (`end build`, `end run`, `end check`) automatically execute the linter pass. If any rule with `severity: error` is violated, compilation is rejected immediately, preventing unmaintainable code from ever reaching binaries or git repositories.

```bash
$ end build src/main.end
Error: end.config.toml policy violation in 'src/main.end'
  ❌ ERROR [files.max_lines] main.end: File has 240 lines, exceeding maximum of 200.
         💡 Fix: Split this file into smaller modules. Each module should have ≤200 lines.
```

---

## 5. Linter Rules Catalogue

| Rule ID | Configuration Key | Description | Default Limit |
|---|---|---|---|
| `files.max_lines` | `[files] max_lines` | Rejects files exceeding the line budget. | 500 lines |
| `files.max_functions_per_file` | `[files] max_functions_per_file` | Prevents kitchen-sink files. | 20 functions |
| `files.max_function_lines` | `[files] max_function_lines` | Caps statement count per function. | 100 statements |
| `files.max_params` | `[files] max_params` | Forces grouping parameters into structs. | 8 parameters |
| `comments.allowed` | `[comments] allowed` | Enables or disables comments project-wide. | `true` |
| `comments.language` | `[comments] language` | Enforces comment language (`en`, `fa`, `any`). | `any` |
| `naming.struct_style` | `[naming] struct_style` | Enforces `PascalCase` or `snake_case` on structs. | `PascalCase` |
| `naming.function_style` | `[naming] function_style` | Enforces `snake_case` or `camelCase` on functions. | `snake_case` |
| `quality.max_cyclomatic_complexity` | `[quality] max_cyclomatic_complexity` | Caps branching complexity (`if`/`while`/`match`). | 15 |
