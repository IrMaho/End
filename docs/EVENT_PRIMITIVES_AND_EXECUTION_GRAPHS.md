# ⚡ End Language — Event Primitives & First-Class Execution Graphs

## 👑 1. Core Philosophy: Compute When Something Happens

In the End programming language, events are **not mere callbacks** or secondary runtime notifications. 

> **"Events are not callbacks. Events are the execution graph."**

Events in End represent **first-class execution graphs and compute triggers**. They unify reactive programming, distributed stream processing, actor channels, backpressure flow control, and self-healing topologies into language-level constructs.

---

## 🏛️ 2. Event Declarations & Modalities

### 2.1 Untyped & Typed Payloads
Events can be declared as simple pulse triggers or with rich typed schemas:

```end
// Simple trigger pulse
event SystemInit;

// Typed payload schema
event UserLogin(user_id: str, ip: str, timestamp: i64);
```

### 2.2 Generic Event Streams & Channel Modalities
End natively supports duplex, half-duplex, and directional event stream channels:

```end
// Generic event stream
event SensorStream<T>;

// Full-Duplex Channel (bidirectional)
event ClientChannel <-> ServerChannel;

// Half-Duplex Channel (alternating)
event MasterSync <~> ReplicaSync;

// Single-Directional Pipeline
event InboundStream -> OutboundStream;
```

### 2.3 Event Subtyping & Hardware Attributes
Events support subtyping relationships and low-level hardware memory attributes:

```end
event CriticalSecurityAlert : SecurityEvent with ring_buffer(4096), wal_persisted;
```

---

## 🎯 3. Event Triggers & Control Flow

End provides dedicated declarative trigger constructs:

### 3.1 Guards (`when`), Filters (`where`), and Projections (`=>`)

```end
on UserLogin when active_users < 10000 where ip != "127.0.0.1" => user_id {
    audit_log(user_id);
}
```

### 3.2 Temporal Triggers (`once`, `every`, `after`, `before`)

```end
// Fires exactly once upon event arrival
once SystemInit {
    bootstrap_subsystems();
}

// Fixed-interval recurring schedule
every 100ms {
    collect_telemetry();
}

// Scheduled delay execution
after 500ms {
    flush_intermediate_buffers();
}

// Interceptor executing before downstream node
before DatabaseFlush {
    verify_wal_checksums();
}
```

---

## 🔄 4. Reactive State & Auto-Tracking

End unifies reactive state tracking with event generation:

```end
// Reactive state variables
state user_count = 0;
state unit_price = 100;
state order_qty = 5;

// Reactive derived computation
derive total_revenue from unit_price, order_qty => unit_price * order_qty;

// State mutation event triggers
on user_count.changed {
    update_dashboard_metrics(user_count);
}
```

---

## 🌊 5. Stream Rate Control & Window Operators

High-throughput event streams are managed with zero-allocation streaming primitives:

```end
// Debounce: Emit only after quiet period
debounce 200ms InboundNetworkPacket {
    process_batch();
}

// Throttle: Limit emission frequency
throttle 16ms FrameRenderTick {
    render_screen();
}

// Sample: Sub-sample percentage
sample 10 InboundNetworkPacket {
    log_sample();
}

// Windowing: Sliding and Tumbling Windows
window sliding(100, 10) InboundNetworkPacket {
    compute_sliding_aggregate();
}

window tumbling(50) InboundNetworkPacket {
    persist_batch_to_storage();
}
```

---

## 🛡️ 6. Reliability, Backpressure & Event Transactions

End guarantees transactional consistency and bounded memory execution:

```end
// Event Transactions with Automatic Rollback
event_transaction {
    reserve_inventory(order_id);
    charge_payment(order_id);
    ack InboundOrderEvent;
} rollback {
    release_inventory_reservation(order_id);
    refund_payment_gateway(order_id);
}
```

---

## 🌐 7. Topologies & First-Class Graph Execution

Topologies define dataflow graphs directly in the language:

```end
topology DataIngestionGraph {
    RawIngest -> SchemaValidator -> DataEnricher -> StorageSink;
}

// Graph Protection & Circuit Breakers
circuit_breaker DataIngestionGraph failure_threshold: 5, reset_timeout: 3000ms;
retry_policy DataIngestionGraph max_attempts: 3, backoff: exponential(100ms);
dead_letter_queue DataIngestionGraph target: "dlq_storage", max_size: 10000;

// Graph Composition & Fusion
fuse ChannelA, ChannelB as UnifiedDataPipeline;
```

---

## 🔬 8. Compiler Pipeline Verification

The Event Primitives & Execution Graph system is verified across the complete compiler stack:
1. **Lexer:** Multi-character tokens (`<->`, `<~>`), keywords (`event`, `on`, `once`, `every`, `state`, `derive`, `topology`, `debounce`, `throttle`, etc.).
2. **Parser:** Lossless recursive descent parser with sub-500-line modular design (`reactive_events.rs`, `event_streams_and_topologies.rs`, `data.rs`).
3. **Semantic Analyzer:** Symbol registration, type verification, and graph cycle checks.
4. **Interpreter / VM:** Direct execution with scope tracking, event dispatch, and rollback transactions.
5. **C Backend Codegen:** Native C output with high-performance event loop bindings (`end_emit_event`, `end_observe_operation`).
6. **Integration Test Suite:** 10-phase verification matrix (`event_graph_tests.rs`) with 100% green pass rate across 547 compiler tests.
