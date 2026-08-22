# 🔒 End Language — Declarative Capability-Based Security Model & Zero Ambient Authority

> **Compile-Time Fine-Grained Sandboxing for Safe AI-Generated Code, Cloud Services, and Zero-Trust Systems.**  
> *How End enforces hardware, OS boundaries, and object-capability patterns at the compiler level without runtime overhead.*

---

## 1. Zero Ambient Authority: The Core Principle

Traditional programming languages allow any function in any module to call global system APIs like `std.fs.write` or `socket.connect`. This ambient authority makes software vulnerable to supply-chain attacks, prompt injection, and lateral privilege escalation.

In End, **there is zero ambient authority**. All access to physical resources (filesystem, sockets, system clock, environment variables, foreign C libraries) requires an unforgeable, first-class **Capability Token**:

```end
// ❌ FAILS TO COMPILE (E0933: Ambient Authority Violation)
pub fn write_app_log(msg: str) void {
    std.fs.write("/var/log/app.log", msg)
}

// ✔ COMPILES CLEANLY (Explicit Capability Required)
pub fn write_app_log(cap: capability<FileWrite>, msg: str) void {
    cap.write("/var/log/app.log", msg)
}
```

---

## 2. The 10 Principles of End Capability Security (Pillar 3)

### 1. First-Class Capability Types (`capability<T>`)
Capabilities are typed values that represent explicit authority to interact with a system resource.

### 2. Capability-Bound Functions (`requires Cap`)
Functions explicitly declare their resource requirements:
```end
@capability("disk=true", "net=false")
pub fn process_receipt(receipt_file: SafePath) void {
    // Filesystem reads permitted; network calls statically blocked
}
```

### 3. Capability Conjunction / Intersection (`CapA & CapB`)
Operations requiring multiple resources specify intersection types:
```end
pub fn sync_remote_cache(cap: capability<FileRead & NetworkSocket>) void {
    // Both capabilities verified at compile time
}
```

### 4. Scoped Delegation (`delegate Cap to worker`)
A caller can delegate a subset of its capabilities to child threads, workers, or asynchronous fibers without exposing full parent authority.

### 5. Temporal Leases & Expiration (`borrow Cap for duration`)
Capabilities can be lent for a finite duration or lexical scope, automatically invalidating after the scope expires.

### 6. Cascading Revocation (`revoke Cap`)
Dynamic capabilities can be revoked at runtime. Any subsequent call attempting to use a revoked capability fails with diagnostic `E0932`.

### 7. Global Capability Flow Graph
The compiler constructs a whole-program capability propagation graph to guarantee that high-privilege tokens cannot leak into untrusted libraries or log sinks.

### 8. Privilege Escalation Prevention (`E0934`)
A low-privilege security context attempting to forge or acquire high-privilege tokens without proof triggers an immediate compiler halt.

### 9. Sandboxed Functions (`sandboxed fn`)
Isolated execution envelopes that guarantee zero side-effects outside an explicitly passed arena memory buffer.

### 10. Security Domains (`domain<Payment>` vs `domain<Analytics>`)
Components are partitioned into security domains. Cross-domain data transit must pass through explicit, audited adapter functions.

---

## 3. Diagnostic Codes for Capability Violations

- `E0931`: Missing Required Capability
- `E0932`: Use of Revoked Capability
- `E0933`: Ambient Authority Violation (No Ambient Authority)
- `E0934`: Privilege Escalation Detected
- `E0935`: Security Domain Boundary Violation
- `E0936`: Dangerous API Quarantine Violation
