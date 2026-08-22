# 🚨 Permissioned Agent Security & AST Capability Guard
## Fine-Grained Agent Sandboxing & Static Vulnerability Auditing

---

## 🌟 The Need for Agent Boundaries

Giving autonomous coding agents unrestricted file and API access is dangerous:
- An agent tasked with backend refactoring might accidentally modify `src/auth/` or leak production API keys.
- An agent might elevate privileges by writing unmanaged raw pointers or bypass pure function contracts.

The **Agent Scoping Engine** (`end scope`) and **AST Security Scanner** (`end security`) create a secure execution perimeter.

---

## 🛡️ Agent Permission Envelopes (`end scope`)

Agent scopes are defined with whitelisted path patterns, allowed actions, and explicit deny rules:

```json
{
  "name": "backend_agent",
  "scope_pattern": "src/**",
  "allow_actions": [
    "read_code",
    "modify_code",
    "run_tests"
  ],
  "deny_patterns": [
    "modify(src/auth/**)",
    "access_secrets",
    "database_write"
  ]
}
```

### Verification & Policy Auditing

When the agent attempts an action:

```bash
# Permitted access:
$ end scope backend_agent src/backend/payment_service.end modify_code
✔ Permitted: Agent `backend_agent` is authorized to perform `modify_code` on `src/backend/payment_service.end`

# Denied access (Blocked by deny rule):
$ end scope backend_agent src/auth/secret_auth.end modify_code
✖ Access Denied: Agent `backend_agent` is strictly forbidden from modifying `src/auth/secret_auth.end` (Deny Rule: `modify(src/auth/**)`)

# Denied access (Out of permitted scope):
$ end scope backend_agent outside/config.toml modify_code
✖ Out of Scope: Target file `outside/config.toml` is outside the permitted scope `src/**`
```

---

## 🔒 AST Static Security Auditing (`end security`)

The AST Security Scanner deterministically identifies vulnerabilities without relying on probabilistic LLM audits:

### 1. CWE-798: Use of Hardcoded Credentials
- Detects plaintext API keys, tokens, and secrets (e.g. `sk_live_...`, `ghp_...`, `AWS_SECRET`).
- Redacts snippets and provides immediate remediation guidelines.

### 2. CWE-285: Improper Authorization / Capability Boundary Violation
- Detects when functions annotated with `@contract("pure=true")` or `@pure` attempt side-effecting disk/network operations.

### 3. CWE-119: Memory Safety & Unmanaged Raw Pointer Escapes
- Detects unchecked raw pointer dereferences (`*raw_ptr`) outside managed `@lease` arenas.

```bash
$ end security temp_sec_secret.end --json
```

```json
{
  "file": "temp_sec_secret.end",
  "is_secure": false,
  "critical_count": 1,
  "high_count": 0,
  "medium_count": 0,
  "vulnerabilities": [
    {
      "cwe_id": "CWE-798",
      "severity": "CRITICAL",
      "title": "Use of Hardcoded Credentials: Stripe Live API Key",
      "line": 3,
      "snippet": "val api_key = \"sk_live_...\"",
      "remediation": "Store API keys in environment variables or a secure key management vault."
    }
  ],
  "total_findings": 1,
  "summary": "✖ Security Guard Blocked: Found 1 security vulnerability (1 critical, 0 high)."
}
```
