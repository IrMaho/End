use clap::Args;
use std::path::PathBuf;

    /// Inspect a specific line for symbols, data-flow and side-effects (Agent Protocol)
        /// Target in format <file.end>:<line_number>
#[derive(Args, Debug, Clone)]
pub struct InspectArgs {
        pub target: String,
}

    /// Explain a specific line with human & agent-friendly semantic breakdown
        /// Target in format <file.end>:<line_number>
#[derive(Args, Debug, Clone)]
pub struct ExplainArgs {
        pub target: String,
}

    /// Trace timeline and data-flow of a symbol across its entire lifecycle
        /// Path to .end source file
        /// Symbol to trace
#[derive(Args, Debug, Clone)]
pub struct TraceArgs {
        pub file: PathBuf,
        pub symbol: String,
}

    /// Query side-effects and capabilities for a symbol
        /// Path to .end source file
        /// Symbol name
#[derive(Args, Debug, Clone)]
pub struct EffectsArgs {
        pub file: PathBuf,
        pub symbol: String,
}

    /// Perform impact analysis before modifying a symbol (Agent Protocol)
        /// Path to .end source file
        /// Symbol to analyze (e.g. function or struct name)
#[derive(Args, Debug, Clone)]
pub struct ImpactArgs {
        pub file: PathBuf,
        pub symbol: String,
}

    /// Generate full Machine Knowledge Graph for AI Agents (1ms token-efficient JSON)
        /// Path to .end source file
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct GraphArgs {
        pub file: PathBuf,
        #[arg(long, default_value_t = true)]
        pub json: bool,
}

    /// Query semantic knowledge, callers, callees, or symbol contracts
        /// Path to .end source file
        /// Symbol name to query
        /// Query functions calling this symbol
        /// Query functions called by this symbol
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct QueryArgs {
        pub file: PathBuf,
        pub symbol: String,
        #[arg(long, default_value_t = false)]
        pub callers: bool,
        #[arg(long, default_value_t = false)]
        pub callees: bool,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Semantic Code Slicing: compress 50,000+ line files into skeletal AST interfaces for AI prompts
        /// Path to .end source file
        /// Only output public interface signatures
        /// Only output struct and enum type definitions
        /// Maximum token budget for AI prompt context
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct SliceArgs {
        pub file: PathBuf,
        #[arg(long, default_value_t = true)]
        pub interface_only: bool,
        #[arg(long, default_value_t = false)]
        pub types_only: bool,
        #[arg(short, long)]
        pub budget: Option<usize>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Structured AST Auto-Patch: apply node-level AST modifications without text/whitespace breakage
        /// Path to .end source file
        /// Path to AST patch JSON file
        /// Raw JSON string patch payload
        /// Apply changes directly to file
#[derive(Args, Debug, Clone)]
pub struct PatchArgs {
        pub file: PathBuf,
        #[arg(long)]
        pub ast_patch: Option<PathBuf>,
        #[arg(long)]
        pub json_input: Option<String>,
        #[arg(long, default_value_t = false)]
        pub apply: bool,
}

    /// Micro-isolated expression evaluator: test formulas, expressions, and algorithms in < 50 µs
        /// Raw End code expression or snippet
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct EvalArgs {
        pub expression: String,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Architecture Guardrails & Rule Enforcement (Architecture.toml validation)
        /// Subcommand action (e.g. check)
        /// Path to Architecture.toml config
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct ArchArgs {
        #[arg(default_value = "check")]
        pub action: String,
        #[arg(short, long)]
        pub config: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// AI Self-Healing engine: analyze diagnostics, typos, and automatically patch source code
        /// Path to .end source file
        /// Apply the fix patch directly to the file
#[derive(Args, Debug, Clone)]
pub struct FixArgs {
        pub file: PathBuf,
        #[arg(long, default_value_t = false)]
        pub apply: bool,
}

    /// Compiler Explorer mode: inspect End -> HIR -> MIR -> LLVM IR -> ASM
        /// Path to .end source file
#[derive(Args, Debug, Clone)]
pub struct ExploreArgs {
        pub file: PathBuf,
}

    /// Lint project files against end.config.toml rules (max lines, naming, comments, complexity)
        /// Path to .end file or project directory
        /// Format as JSON for AI Agent integration
#[derive(Args, Debug, Clone)]
pub struct LintArgs {
        #[arg(default_value = ".")]
        pub path: PathBuf,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

