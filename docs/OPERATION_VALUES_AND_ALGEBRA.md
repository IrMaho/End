# ⚡ End Language — First-Class Operation Values & Algebraic Primitives

> **First-Class Operations, Algebraic Composition, Event-Native Architecture, Resilient Retries, and Autonomous Refactoring Primitives.**  
> *End treats computational tasks as first-class composable algebraic values equipped with telemetry, contracts, and autonomous evolution capabilities.*

---

## 1. Overview & Paradigm

In traditional languages, functions are static blocks of executable code. In **End**, **Operations (`operation`)** are first-class values that encapsulate:
- **Contract guarantees**: Pre-conditions (`requires`), post-conditions (`guarantee`), and side-effect declarations (`effects`).
- **Algebraic Composability**: Pipelines (`>>`), parallel execution (`&`), alternative fallback (`|`), repetition (`*` / `repeat`), and conditional execution (`when`).
- **Built-in Resilience**: Native retry policies (`retry(attempts, delay)`), hedged latency racing, and memoization (`memoize`).
- **Telemetry & Observability**: Every execution yields structured metrics (duration, error status, retry count, outputs) as first-class return types.
- **Autonomous Refactoring Primitives**: Direct compiler-enforced refactoring directives (`extract`, `inline`, `split`, `merge`, `explain`, `evolve`, `decompose`).

```text
               ┌────────────────────────────────────────┐
               │         First-Class Operation          │
               │   (Contract + Telemetry + Algebra)     │
               └───────────────────┬────────────────────┘
                                   │
       ┌───────────────────────────┼───────────────────────────┐
       ▼                           ▼                           ▼
┌──────────────┐           ┌──────────────┐           ┌──────────────────┐
│  Pipelines   │           │  Resilience  │           │   Refactoring    │
│  (A >> B)    │           │ (retry, memo)│           │ (extract, split) │
└──────────────┘           └──────────────┘           └──────────────────┘
```

---

## 2. Defining First-Class Operations

An operation is declared using the `operation` keyword:

```end
operation validate_cart(items_count: i64) -> i64 {
    requires: items_count > 0;
    guarantee: result == items_count;
    effects: ["read_inventory"];
    version: "1.0.0";

    return items_count;
}

operation charge_payment(amount: i64) -> i64 {
    requires: amount > 0;
    guarantee: result >= amount;
    effects: ["network_pci", "ledger_mutation"];
    version: "1.0.0";

    val rate = 100;
    return amount * rate;
}
```

---

## 3. Operation Algebra & Composition

Operations can be combined dynamically using mathematical and algebraic operators:

| Operator / Keyword | Syntax | Semantics |
| :--- | :--- | :--- |
| **Pipeline / Compose** | `opA >> opB` or `opA + opB` | Chains output of `opA` directly into input of `opB`. |
| **Parallel Execution** | `opA & opB` | Executes both operations concurrently and produces a tuple `(resA, resB)`. |
| **Alternative / Fallback** | `opA \| opB` | Attempts `opA`; if it fails, falls back immediately to `opB`. |
| **Repetition** | `op * 3` or `repeat(op, 3)` | Executes the operation sequentially $N$ times. |
| **Resilient Retry** | `op.retry(3, 100ms)` | Automatically catches failures and retries up to $N$ times with backoff. |
| **Memoization** | `op.memoize()` | Caches results keyed by input arguments in memory. |
| **Conditional** | `op.when(condition)` | Executes the operation only if `condition` evaluates to `true`. |

### Example: Composing an End-to-End Pipeline

```end
fn execute_checkout_pipeline(items: i64) -> i64 {
    // 1. Pipeline composition (validate >> charge)
    val pipeline = validate_cart >> charge_payment;

    // 2. Wrap with resilience & memoization
    val resilient_flow = pipeline.retry(3, 50ms).memoize();

    // 3. Execute with telemetry
    val res = resilient_flow(items);
    return res.output;
}
```

---

## 4. Event Hubs & Native Dispatch

End provides first-class event declarations and message hubs for event-driven reactive architectures:

```end
event OrderPlaced {
    order_id: i64,
    amount: i64,
}

event UserCreated {
    user_id: i64,
    email: string,
}

hub CommerceEvents {
    subscribes: ["OrderPlaced", "UserCreated"];
}

fn process_order(id: i64, cost: i64) {
    // Emit events directly into the runtime event bus
    emit OrderPlaced(id, cost);
    emit UserCreated(42, "user@endlang.org");
}
```

---

## 5. Telemetry & Observability Primitives

Every operation invocation returns a rich result structure containing metadata:

```end
val result = charge_payment(500);

val output_value = result.output;      // The computational return value
val duration_ms  = result.duration_ms; // Execution latency
val is_success   = result.success;     // Boolean status
val retries_used = result.retries;     // Number of retries performed
```

Additionally, reactive streams and operations can be observed and analyzed directly in source code:

```end
observe operation charge_payment {
    metrics: ["latency", "throughput", "error_rate"];
    sample_rate: 1.0;
}

analyze operation charge_payment {
    p99_max: 25ms;
    max_memory: 10MB;
}
```

---

## 6. Autonomous Refactoring & Evolution Directives

To support AI Pair Programming and autonomous refactoring, End provides compiler-level statements to document and enforce structural transformations:

```end
// 1. Extract a sub-operation from a legacy service
extract operation PaymentService from LegacyMonolith {
    methods: ["charge", "refund"];
    target: "services/payment.end";
}

// 2. Inline a micro-operation
inline operation FastValidation;

// 3. Split a monolithic operation into modular units
split operation FullCheckout into ["ValidateCart", "ChargeCard", "SendInvoice"];

// 4. Merge operations into a unified facade
merge { ValidateCart, ChargeCard } as FastCommerce;

// 5. Explain an operation's architectural responsibility
explain operation LegacyOrchestration;

// 6. Evolve an operation with strict architectural constraints
evolve operation LegacyOrchestration {
    preserve: ["pci_compliance", "idempotency"];
    optimize: ["latency", "throughput"];
    allow: ["async_dispatch"];
    reject: ["blocking_calls"];
}

// 7. Decompose a legacy module
decompose LegacyMonolith {
    target_modules: 15;
    optimize: ["cohesion", "coupling"];
    preserve: ["behavior", "api"];
    verify: ["compilation", "tests"];
}
```

---

## 7. Verification & Compilation

All operation values, algebraic compositions, and refactoring directives are validated at compile-time and can be executed natively via the End interpreter (`endc run`) or compiled into highly optimized C (`endc build`).

To test the Operation Values suite:
```bash
cargo test test_50_agent_operation_values_complete_family_matrix
```
