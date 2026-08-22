# Security-by-Construction & Verified Build Specification in End Language

## 1. Executive Philosophy & The "Verified Build" Paradigm

The End Programming Language shifts software security from an afterthought (linters, libraries, runtime patches, and human reviews) to a **foundational language invariant**. 

In End, security is **Security-by-Construction**. If a program cannot formally prove that it satisfies its declared security contracts, capability bounds, and information-flow constraints, the compiler rejects the code and **refuses to produce a binary**:

$$\text{Code} + \text{Capabilities} + \text{Information Flow Proofs} \xrightarrow{\text{End Compiler}} \begin{cases} \text{Verified Binary} + \text{Cryptographic Attestation Manifest} & (\text{Proved Safe}) \\ \text{Compilation Failure (Zero Binary Generated)} & (\text{Unverified}) \end{cases}$$

---

## 2. Complete Taxonomy of 50 Security Primitives Across 5 Pillars

### Pillar 1: Type-Level Security Primitives (Features 1–10)

| No. | Primitive | Syntax Example | Mathematical & Semantic Guarantee |
| :--- | :--- | :--- | :--- |
| 1 | `tainted<T>` | `val input: tainted<str>` | Marks data originating from untrusted boundaries (network, disk, env). Cannot reach sinks without sanitizer transform. |
| 2 | `sanitized<T>` | `val safe: sanitized<str>` | Produced exclusively through approved sanitizer functions (e.g. `SqlValue.param()`). |
| 3 | `secret<T>` | `val key: secret<str>` | Zeroized on drop; rejected from `println`, string interpolation, logs (`E0926`), and serialization (`E0927`). |
| 4 | `private<T>` | `val data: private<UserData>` | Restricts visibility strictly to the declaring security boundary; prohibited from cross-domain export without declassification. |
| 5 | `trusted<T>` | `val admin: trusted<User>` | Guaranteed to originate from authenticated and authorized system boundaries. |
| 6 | `untrusted<T>` | `val req: untrusted<HttpRequest>` | Forces defensive validation before any state mutation or capability invocation. |
| 7 | `capability<T>` | `borrow capability<FileRead>` | Unforgeable first-class token required for calling effectful/system APIs. |
| 8 | `authority<T>` | `authority<DatabaseRead>` | Encapsulates specific authorization level (`Read`, `Write`, `Admin`) preventing ambient privilege abuse. |
| 9 | `sealed<T>` | `val uid: sealed<i64>` | Immutable runtime value that cannot be modified after initial validation construction. |
| 10 | `verified<T>` | `val p: verified<Payment>` | Structurally guarantees that all formal invariants (`prove amount > 0`) have succeeded. |

---

### Pillar 2: Vulnerability Class Elimination (Features 11–20)

| No. | Feature | Compiler Diagnostic | Semantic Rule & Prevention Mechanism |
| :--- | :--- | :--- | :--- |
| 11 | `SqlValue` Parameterization | `E0921` (CWE-89) | SQL sinks strictly require `SqlValue` or parameterized strings. Raw string concatenation triggers fatal compile rejection. |
| 12 | `HtmlEscaped<T>` XSS Safety | `E0922` (CWE-79) | Template rendering engines reject unescaped string injection into HTML sinks. |
| 13 | `ShellArg` Injection Safety | `E0923` (CWE-78) | OS process execution forbids raw shell strings; arguments must be typed `ShellArg`. |
| 14 | `SafePath` Traversal Safety | `E0924` (CWE-22) | Filesystem operations require canonicalized `SafePath` values, preventing `../` path traversal attacks. |
| 15 | `TrustedUrl` Policy Matching | `E0925` (CWE-918) | HTTP client sinks validate destination URLs against declared whitelist policies to prevent SSRF. |
| 16 | Typed Cryptographic Keys | Type Soundness | `Key<256>`, `Nonce<96>`, `Ciphertext<AesGcm256>` enforce correct key sizes and algorithms at compile time. |
| 17 | Linear Consumable `Nonce` | `E0930` (CWE-323) | `Nonce<N>` is an affine/linear type consumed upon use; re-use triggers compile-time violation. |
| 18 | `constant_time` Functions | `E0929` (CWE-208) | Functions marked `@constant_time` prohibit secret-dependent branching (`if`, `match`), preventing timing side-channels. |
| 19 | Memory `zeroize` on Drop | Memory Model | Data marked `secret<T>` automatically generates zero-fill cleanup in LLVM/C IR upon scope exit. |
| 20 | Non-Serializable Secrets | `E0927` (CWE-359) | Standard JSON/XML serialization reflection traits are explicitly disabled for `secret<T>`. |

---

### Pillar 3: Capability Security & Zero Ambient Authority (Features 21–30)

```
        ┌─────────────────────────────────────────────────────────────┐
        │                 NO AMBIENT AUTHORITY                        │
        │  No global filesystem, network, or OS access without tokens │
        └──────────────────────────────┬──────────────────────────────┘
                                       │
                ┌──────────────────────┴──────────────────────┐
                │                                             │
      ┌─────────▼─────────┐                         ┌─────────▼─────────┐
      │  Capability Grant │                         │ Capability Domain │
      │  requires CapA    │                         │ domain<Payment>   │
      └─────────┬─────────┘                         └─────────┬─────────┘
                │                                             │
      ┌─────────▼─────────┐                         ┌─────────▼─────────┐
      │ Delegation / Loan │                         │ Revocation Check  │
      │ delegate Cap to W │                         │ revoke Cap -> E0932│
      └───────────────────┘                         └───────────────────┘
```

1. **Capability-Bound Functions**: Functions declare prerequisites via `@capability(...)` or `requires Capability`.
2. **Capability Intersection**: Multi-capability requirements `requires FileRead & Network`.
3. **Delegation**: Explicit token delegation `delegate FileRead to worker`.
4. **Scoped Expiration**: Token borrowing with lifetime bounds.
5. **Revocation**: Instant revocation of dynamic capabilities; post-revocation use fails with `E0932`.
6. **Zero Ambient Authority**: Direct access to OS/FS without capability token triggers `E0933`.
7. **Capability Flow Graph**: Whole-program callgraph tracking of privilege propagation.
8. **Privilege Escalation Detection**: Static analysis prevents low-privilege scopes from forging high-privilege context (`E0934`).
9. **Sandboxed Functions**: Isolated computation blocks with restricted instruction sets.
10. **Security Domains**: Logical isolation barriers between components (e.g. `domain<Payment>` vs `domain<Analytics>`).

---

### Pillar 4: Compiler Security Gate & Contract Verification (Features 31–40)

- **Security Contracts**: Declarative contract specifications (`contract PaymentSecurity { accepts: [...], guarantees: [...] }`).
- **In-Source Threat Models**: `@threat_model("attacker=remote", "trust=zero")`.
- **Machine-Checked Security Budgets**: Maximum allowable taint depth, capability grants, and unverified assumptions.
- **Security Levels**:
  - `standard`: Baseline memory safety and taint warnings.
  - `strict`: Zero warnings allowed; all taint flows must be parameterized.
  - `paranoid`: Strict capability isolation + zero ambient authority.
  - `critical`: Full SMT formal proofs required for state transitions.
  - `absolute`: Verified Build mode; binary prohibited without complete formal attestation.
- **Verified Dependency Graph**: Every package declares a `trust` level (`verified` vs `untrusted`). Untrusted dependencies are blocked in strict modes (`E0937`).
- **Cryptographic Attestation**: The compiler emits a SHA-256 signed `VerifiedBuildManifest` containing proof hashes, capability graphs, and build timestamps.

---

### Pillar 5: Information Flow, Adversarial Compilation & Verified Build (Features 41–50)

- **Security Borrowing**: Capabilities can be lent temporarily and are automatically returned at scope exit.
- **Explicit Declassification**: Secrets can only be transformed into public representations via explicit proof (`declassify secret_val with proof<MaskProof>`), failing with `E0928` if unverified.
- **Security State Machines**: Explicit state progression (`Untrusted -> Authenticated -> Authorized -> Active`).
- **Dangerous API Quarantine**: Low-level assembly or raw memory operations must be enclosed inside `unsafe { reason: "...", proof: ... }` (`E0936`).
- **Adversarial Compilation Engine**: The compiler runs an internal exploit generator (synthetic SQLi, XSS, taint exfiltration) against the AST during compilation. If an attack succeeds, the build fails with `E0938`.
- **Multi-Agent Consensus Gate**: Critical builds require multi-agent agreement (3/3 votes) before binary generation.
- **Verified Build Pipeline**: If any violation exists, binary generation is aborted and diagnostic remediations are presented.

---

## 3. CLI Reference & Verification Commands

```bash
# 1. Audit source file with Security-by-Construction engine
endc security src/main.end --level absolute

# 2. Output machine-readable JSON security report
endc security src/main.end --level paranoid --json

# 3. Generate cryptographic Verified Build Manifest
endc attest src/main.end

# 4. Run compiler test suite
cargo test
```
