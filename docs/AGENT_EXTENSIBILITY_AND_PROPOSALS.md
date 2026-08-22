# AI Agent Extensibility, Proposals & Proof Gates

Autonomous AI coding agents operate under mathematical safety constraints in the End Programming Language.

---

## 🤖 1. Agent Extension Contracts (`agent_extension`)

Declare permission boundaries, maximum token budgets, and allowed mutation capabilities for agents:

```end
agent_extension MigrationAgent {
    allowed_actions: ["refactor", "migrate_v1_to_v2"],
    max_tokens: 8000,
    sandbox: true,
    change_budget: {
        max_files: 5,
        max_modules: 2,
    }
}
```

---

## 📝 2. Formal Change Proposals (`proposal`)

Agents do not apply unverified direct writes. Instead, agents construct a formal `proposal`:

```end
proposal UpgradeOAuthProvider {
    target: "AuthModule",
    intent: "Migrate legacy OAuth1 to OpenID Connect OAuth2",
    files_modified: ["auth.end", "session.end"],
    rationale: "Compliance with RFC 6749",
}
```

---

## 🚪 3. Proof Gates (`proof_gate`)

A proposal cannot be committed or merged without clearing all proof gates:

```end
proof_gate ProductionMergeGate {
    require tests_passed >= 100%;
    require blast_radius_risk <= "Medium";
    require security_scan == "clean";
    require backward_compatibility == true;
}
```

---

## 🔒 4. Atomic Multi-File Agent Transactions (`agent_transaction`)

When refactoring spanning multiple files, the transaction engine guarantees all-or-nothing rollback:

```end
agent_transaction Tx402 {
    begin;
    // Step 1: Update struct User
    // Step 2: Update database schema
    // Step 3: Run compiler checks
    commit;
}
```

If any compilation or proof gate error occurs between `begin` and `commit`, the End runtime executes an instant transactional rollback, restoring the exact previous state with zero leftover corruption.
