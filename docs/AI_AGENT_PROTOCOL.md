# 🤖 End Language AI-Agent Cognitive Protocol Guide

> **How AI Agents navigate, understand, and safely modify 100M+ line codebases in End.**

---

## 🏛️ The 7 Cognitive Capabilities

### 1. Machine Knowledge Graph (`end graph` / `end query`)
AI agents query caller/callee relationships without reading hundreds of source files:
```bash
end query server.end calculate_physics --callers --json
```
```json
{
  "symbol": "calculate_physics",
  "callers": ["nexus_game_loop", "test_collision_suite"],
  "signature": "fn calculate_physics(x: i32, y: i32, vx: i32, vy: i32) -> i64",
  "memory_arena": "ArenaScoped",
  "purity": "Pure (No I/O)"
}
```

---

### 2. Blast-Radius & Impact Analysis (`end impact`)
Before modifying any symbol in a massive codebase, the agent checks the risk level:
```bash
end impact server.end calculate_physics --json
```
```json
{
  "impact": {
    "target": "calculate_physics",
    "risk_level": "LOW",
    "directly_affected_functions": ["main"],
    "affected_files": [{ "path": "server.end", "lines": [38, 44] }],
    "breaking_changes": false,
    "safe_to_modify": true
  }
}
```

---

### 3. Capability Contracts (`end effects`)
Inspect declared and inferred side effects:
```bash
end effects server.end pure_physics --json
```
Returns: `net: false`, `disk: false`, `io: false`, `memory: "ArenaScoped"`, `is_pure: true`.

---

### 4. Semantic Code Slicing (`end slice`)
Compresses 50,000-line files to 50-line skeletal AST interfaces:
```bash
end slice server.end --interface-only
```

---

### 5. Structured AST Auto-Patches (`end patch`)
Applies node-level AST modifications without text/whitespace breakage:
```bash
end patch server.end --ast-patch patch.json --apply
```

---

### 6. Micro-Isolated Expression Evaluator (`end eval`)
Tests mathematical formulas and algorithms in microseconds without rebuilding:
```bash
end eval "val x = 100 * 31; (x ^ 0x5AA5) % 50" --json
```

---

### 7. Architecture Guardrails (`end arch check`)
Validates project layer invariants against `Architecture.toml`:
```bash
end arch check --json
```
