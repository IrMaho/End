# 🌳 Semantic Git & Verified Commits Specification
## AST-Aware Version Control & Cryptographic Proof-of-Work Manifests

---

## 🌟 Evolution from Line Diffs to Semantic Symbol Diffs

Traditional git diffs operate on raw text lines. They cannot distinguish between:
- Harmless formatting/whitespace changes.
- Breaking API signature changes affecting 20 downstream callers.
- Security vulnerability introductions.
- Formal `@skill` contract regressions.

**End Semantic Git** (`end semantic-git`) evaluates changes at the AST and semantic graph level.

---

## 🔍 Semantic Diff Output Schema (`end semantic-git diff`)

```json
{
  "file": "temp_banking_service.end",
  "symbol_deltas": [
    {
      "symbol": "process_payment",
      "delta_type": "Modified",
      "old_signature": "process_payment(req: PaymentRequest) bool",
      "new_signature": "process_payment(req: PaymentRequest, timeout_ms: i64) bool",
      "callers_impacted": 3
    }
  ],
  "architecture_status": "✔ Architectural Invariants 100% Preserved",
  "contract_adherence": "✔ All @skill & @contract Invariants Satisfied",
  "security_status": "✔ Zero Security Vulnerabilities Detected",
  "behavior_summary": "Deterministic implementation update with machine-verified proof of work"
}
```

---

## 🔐 Cryptographically Signed Verified Commits (`end semantic-git commit`)

Every commit produced by an autonomous agent carries a cryptographic proof-of-work signature confirming:
1. **100% Tests Passed**: All unit and integration tests executed in the End VM.
2. **0 Security Vulnerabilities**: AST scanner verified zero CWE-798, CWE-285, or CWE-119 issues.
3. **0 Skill Contract Violations**: Verified all `@skill` and `@contract` formal constraints.

```json
{
  "is_valid": true,
  "manifest": {
    "commit_hash": "end-commit-aec033a453cd1247",
    "timestamp_ms": 1771675200000,
    "agent_id": "autonomous_agent_01",
    "task_id": "task-183",
    "requirement": "Implement idempotent payment engine",
    "skills_applied": ["PaymentSafe"],
    "files_changed": ["temp_banking_service.end"],
    "semantic_diff": { ... },
    "tests_passed": 1,
    "total_tests": 1,
    "security_passed": true,
    "contracts_verified": true,
    "compiler_hash": "endc-v2.0.0-verified",
    "verification_signature": "proof-of-work-sig-aec0cd49a903d8b9"
  },
  "rejected_reasons": []
}
```

### Rejection Policy

If an agent attempts to commit code with failing tests or violated skills:
```
✖ Commit Rejected:
  ✖ Rejected: Only 0/1 unit tests passed.
  ✖ Rejected: Formal contract / skill verification failed with hard constraint violations.
```
