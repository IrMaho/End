# 🧠 Dynamic Research Memory (DRM) Specification
## Persistent Investigation State & Multi-Hypothesis Evidence Tracking

---

## 🌟 The Problem: AI Amnesia during Complex Tasks

During complex refactorings, multi-module bugs, or multi-step feature developments, traditional AI agents suffer from amnesia:
- Context windows reset or get compacted, erasing previous hypotheses.
- The agent repeats failed approaches or loses track of verified evidence.
- There is no persistent trace of affected contracts or investigated files.

**Dynamic Research Memory (DRM)** introduces a deterministic file-based memory system stored in `.end/memory/drm_<task_id>.json`.

---

## 📂 DRM Architecture & File Format

DRM checkpoints are stored per task under `.end/memory/`:

```
.end/
└── memory/
    ├── drm_task-183.json
    ├── drm_task-9001.json
    └── drm_task-9002.json
```

### Checkpoint Data Schema

```json
{
  "task_id": "task-183",
  "requirement": "Investigate database race in payment worker",
  "agent_id": "autonomous_agent_01",
  "created_at_ms": 1771675200000,
  "last_updated_ms": 1771675230000,
  "current_phase": "HypothesisVerification",
  "investigated_files": [
    "src/backend/payment_service.end",
    "src/backend/worker.end"
  ],
  "contracts_affected": [
    "PaymentSafe",
    "idempotent=true"
  ],
  "hypotheses": [
    {
      "id": 1,
      "statement": "Missing transaction lock on ledger write allows duplicate processing.",
      "status": "CONFIRMED",
      "evidence": "Observed 2 concurrent requests acquiring the same sequence lock simultaneously.",
      "confidence": 0.95
    },
    {
      "id": 2,
      "statement": "Stripe webhook arrives before DB commit completes.",
      "status": "REJECTED",
      "evidence": "Webhooks are timestamped 400ms after DB commit confirmation.",
      "confidence": 0.10
    }
  ],
  "decision_log": [
    "Phase 1: Isolated payment worker concurrent execution loop.",
    "Phase 2: Confirmed Hypothesis #1; rejected Hypothesis #2.",
    "Phase 3: Synthesizing atomic leasing patch with Arena isolation."
  ],
  "discovered_dependencies": [
    "LedgerRepository",
    "StripeAdapter"
  ],
  "task_completed": false
}
```

---

## 🛠️ DRM CLI Operations

```bash
# 1. Initialize a new DRM task checkpoint
end memory new --task task-183 --req "Investigate database race in payment worker"

# 2. Inspect active DRM state and hypotheses
end memory show --task task-183

# 3. List all checkpointed DRM tasks in project
end memory list

# 4. Resume an existing DRM task in Autonomous Agent Runtime
end agent-run src/backend/payment_service.end "Fix race condition" --task-id task-183
```
