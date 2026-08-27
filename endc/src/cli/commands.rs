use clap::Subcommand;
use super::agent_args::*;
use super::ai_args::*;
use super::build_args::*;
use super::db_args::*;
use super::dev_args::*;
use super::gpu_args::*;
use super::package_args::*;
use super::query_args::*;
use super::security_args::*;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Print End language version and toolchain info
    Version,
    /// Real Embedded SQLite Database Engine Subcommands (exec, query, tables)
    Db(DbArgs),
    /// Generate comprehensive OpenAPI 3.1, AI Agent Passport, Struct Memory Layout, and Interactive Swagger Dashboard
    /// Compile and run EndUI reactive declarative applications with AI Agent DevMode Canvas
        /// Path to .end entrypoint file
        /// Output directory (default: ./ui_build)
        /// Enable live AI DevMode Canvas Overlay with visual pin annotations and task board
        /// Start local interactive DevServer
        /// Port for DevServer (default: 3000)
        /// Automatically open UI in default web browser
        /// Compilation target (web, desktop, flutter)
    Ui(UiArgs),
    /// AI Coding Agent bidirectional feedback, task planning, and inspection channel
        /// Action: feedback, list, reply, sync, board
        /// Target feedback / task ID
        /// Reply message or resolution note from AI Agent
        /// Status update: Open, In Progress, Resolved
        /// Output report as machine-readable JSON
    Agent(AgentArgs),
    /// Flutter / Dart Native FFI Bridge and Widget Bindings Generator
        /// Action: bindgen or bridge
        /// Path to .end entrypoint file
        /// Output directory for generated Dart files (default: ./lib)
    Flutter(FlutterArgs),
        /// Path to .end entrypoint file or project directory
        /// Output documentation directory (default: ./docs)
        /// Documentation format: all, html, openapi, passport, markdown
        /// Start built-in local live documentation preview server
        /// Port for local preview server (default: 8080)
        /// Automatically open generated documentation in default web browser
    Doc(DocArgs),
    /// Manage End Language AI coding skills (.agents/skills/end-language)
        /// Subcommand: init (copy skill into current project) or path (print skill path)
    Skill(SkillArgs),
    /// Agent Contract System: verify, inspect, submit, and enforce .agents/contract.toml
    Contract(ContractArgs),
    /// Run an End source file directly (Instant Interpreter VM or JIT Backend)
        /// Path to .end source file
        /// Execution backend: vm, cranelift, llvm
    Run(RunArgs),
    /// Compile an End source file to an ultra-optimized native binary, DLL, or cross-platform target
        /// Path to .end source file
        /// Output binary or library path (e.g. -o mylib.dll or -o app.exe)
        /// Target architecture & OS triple (e.g. x86_64-windows, x86_64-linux, aarch64-macos, aarch64-linux, wasm32-wasi)
        /// Compile as a shared library / dynamic library (.dll, .so, .dylib) with C-ABI header
        /// Compile as a library (alias for --dll / shared library)
        /// Strip all debug symbols for absolute minimum bare-metal binary size
        /// Emit generated C code and header only
        /// Emit LLVM IR (.ll) code directly (Zero C Dependency)
        /// Emit WebAssembly WAT and glue code (.wat)
        /// Dump WebAssembly WAT format to .wat file
        /// Include full DWARF / CodeView debug symbols in LLVM IR
        /// Dump LLVM IR to .ll file
        /// Dump Cranelift CLIF IR to .clif file
        /// Code generation backend (c, llvm, cranelift, wasm)
        /// Perform binary tree-shaking & dead-code elimination (micro-binary optimization)
        /// Enable AddressSanitizer & UndefinedBehaviorSanitizer
    Build(BuildArgs),
    /// Perform fast semantic check and return machine-readable diagnostics
        /// Path to .end source file
        /// Format as JSON for AI Agent
        /// Enforce strict zero memory leaks (fails on escaping pointers and unmanaged allocations)
    Check(CheckArgs),
    /// Start Language Server Protocol (LSP) for VS Code and IDE integrations
    Lsp,
    /// Inspect a specific line for symbols, data-flow and side-effects (Agent Protocol)
        /// Target in format <file.end>:<line_number>
    Inspect(InspectArgs),
    /// Explain a specific line with human & agent-friendly semantic breakdown
        /// Target in format <file.end>:<line_number>
    Explain(ExplainArgs),
    /// Trace timeline and data-flow of a symbol across its entire lifecycle
        /// Path to .end source file
        /// Symbol to trace
    Trace(TraceArgs),
    /// Query side-effects and capabilities for a symbol
        /// Path to .end source file
        /// Symbol name
    Effects(EffectsArgs),
    /// Perform impact analysis before modifying a symbol (Agent Protocol)
        /// Path to .end source file
        /// Symbol to analyze (e.g. function or struct name)
    Impact(ImpactArgs),
    /// Generate full Machine Knowledge Graph for AI Agents (1ms token-efficient JSON)
        /// Path to .end source file
        /// Format as JSON
    Graph(GraphArgs),
    /// Query semantic knowledge, callers, callees, or symbol contracts
        /// Path to .end source file
        /// Symbol name to query
        /// Query functions calling this symbol
        /// Query functions called by this symbol
        /// Format as JSON
    Query(QueryArgs),
    /// Semantic Code Slicing: compress 50,000+ line files into skeletal AST interfaces for AI prompts
        /// Path to .end source file
        /// Only output public interface signatures
        /// Only output struct and enum type definitions
        /// Maximum token budget for AI prompt context
        /// Format as JSON
    Slice(SliceArgs),
    /// Structured AST Auto-Patch: apply node-level AST modifications without text/whitespace breakage
        /// Path to .end source file
        /// Path to AST patch JSON file
        /// Raw JSON string patch payload
        /// Apply changes directly to file
    Patch(PatchArgs),
    /// Micro-isolated expression evaluator: test formulas, expressions, and algorithms in < 50 µs
        /// Raw End code expression or snippet
        /// Format as JSON
    Eval(EvalArgs),
    /// Architecture Guardrails & Rule Enforcement (Architecture.toml validation)
        /// Subcommand action (e.g. check)
        /// Path to Architecture.toml config
        /// Format as JSON
    Arch(ArchArgs),
    /// AI Self-Healing engine: analyze diagnostics, typos, and automatically patch source code
        /// Path to .end source file
        /// Apply the fix patch directly to the file
    Fix(FixArgs),
    /// Run comprehensive unit tests with parallel test runner and @test attribute support
        /// Path to .end source file or test file
        /// Filter test names
        /// Format as JSON for AI Agent
    Test(TestArgs),
    /// Zero-downtime development server with instant hot-reload and state preservation
        /// Entry .end source file
        /// Port for development server
        /// Enable automated nanosecond function profiling and .end_diagnostics/perf_audit log generation
        /// Run single startup pass without blocking server loop
    Dev(DevArgs),
    /// Continuous background health patrol testing runner
        /// Path to .end source file
        /// Health check interval in seconds (default: 5)
        /// Number of patrol iterations (default: 3)
        /// Format as JSON
    Patrol(PatrolArgs),
    /// Watch directory or files for sub-millisecond change detection
        /// Directory or file path to watch
        /// Run single inspection pass without blocking loop
    Watch(WatchArgs),
    /// Interactive "What-If" differential mutation and simulation sandbox
        /// Path to .end source file
        /// Scenario or parameter mutation (e.g. friction=0.05)
        /// Format as JSON
    Simulate(SimulateArgs),
    /// Real High-Scale HTTP Load & Stress Engine (with P50, P90, P99, P99.9 HdrHistogram metrics)
    Stress(StressArgs),
    /// Create a new End language project with end.toml manifest and scaffold
        /// Project directory name
    New(NewArgs),
    /// Initialize end.toml package manifest in current directory
    Init,
    /// Add a dependency to the current project's end.toml
        /// Package name
    Add(AddArgs),
    /// Publish the current package distribution
        /// Validate package without uploading
        /// Store package in local repository (~/.end/local-registry)
    Publish(PublishArgs),
    /// Format End source code canonically
        /// File or directory to format
        /// Check formatting without modifying files (CI mode)
    Fmt(FmtArgs),
    /// Profile program execution (CPU, memory, SVG flamegraph)
        /// Target executable or .end source file
        /// Generate interactive SVG flamegraph
    Profile(ProfileArgs),
    /// Start Debug Adapter Protocol (DAP) server for VS Code / LLDB source debugging
    Dap,
    /// Compiler Explorer mode: inspect End -> HIR -> MIR -> LLVM IR -> ASM
        /// Path to .end source file
    Explore(ExploreArgs),
    /// Install and lock all dependencies specified in end.toml
    Install,
    /// Generate idiomatic FFI bindings for Python, TypeScript, Dart/Flutter, and C#/Unity
        /// Path to .end source file
        /// Output directory for generated bindings
        /// Target languages (comma-separated: python,typescript,dart,csharp)
        /// Name of the native library file (default: file basename)
    Bindgen(BindgenArgs),
    /// Parse C/C++ Header (.h/.hpp) and auto-generate typed End Language bindings
        /// Path to C header file (.h or .hpp)
        /// Output file for generated End bindings (default: <header_name>.end)
    CBindgen(CBindgenArgs),
    /// Automated security fuzz testing engine (AddressSanitizer & UBSan enabled)
        /// Path to .end source file
        /// Number of fuzzing iterations (default: 10,000)
        /// Format as JSON
    Fuzz(FuzzArgs),
    /// Start interactive Read-Eval-Print Loop (REPL)
    Repl,
    /// Package End source files into native Android (.aar) or iOS (.xcframework) archives
        /// Platform target (android or ios)
        /// Path to .end source file
        /// Output package archive path (e.g. --out dist/app.aar or dist/App.xcframework)
    Mobile(MobileArgs),
    /// Intelligent Architectural Scaffolder & Auto-Template Generator (Clean Architecture)
        /// Entity/Feature type (e.g. feature, entity, service)
        /// Name of the feature/module (e.g. auth, billing, payment)
        /// Architecture preset (clean_hexagonal, game_ecs, event_driven_microservice, layered_api)
    Gen(GenArgs),
    /// Verify cognitive alignment between implementation and @intent formal contracts
        /// Path to .end source file
        /// Format as JSON
    IntentVerify(IntentVerifyArgs),
    /// Lint project files against end.config.toml rules (max lines, naming, comments, complexity)
        /// Path to .end file or project directory
        /// Format as JSON for AI Agent integration
    Lint(LintArgs),
    /// Generate a default end.config.toml configuration file
    ConfigInit,
    /// Mine Project DNA, architectural signals, naming conventions, and generate AI prompt guidelines
        /// Path to .end file or project directory
        /// Generate system prompt for AI Pair Programmers
        /// Format as JSON
    Dna(DnaArgs),
    /// Task-Intent driven Smart Context Extraction (DEC_v2 High-Density Token Compressor)
        /// Path to .end source file
        /// Task intent / prompt (e.g. "Add discount calculation to checkout")
        /// Token budget (default: 500 tokens)
        /// Format as JSON
    Context(ContextArgs),
    /// Pre-touch impact and blast radius analysis before touching code
        /// Path to .end source file
        /// Target symbol to inspect
        /// Format as JSON
    Precheck(PrecheckArgs),
    /// Formal Compiler Skill & Contract Verification (PaymentSafe, Idempotent, AuditLogged, etc.)
        /// Path to .end source file
        /// Format as JSON
    Verify(VerifyArgs),
    /// Security-by-Construction Scanner & Verified Build Gate
        /// Path to .end source file
        /// Security Level: standard, strict, paranoid, critical, absolute
        /// Format as JSON
    Security(SecurityArgs),
    /// Cryptographic Verified Build Attestation Generator
        /// Path to .end source file
        /// Format as JSON
    Attest(AttestArgs),
    /// Dynamic Research Memory (DRM) multi-step engineering lifecycle checkpointing
        /// Action: list, show, new
        /// Target Task ID (e.g. task-183)
        /// Requirement description
        /// Format as JSON
    Memory(MemoryArgs),
    /// Permissioned Agent Scoping & Capability Boundary Guard
        /// Target Agent Name (e.g. backend_refactor)
        /// Target file to modify
        /// Requested action (e.g. modify_code, read_code)
        /// Format as JSON
    Scope(ScopeArgs),
    /// Semantic Git: compute semantic diffs and generate Cryptographically Verified Commits
        /// Action: diff or commit
        /// Path to .end source file
        /// Task ID
        /// Commit message / requirement
        /// Format as JSON
    SemanticGit(SemanticGitArgs),
    /// Export official End Semantic IR (AST, TypeGraph, SymbolGraph, ContractGraph) for DeepSift
        /// Path to .end source file
        /// Format as JSON
    SemanticIr(SemanticIrArgs),
    /// Run Autonomous Software Engineering Agent on a task intent
        /// Path to .end entrypoint file
        /// Task intent string
        /// Task ID (default: auto-generated)
        /// Format as JSON
    AgentRun(AgentRunArgs),
    /// Deeply evaluate evolvable modules, generate extensibility DNA report, and compute impact
        /// Target module name or struct to evaluate (e.g. all, User, Auth)
        /// Path to .end source file
        /// Format as machine-readable JSON
    Evolve(EvolveArgs),
    /// Manage API stability, snapshots, SemVer diffs, and migration paths
        /// Action: snapshot, diff, verify, migrate
        /// Primary .end source file or v1 snapshot
        /// Secondary .end source file or v2 snapshot for diffing
        /// Format as JSON
    Api(ApiArgs),
    /// Feature-Oriented Paradigm Lifecycle Engine (create, list, evolve, impact)
        /// Subcommand: create, list, evolve, impact
        /// Feature name, target symbol, or target path
        /// Architecture template or preset
        /// Format output as JSON
    Feature(FeatureArgs),
    /// Local AI Runtime & GGUF Quantized Inference Engine
    Ai(AiArgs),
    /// Real GPU Compute Engine & Hardware Acceleration
    Gpu(GpuArgs),
}
