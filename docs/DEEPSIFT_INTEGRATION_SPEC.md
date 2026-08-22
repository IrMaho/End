# 🧠 DeepSift Cortex Integration Specification
## Zero-Dependency Native Compiler + High-Level Neural Cortex Architecture

---

## 🌟 Philosophy & Architectural Separation

```
+--------------------------------------------------------------------+
|                         DeepSift Cortex                            |
|  - Multi-agent orchestration & goal decomposition                  |
|  - Semantic graph querying & prompt context assembly (DEC_v2)      |
|  - Neural code synthesis & speculative patch generation            |
|  - Dynamic Research Memory (DRM) persistence & cross-task learning |
+--------------------------------------------------------------------+
                               ▲ │
          JSON IPC Protocol    │ │  Command-Line / Stdout Streaming
          (Zero DB Bloat)      │ │  (100% Deterministic)
                               │ ▼
+--------------------------------------------------------------------+
|                         Native End Compiler                        |
|  - Zero-GC Lexer, Parser, & Type Checker (<1ms per file)           |
|  - Formal @skill & @contract invariant verification                |
|  - Blast radius & pre-touch boundary analysis (ImpactGuard)        |
|  - Static AST security auditor (CWE-798, CWE-285, CWE-119)         |
|  - Cryptographically signed Verified Commit Manifests              |
+--------------------------------------------------------------------+
```

DeepSift operates externally to the native compiler:
1. DeepSift never links native compiler binary into Python/Node runtimes or bundles heavy vector databases inside `endc`.
2. The End Compiler exposes high-performance CLI subcommands (`end semantic-ir`, `end precheck`, `end context`, `end verify`, `end security`, `end memory`, `end semantic-git`) that return structured, machine-parseable JSON.
3. DeepSift parses this JSON to make informed architectural decisions with formal compiler-guaranteed safety.

---

## 📡 End Semantic IR Schema (`end semantic-ir`)

When DeepSift calls `end semantic-ir <file.end> --json`, the native compiler outputs the following canonical IR:

```json
{
  "type_graph": {
    "types": [
      {
        "name": "PaymentRequest",
        "kind": "Struct",
        "fields": [
          {"name": "order_id", "field_type": "str"},
          {"name": "amount_cents", "field_type": "i64"},
          {"name": "idempotency_key", "field_type": "str"}
        ],
        "is_exported": false
      }
    ],
    "type_dependencies": []
  },
  "symbol_graph": {
    "symbols": [
      {
        "name": "process_payment",
        "kind": "Function",
        "signature": "process_payment(req: PaymentRequest) bool",
        "file": "payment_service.end",
        "line": 15,
        "is_exported": true,
        "is_pure": false,
        "capabilities": ["network", "disk_io"]
      }
    ],
    "call_matrix": [
      {
        "caller": "checkout_handler",
        "callee": "process_payment",
        "frequency": 1,
        "is_critical_path": true
      }
    ]
  },
  "contract_graph": {
    "contracts": [
      {
        "symbol": "process_payment",
        "skills_required": ["PaymentSafe"],
        "invariants": ["idempotent=true", "audit=true"]
      }
    ]
  },
  "resource_graph": {
    "allocations_count": 0,
    "pure_symbols_count": 0,
    "io_symbols_count": 1,
    "network_symbols_count": 1
  }
}
```

---

## ⚡ Smart Context Extraction (DEC_v2 Algorithm)

When an AI agent is tasked with a prompt like `"Apply discount voucher to checkout"`, passing 50,000 LOC causes prompt bloat, high cost, and hallucinated types.

`end context <file.end> "<task_intent>" --budget 500` executes DEC_v2:
1. **Keyword Extraction**: Parses intent tokens (`discount`, `voucher`, `checkout`).
2. **Seed Symbol Matching**: Identifies core target functions in `SemanticGraph`.
3. **Neighborhood Traversal**: Gathers direct dependencies (structs, enums, referenced types).
4. **Skeletal Code Synthesis**: Emits full struct definitions and enum variants, but strips internal implementation bodies of non-seed functions, retaining only typed signatures and `@contract` decorators.
5. **Budget Enforced Pruning**: Guarantees the generated context payload fits comfortably within the token budget.

```
+-------------------------------------------------------------+
| 50,000 LOC Project                                          |
|  -> DEC_v2 Slicing Algorithm                                |
|  -> 180 Tokens (99.6% Token Compression Ratio)              |
+-------------------------------------------------------------+
```

---

## 🛡️ Pre-Touch Safety Gating (`end precheck`)

Before generating code changes, DeepSift invokes `end precheck <file> <symbol>`:
- If `can_proceed_safely` is `false`, DeepSift halts and prompts the user or modifies the plan.
- If `risk_level` is `CRITICAL`, DeepSift automatically activates deep multi-hypothesis DRM reasoning.

---

## 🔐 Verified Commits Protocol (`end semantic-git`)

DeepSift never commits unverified code to git repositories. It produces an immutable **Verified Commit Manifest**:

```json
{
  "commit_hash": "end-commit-41f8bff60ccc8c8e",
  "timestamp_ms": 1771675200000,
  "agent_id": "deepsift-cortex-01",
  "task_id": "task-9001",
  "requirement": "Implement idempotent payment engine",
  "skills_applied": ["PaymentSafe"],
  "tests_passed": 1,
  "total_tests": 1,
  "security_passed": true,
  "contracts_verified": true,
  "verification_signature": "proof-of-work-sig-41f8411bf6024670"
}
```
Commit validation fails if:
- Unit tests fail (`tests_passed < total_tests`).
- AST security auditor flags any critical/high CWE vulnerabilities.
- Formal skill contracts are violated (e.g. missing audit logging in `PaymentSafe`).
