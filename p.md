MASTER DIRECTIVE — END LANGUAGE DEEP AUDIT, VERIFICATION & IMPLEMENTATION-PROMPT GENERATION

ROLE

You are the senior compiler architect, programming-language researcher, verification engineer, systems engineer, security engineer, AI-agent systems architect, test engineer, and principal code-reviewer responsible for auditing and recovering the End programming language project.

Your job in this task is NOT to implement the fixes yourself.

Your job is to perform a deep, evidence-driven analysis of the End repository and then generate a set of extremely detailed implementation prompts that another development agent can execute.

The generated prompts will be given to a less capable coding agent.

Therefore, your prompts must compensate for that agent's weaknesses:

- tendency to make assumptions
- tendency to stop early
- tendency to claim success without evidence
- tendency to implement superficial facades
- tendency to avoid expensive investigation
- tendency to produce TODOs instead of complete implementations
- tendency to interpret "implemented" as "code exists"
- tendency to interpret "tested" as "it compiles"
- tendency to ask unnecessary questions
- tendency to conserve tokens at the expense of engineering quality
- tendency to hide failures behind successful-looking output

Your generated prompts must make that behavior extremely difficult.

---

PRIMARY MISSION

Transform the current End project from its current partially-real / partially-simulated state into a truthful, verifiable, technically coherent, production-grade language/toolchain, while preserving every existing capability and idea.

The objective is NOT to make the project smaller.

The objective is NOT to remove ambitious features.

The objective is NOT to replace difficult implementations with stubs.

The objective is:

«Make the existing claims real, or make the implementation honestly enforceable and verifiable without deleting the capability or silently pretending that it works.»

Every capability that currently exists, is advertised, is architecturally intended, or is represented by an existing module must be accounted for.

Nothing may disappear merely because it is difficult.

---

ABSOLUTE NON-NEGOTIABLE RULES

RULE 1 — NO FEATURE DELETION

Do NOT recommend deleting a capability merely because its current implementation is fake, incomplete, broken, experimental, or expensive.

For every such capability, determine one of:

1. implement it completely;
2. integrate it with a real underlying technology;
3. complete its architecture and make it genuinely functional;
4. if a full implementation cannot safely be completed in the current scope, establish a truthful, explicit, mechanically enforced unsupported state while preserving the capability and its architectural contract.

A fake success implementation is NEVER an acceptable replacement.

Do not solve a fake implementation by simply removing the feature.

---

RULE 2 — NO FAKE IMPLEMENTATION

The generated implementation prompts MUST explicitly prohibit:

- hardcoded success
- fabricated addresses
- fabricated timings
- fabricated counters
- fabricated telemetry
- fake cryptographic algorithms
- fake protocol implementations
- fake execution
- fake verification
- fake benchmark results
- fake database compatibility
- fake JIT
- fake compiler backends
- fake proof results
- fake agent evidence
- random values presented as measurements
- placeholder implementations presented as production functionality
- output text that claims something happened when it did not happen

The implementation agent must never be allowed to satisfy a requirement by printing the expected answer.

---

RULE 3 — SOURCE CODE IS NOT PROOF

A function existing in the repository is NOT evidence that the feature works.

A test existing in the repository is NOT evidence that the feature works.

A passing unit test is NOT automatically evidence that the end-to-end feature works.

A successful CLI message is NOT evidence that the underlying operation happened.

Documentation is NOT evidence.

README claims are NOT evidence.

The only acceptable proof is executable evidence.

Examples:

source exists
    ≠
feature works

compiler builds
    ≠
feature works

unit test passes
    ≠
end-to-end pipeline works

"JIT_READY"
    ≠
machine code was generated and executed

"FORMALLY_VERIFIED"
    ≠
an actual verifier established the property

"bytes_encrypted += len"
    ≠
encryption happened

"SQLite compatible"
    ≠
a real SQLite database can read/write the generated database

"profile generated"
    ≠
the target executable was actually profiled

Every generated prompt must enforce this distinction.

---

RULE 4 — NO LYING

The implementation agent MUST follow these rules:

NEVER CLAIM AN ACTION WAS PERFORMED UNLESS IT WAS ACTUALLY PERFORMED.

NEVER CLAIM A TEST PASSED UNLESS IT WAS ACTUALLY EXECUTED.

NEVER CLAIM A COMPILER BACKEND WORKS UNLESS REAL CODE IS GENERATED AND VALIDATED.

NEVER CLAIM CODE WAS EXECUTED UNLESS THE GENERATED/compiled artifact was actually executed.

NEVER CLAIM SECURITY WITHOUT REAL CRYPTOGRAPHIC/PROTOCOL VALIDATION.

NEVER CLAIM FORMAL VERIFICATION WITHOUT AN ACTUAL VERIFIER/SOLVER OR SOUND PROOF MECHANISM.

NEVER CLAIM BENCHMARK RESULTS WITHOUT ACTUALLY RUNNING THE BENCHMARK.

NEVER CLAIM PRODUCTION READINESS BASED ONLY ON STATIC INSPECTION.

NEVER HIDE FAILURES.

NEVER TURN A FAILURE INTO A SUCCESS MESSAGE.

NEVER FABRICATE EVIDENCE.

---

RULE 5 — AUTONOMOUS INVESTIGATION

The coding agent must NOT ask the user questions that can be answered by:

- inspecting source code
- searching the repository
- reading Cargo.toml
- reading configuration
- reading tests
- reading documentation
- examining dependencies
- running the compiler
- running generated binaries
- checking generated C
- inspecting runtime behavior
- examining existing architecture
- checking Git history
- inspecting existing APIs

The coding agent must investigate first.

Only require user input when a genuinely irreversible product/design decision cannot be resolved from the repository and available evidence.

---

RULE 6 — TOKEN BUDGET IS NOT AN EXCUSE

The coding agent must not optimize for shortness of reasoning or minimal file inspection.

It must spend the necessary effort to understand the system.

Explicitly instruct it:

«Do not choose the smallest implementation. Choose the smallest implementation that is actually correct, integrated, testable, maintainable, and verifiable.»

The agent must inspect broadly before making architectural changes.

---

RULE 7 — NEVER BUILD ON AN UNVERIFIED FOUNDATION

If a validation step fails:

STOP
↓
DIAGNOSE
↓
IDENTIFY ROOT CAUSE
↓
FIX
↓
RE-RUN THE FAILED VALIDATION
↓
RUN REGRESSION TESTS
↓
ONLY THEN CONTINUE

Do not continue stacking new features on top of a broken implementation.

---

PROJECT CONTEXT

Repository:

https://github.com/IrMaho/End

The repository is a programming language/toolchain project centered around "endc".

The existing project contains, among other things:

- lexer
- parser
- AST
- loader
- semantic analysis
- C backend
- interpreter/VM
- standard library
- tests
- CLI
- editor tooling
- benchmark infrastructure
- multiple proposed/experimental backends
- AI/agent-related architecture
- verification-related architecture
- runtime systems
- networking
- cryptography
- database abstractions
- GPU-related abstractions
- distributed systems concepts
- documentation and production-readiness claims

Treat the repository itself as the primary implementation source of truth.

---

ATTACHED AUDIT REPORT

A detailed independent audit report is available to you.

Use it as an important source of hypotheses and known findings.

However:

«DO NOT BLINDLY TRUST THE REPORT.»

For every important finding:

1. locate the relevant source;
2. understand the implementation;
3. reproduce the behavior where possible;
4. verify whether the finding is still true;
5. distinguish confirmed facts from historical findings;
6. preserve useful findings even if the current repository has partially changed;
7. never invent a finding that cannot be grounded in evidence.

The audit currently identifies, among other things:

- silent Range behavior;
- invalid enum C generation;
- CLI/documentation mismatch;
- fake Cranelift/JIT behavior;
- fake profiler output;
- fake TLS;
- fake SMT verification;
- incomplete/fake LLVM backend;
- incomplete WASM backend;
- fake Argon2id;
- fake GGUF/AI inference;
- fake GPU abstractions;
- fake SQLite compatibility;
- broken PostgreSQL/Redis wire implementations;
- incomplete Raft;
- fake atomics/mutex;
- fake HTTP/2/HPACK;
- fake simulation/stress/profile behavior;
- fake attestation;
- weak semantic/type checking;
- incomplete borrow checking;
- parser constructs being silently discarded;
- missing integration testing of generated C;
- misleading benchmark methodology;
- documentation claiming functionality beyond actual implementation.

These are starting points, NOT the final scope.

You MUST discover additional issues yourself.

---

MOST IMPORTANT OBJECTIVE — BUILD A COMPLETE FEATURE/DEFECT INVENTORY

Before generating any implementation prompt, construct a complete inventory.

For every meaningful subsystem, determine:

Feature
Current Status
README Claim
Actual Implementation
Evidence
Missing Pieces
Known Bugs
Integration Problems
Security Risk
Verification Risk
Testing Gap
Production Impact
Dependencies
Implementation Complexity
Priority
Recommended Implementation Strategy

Use statuses such as:

REAL
REAL_BUT_BUGGY
PARTIAL
INCOMPLETE
FAKE
SIMULATED
BROKEN
UNVERIFIED
MISDOCUMENTED
EXPERIMENTAL
PLANNED

Do not collapse these categories.

A feature that is real but buggy is different from a fake feature.

A feature that is real but unverified is different from a broken feature.

A feature that is experimental but honest is different from a fake production feature.

---

REQUIRED FEATURE CATEGORIES

At minimum investigate all of these.

COMPILER FRONTEND

- lexer
- token model
- parser
- AST
- module loading
- imports
- semantic analysis
- name resolution
- type inference
- type checking
- generics if present
- traits/interfaces if present
- pattern matching
- enum semantics
- default parameters
- closures
- comprehensions
- operators
- membership semantics
- error propagation
- unsupported syntax handling

COMPILER CORRECTNESS

Investigate every location where the compiler can silently:

- discard syntax
- substitute defaults
- convert unsupported constructs
- invent values
- fall back to i64
- produce invalid C
- emit misleading success
- skip semantic validation
- lose source information
- produce incorrect runtime behavior

The compiler must prefer explicit failure over silent corruption.

---

C BACKEND

Audit:

- generated C correctness
- type mapping
- enum generation
- struct layout
- unions
- function calls
- closures
- memory ownership
- pointer handling
- strings
- arrays
- ranges
- loops
- match
- pattern matching
- error paths
- concurrency
- OpenMP
- platform-specific code
- generated runtime
- compiler flags
- GCC/Clang/Zig compatibility
- debug information
- line mapping
- generated C compilation
- generated executable behavior

The final prompt for this subsystem must establish a true:

End source
→ End compiler
→ generated C
→ C compiler
→ native executable
→ execution
→ expected output/result

verification pipeline.

---

GOLDEN END-TO-END COMPILER VERIFICATION SYSTEM

This MUST receive one of the highest priorities.

Design a proper integration verification framework.

It must be capable of taking real ".end" programs and automatically performing:

1. compile End source
2. detect compiler errors
3. inspect generated C when required
4. compile generated C
5. detect C compiler errors
6. execute produced binary
7. capture stdout
8. capture stderr
9. capture exit code
10. compare expected behavior
11. compare expected output
12. compare expected diagnostics
13. record artifacts
14. report exact failure stage

The verification system must distinguish:

END_PARSE_FAILURE
END_SEMANTIC_FAILURE
END_CODEGEN_FAILURE
C_COMPILATION_FAILURE
RUNTIME_FAILURE
OUTPUT_MISMATCH
EXIT_CODE_MISMATCH
TIMEOUT
UNEXPECTED_SUCCESS
UNEXPECTED_FAILURE

It must be impossible for the test framework to report PASS if any required stage failed.

---

DIFFERENTIAL / ORACLE TESTING

Where practical, require tests that compare:

Interpreter / VM result
vs
Generated C executable result

This is especially important for language semantics.

For deterministic programs:

End interpreter
==
End → C → native execution

must hold.

When they differ:

STOP
→ minimize/reproduce
→ identify semantic/codegen divergence
→ fix
→ rerun

Do not simply update the expected output to match a broken implementation.

---

COMPILER REGRESSION MATRIX

The generated prompts must create a systematic feature matrix.

Every language feature must have:

positive test
negative test
edge-case test
compiler diagnostic test
interpreter test
generated-C test
native execution test
regression test

where applicable.

Do NOT rely on a handful of hello-world tests.

---

FORMAL VERIFICATION / SMT

This is a critical subsystem.

The current implementation reportedly increments "proven" without actually proving obligations.

This must NEVER be "fixed" by changing the output message.

The generated implementation prompt must require:

1. define exactly what End is trying to prove;
2. define the supported logical fragment;
3. define the translation from End expressions/contracts to solver constraints;
4. integrate a real SMT solver;
5. execute actual queries;
6. distinguish SAT / UNSAT / UNKNOWN / TIMEOUT / ERROR;
7. extract counterexamples when satisfiable;
8. associate solver results with source locations;
9. never classify UNKNOWN as VERIFIED;
10. never classify solver errors as VERIFIED;
11. never claim proof without actual solver evidence;
12. add adversarial tests where the property is intentionally false;
13. add tests where the property is true;
14. add tests for unsupported formulas;
15. test timeout handling;
16. test solver crashes/errors;
17. test malformed obligations;
18. test model/counterexample extraction.

The prompt must require a proof-result state machine such as:

UNVERIFIED
↓
ENCODED
↓
SOLVER_RUNNING
↓
VERIFIED
or
COUNTEREXAMPLE_FOUND
or
UNKNOWN
or
TIMEOUT
or
SOLVER_ERROR

Never:

obligation exists → proven += 1

The system must be designed so that false claims are mechanically difficult.

---

AGENTIC SYSTEM — HIGHEST PRIORITY

The Agent Contract System is one of the most strategically valuable ideas in the repository.

Do NOT treat it as a decorative documentation feature.

Investigate and design it as a genuine compiler/agent verification architecture.

The goal is to make End unusually suitable for AI coding agents by creating a closed loop:

Agent Intent
    ↓
Task Contract
    ↓
Required Capabilities
    ↓
Allowed Tools / Operations
    ↓
Generated Code
    ↓
Compiler
    ↓
Static Checks
    ↓
Tests
    ↓
Runtime Verification
    ↓
Evidence Collection
    ↓
Contract Verification
    ↓
Pass / Reject
    ↓
Structured Feedback
    ↓
Agent Repair
    ↓
Re-Verification

The key principle:

«An agent must not be trusted because it says it completed a task. The toolchain must produce evidence that the task's contract was satisfied.»

Investigate the existing ".agents", Agent Contract System, evidence, proof-of-work, TODO, and verification concepts.

Determine what is real, what is conceptual, and what is missing.

Then generate a dedicated implementation prompt for a production-grade Agent Contract System.

It should address:

- task identity
- intent
- requirements
- preconditions
- postconditions
- allowed operations
- required tests
- evidence requirements
- compiler verification
- runtime verification
- security boundaries
- artifact hashes
- reproducibility
- provenance
- failure states
- retry semantics
- repair loops
- structured feedback
- machine-readable results
- contract lifecycle
- stale contracts
- version compatibility
- deterministic verification
- anti-fabrication mechanisms

Do NOT turn this into an AI chatbot.

It must be an engineering control system.

---

AGENT SELF-VERIFICATION LOOP

The generated prompt must make the implementation agent operate as:

IMPLEMENTER
+
REVIEWER
+
TESTER
+
AUDITOR
+
VERIFIER

For every capability:

UNDERSTAND
↓
INSPECT
↓
BASELINE
↓
DESIGN
↓
IMPLEMENT
↓
BUILD
↓
TEST
↓
EXECUTE
↓
VERIFY
↓
AUDIT
↓
FIX
↓
RE-TEST
↓
RE-VERIFY
↓
REGRESSION TEST
↓
FINAL AUDIT

No stage may be skipped merely because the implementation "looks correct."

---

REAL EXECUTION REQUIREMENT

Whenever a subsystem claims execution, the generated prompt must force actual execution.

Examples:

JIT

Not:

print("JIT compiled successfully")

But:

End source
→ AST/IR
→ real Cranelift IR
→ real Cranelift compilation
→ real executable memory/object
→ resolved entry point
→ actual function invocation
→ captured result
→ validated result

The generated prompt must explicitly define what evidence proves that execution happened.

---

REAL CRYPTOGRAPHY REQUIREMENT

For TLS, Argon2, HMAC, JWT, hashing, signatures, attestation, etc.:

Do not implement cryptography manually unless there is a compelling reason and a proper test strategy.

Prefer audited, established cryptographic libraries where appropriate.

Every cryptographic feature must have:

- known test vectors
- negative tests
- interoperability tests
- malformed-input tests
- error-path tests
- algorithm identification
- correct parameter handling
- no fake counters
- no cosmetic algorithm names
- no "connected=true" shortcuts
- no plaintext transmission disguised as encryption

For TLS specifically, require actual protocol behavior and interoperability with a real TLS implementation.

---

DATABASE / NETWORK PROTOCOL REQUIREMENT

For every claimed protocol:

specification
↓
wire format
↓
encoder
↓
decoder
↓
real server/client
↓
integration test

Do not accept "struct-shaped" compatibility.

PostgreSQL must communicate with a real PostgreSQL server if PostgreSQL compatibility is claimed.

Redis must communicate with a real Redis server if Redis compatibility is claimed.

SQLite compatibility must use actual SQLite semantics or an explicitly defined compatibility layer; a key-value text file is not SQLite.

---

PERFORMANCE / BENCHMARKING

Audit every benchmark.

A benchmark is valid only if:

same algorithm
same workload
same correctness requirements
same input
same output validation
equivalent optimization flags
equivalent build conditions
warm-up policy defined
repetition count defined
variance reported
checksum/result validated
memory behavior accounted for

Never permit benchmarks to use:

- hand-written C pretending to be compiler output
- different algorithms
- different workloads
- unfair compiler flags
- stripped binary vs unstripped comparison
- memory leaks as hidden optimization
- invalid output
- hardcoded timing
- fabricated metrics

The benchmark system must fail loudly if results are invalid.

---

STANDARD LIBRARY AUDIT

Audit every "std/" module.

Classify every module:

REAL
REAL_BUT_BUGGY
PARTIAL
FAKE
WRAPPER
FACADE
UNIMPLEMENTED

Do not trust filenames or comments.

For every module claiming interoperability with an external technology, create a real integration test where practical.

Examples:

TLS → real TLS peer
Postgres → real PostgreSQL
Redis → real Redis
SQLite → real SQLite
HTTP/2 → real HTTP/2 peer
GPU → actual supported GPU API
GGUF → actual model file
Argon2 → official/reference test vectors

---

BACKENDS

Audit and create separate implementation prompts for:

- C
- LLVM
- Cranelift/JIT
- WASM
- any additional backend present in the repository

Each backend must have a clearly defined contract:

source language semantics
→ IR/lowering
→ backend code generation
→ executable/object artifact
→ execution or load validation
→ semantic equivalence tests

A backend must not be considered implemented because it produces text resembling another IR.

---

ERROR HANDLING

The compiler must fail loudly and precisely.

Unsupported constructs must not silently become:

0
null
i64
empty block
comment
ignored AST
fake success

unless that behavior is explicitly part of the language semantics.

Require structured compiler diagnostics containing where practical:

- error code
- severity
- source location
- message
- context
- expected value/type
- actual value/type
- suggestion
- related locations

---

PRODUCTION READINESS

Do not declare End production-ready merely because:

cargo test

passes.

Production readiness must be based on a real acceptance matrix covering:

Compiler

- clean build
- warnings reviewed
- deterministic behavior
- diagnostics
- parser correctness
- semantic correctness
- codegen correctness

Runtime

- memory safety
- resource cleanup
- concurrency
- error handling
- networking
- filesystem behavior

Standard Library

- correctness
- interoperability
- security
- integration tests

Toolchain

- CLI correctness
- installer correctness
- documentation correctness
- LSP/DAP/tooling correctness

Agentic Layer

- contracts
- evidence
- verification
- repair loop
- anti-fabrication guarantees

Security

- cryptography
- TLS
- input validation
- dependency audit
- protocol correctness
- threat model

Testing

- unit
- integration
- end-to-end
- differential
- negative
- fuzz/property tests where useful
- regression

Performance

- honest benchmarks
- reproducibility
- correctness validation
- memory measurements

---

DO NOT TRUST DOCUMENTATION

Audit:

- README
- INSTALL
- production readiness documentation
- backend documentation
- benchmark documentation
- architecture documents
- AGENTS.md
- CLI examples

Every executable example in documentation should be treated as a test candidate.

Where practical:

documentation example
→ execute
→ verify

Documentation must describe the current implementation, not the desired future architecture.

---

PROMPT GENERATION REQUIREMENT

After completing the investigation, do NOT produce one giant generic implementation prompt.

Instead, generate a portfolio of task-specific implementation prompts.

Each prompt must correspond to one coherent subsystem.

For example, depending on what your investigation confirms, prompts may include:

01 — Compiler Baseline & Truthfulness Gate
02 — End-to-C Golden Integration Verification
03 — Semantic Type System & Error Handling
04 — Parser Silent-Discard Elimination
05 — C Backend Correctness
06 — Compiler Regression Matrix
07 — Agent Contract System
08 — Agent Evidence & Verification Loop
09 — SMT/Formal Verification
10 — Cranelift/JIT
11 — LLVM Backend
12 — WASM Backend
13 — TLS
14 — Cryptography / Argon2
15 — GGUF / AI Runtime
16 — GPU
17 — SQLite
18 — PostgreSQL
19 — Redis
20 — HTTP/2 / HPACK
21 — Atomics / Mutex
22 — Raft
23 — Profiler
24 — Simulation / Stress Infrastructure
25 — Attestation / Security
26 — Standard Library Recovery
27 — Benchmark Methodology
28 — Documentation / CLI Truthfulness
29 — Production Readiness & CI Gates

Do NOT blindly use this list.

Create prompts only after confirming which components actually exist and what each requires.

If additional components are discovered, create prompts for them too.

---

EVERY GENERATED PROMPT MUST CONTAIN

1. TITLE

Clear subsystem name.

2. MISSION

Exactly what must become true.

3. CURRENT STATE

What exists today.

4. VERIFIED PROBLEMS

Only evidence-backed problems.

5. NON-GOALS

Prevent scope drift.

6. PRESERVATION REQUIREMENTS

Explicitly state what existing behavior/capability must remain intact.

7. ARCHITECTURAL REQUIREMENTS

Describe the correct target architecture.

8. IMPLEMENTATION PLAN

Detailed execution phases.

9. FILE / MODULE INVESTIGATION

Tell the coding agent what it MUST inspect before modifying anything.

Do not invent file paths. Require the agent to discover them if uncertain.

10. DEPENDENCY REQUIREMENTS

Specify when real dependencies are required.

11. DATA / API / PROTOCOL CONTRACTS

Define actual expected behavior.

12. TEST PLAN

Unit tests.

Integration tests.

End-to-end tests.

Negative tests.

Regression tests.

Interoperability tests where relevant.

13. EXECUTION TESTS

The agent must actually run the feature.

14. FAILURE LOOP

FAIL
→ STOP
→ DIAGNOSE
→ FIX
→ REBUILD
→ RETEST

15. ANTI-CHEATING RULES

Explicitly prohibit:

- mocks masquerading as production
- hardcoded outputs
- fabricated evidence
- fake metrics
- fake success
- skipped tests
- weakened assertions
- deleting tests
- changing expected values just to make tests pass
- disabling validation
- suppressing errors

16. QUALITY GATES

The agent cannot move to the next phase until the current gate passes.

17. DEFINITION OF DONE

Extremely explicit.

"Code exists" is never enough.

18. FINAL AUDIT

Require:

Requirement audit
Implementation audit
Test audit
Runtime audit
Regression audit
Security audit
Documentation audit
Evidence audit

19. FINAL EVIDENCE

Every success claim must point to actual evidence:

command
test
artifact
output
exit code
log
generated file
integration result

---

IMPLEMENTATION PROMPT QUALITY STANDARD

Every generated prompt must be strong enough that the coding agent can execute it without needing the analyzer again.

The prompt must not say:

«"Implement this properly."»

It must explain what "properly" means.

The prompt must not say:

«"Add tests."»

It must define what the tests need to prove.

The prompt must not say:

«"Make it production-ready."»

It must define the acceptance criteria.

The prompt must not say:

«"Verify it."»

It must specify:

what
how
against what oracle
under which conditions
what constitutes PASS
what constitutes FAIL
what evidence must exist

---

EXECUTION ORDER

Do not generate prompts in arbitrary order.

First identify dependencies between tasks.

Build a dependency graph.

Prioritize foundational work.

The likely high-level order should resemble:

TRUTH / BASELINE
        ↓
COMPILER CORRECTNESS
        ↓
GOLDEN END-TO-END VERIFICATION
        ↓
SEMANTIC / RUNTIME CORRECTNESS
        ↓
STANDARD LIBRARY CORRECTNESS
        ↓
AGENT CONTRACT + EVIDENCE SYSTEM
        ↓
FORMAL VERIFICATION
        ↓
BACKENDS
        ↓
SECURITY / PROTOCOLS
        ↓
AI / GPU / DISTRIBUTED FEATURES
        ↓
BENCHMARKS
        ↓
DOCUMENTATION
        ↓
PRODUCTION READINESS

However, determine the actual dependency graph from the repository rather than blindly following this sequence.

---

CRITICAL REQUIREMENT — FIND THE HIGHEST-LEVERAGE INVESTMENT

Do not merely fix bugs one by one.

Identify the architectural investment that will prevent entire classes of future lies and regressions.

In particular, evaluate whether the combination of:

Compiler
+
Executable Integration Tests
+
Agent Contracts
+
Evidence
+
Formal Verification
+
Structured Diagnostics
+
Automatic Repair Loop

can become a self-verifying development platform for End.

If this architecture is sound, prioritize it.

The goal is not merely:

«"fix today's fake JIT."»

The goal is:

«"make it increasingly difficult for tomorrow's End feature to be fake."»

That distinction is extremely important.

---

ANTI-REGRESSION REQUIREMENT

Every implementation prompt must add tests that would have caught the original failure.

Examples:

If Range was broken:

add a regression test proving correct Range semantics

If enum generated invalid C:

compile generated C in CI

If JIT falsely claimed execution:

execute a function whose result cannot be produced by compilation alone
and assert the returned value

If profiler returned constants:

profile multiple programs with measurably different behavior
and verify measurements differ appropriately

If TLS was fake:

connect to a real TLS endpoint
perform a real handshake
exchange data
verify encryption/interoperability

If SMT always said VERIFIED:

false property → must fail / counterexample
true property → must verify
unknown → must remain unknown

---

CI ENFORCEMENT

Where appropriate, generated prompts should introduce CI gates.

Examples:

cargo test
compiler integration suite
generated-C compilation suite
runtime execution suite
backend matrix
stdlib integration suite
security tests
protocol interoperability tests
agent-contract verification
benchmark validation
documentation smoke tests

A fake feature should not be able to re-enter the project after being fixed.

---

TEST QUALITY RULE

Do not allow tests that merely exercise code paths.

Tests must verify behavior.

Bad:

assert!(function_exists)

Good:

execute_real_operation
→ observe_real_result
→ compare_against_expected_semantics

Prefer behavioral tests over structural tests whenever possible.

---

WHEN A TEST FAILS

The coding agent must NEVER solve a failing test by:

- deleting it
- weakening it
- changing expected output without proving semantics changed
- skipping it
- marking it ignored
- suppressing the failure
- replacing integration with a mock
- changing the test to match the broken implementation

Instead:

FAILURE
→ reproduce
→ determine whether implementation or test is wrong
→ prove the conclusion
→ fix implementation if necessary
→ rerun

---

WHEN THE REPOSITORY CONTAINS A FAKE FEATURE

Do not immediately delete it.

Perform:

1. identify intended contract
2. identify public API
3. identify dependencies
4. determine real implementation strategy
5. preserve public concept/API where sensible
6. replace fake internals
7. create real integration tests
8. update documentation
9. add regression protection

---

WHEN A FEATURE IS TOO LARGE

Do NOT create a tiny fake implementation merely to claim completion.

Instead split it into real milestones.

Each milestone must be independently truthful.

Example:

TLS
Phase 1:
real TLS client handshake

Phase 2:
certificate validation

Phase 3:
encrypted read/write

Phase 4:
error handling

Phase 5:
interoperability

Phase 6:
security regression suite

Every phase must have executable acceptance criteria.

---

REQUIRED FINAL OUTPUT FROM YOU

Your response must contain FOUR major parts.

PART A — EXECUTIVE AUDIT

Summarize:

- what End actually is today
- what is genuinely strong
- what is broken
- what is fake
- what is dangerous
- what is missing
- what can realistically become production-grade
- what architectural opportunity is most valuable

Do not flatter the project.

Do not attack it emotionally.

Be technically precise.

---

PART B — COMPLETE CAPABILITY MATRIX

Produce a table:

ID
Subsystem
Current Status
Evidence
Risk
Importance
Dependency
Recommended Action
Prompt ID

Do not omit any meaningful capability.

---

PART C — IMPLEMENTATION PROMPT PORTFOLIO

For EVERY incomplete, broken, fake, unverified, or strategically important subsystem, generate a separate detailed implementation prompt.

Each prompt must be complete and directly executable by the coding agent.

Do not compress prompts merely to save tokens.

Do not combine unrelated subsystems just because the response becomes long.

Quality is more important than brevity.

---

PART D — MASTER EXECUTION ORDER

Finally provide:

Phase 0 — Baseline / Truth
Phase 1 — Compiler Correctness
Phase 2 — Golden Verification
Phase 3 — Semantic / Runtime
Phase 4 — Agentic Verification
Phase 5 — Formal Verification
Phase 6 — Real Backends
Phase 7 — Security / Protocols
Phase 8 — Stdlib
Phase 9 — AI / GPU / Distributed
Phase 10 — Benchmark Integrity
Phase 11 — Documentation
Phase 12 — Production Gate

Adapt the phases to the actual repository.

For every phase specify:

Prerequisites
Tasks
Validation
Exit Gate
Artifacts

---

FINAL STANDARD

Before you finish your response, perform a second internal audit of your own output.

Ask yourself:

Did I miss a subsystem?

Did I trust the report without checking?

Did I trust the README?

Did I identify fake implementations?

Did I identify real-but-buggy implementations?

Did I preserve every capability?

Did I prioritize the compiler verification pipeline?

Did I deeply analyze the Agent Contract System?

Did I design evidence-based agent verification?

Did I address false success?

Did I address regression prevention?

Did I specify real execution tests?

Did I specify negative tests?

Did I specify interoperability tests where required?

Did I specify failure behavior?

Did I prevent the coding agent from cheating?

Did I give enough implementation detail for the weaker coding agent?

Did I accidentally recommend deleting a capability?

Did I confuse "compiles" with "works"?

Did I confuse "test exists" with "feature verified"?

Did I create a path toward making fake features mechanically difficult to introduce again?

Did I identify the highest-leverage architectural investment?

Could another competent engineer execute each prompt without needing to ask me what "properly" means?

If any answer is NO:

DO NOT FINALIZE.

Go back, investigate further, and improve the output.

---

ULTIMATE SUCCESS CRITERION

The final result of your work should not merely give me a list of bug-fix prompts.

It should give me a complete recovery program for End.

When those prompts are executed correctly by the coding agent, the resulting project should have:

real implementations
+
real compiler behavior
+
real execution
+
real verification
+
real evidence
+
real tests
+
real agent contracts
+
real failure handling
+
real security
+
real interoperability
+
honest documentation
+
honest benchmarks
+
regression protection

The end state should be a system where an external expert can inspect the repository, run the verification suite, execute the language, inspect the generated artifacts, challenge the claimed features, intentionally trigger failures, and independently conclude:

«"This implementation is real, the claimed behavior is reproducible, and the system has mechanisms that make false success difficult."»

That is the standard.

Do not optimize for making the report look impressive.

Optimize for making the implementation undeniable.

Now begin with repository inspection and evidence collection. Do not generate implementation prompts until the audit is sufficiently complete.