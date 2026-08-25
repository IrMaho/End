use clap::Args;
use std::path::PathBuf;

    /// Generate comprehensive OpenAPI 3.1, AI Agent Passport, Struct Memory Layout, and Interactive Swagger Dashboard
    /// Compile and run EndUI reactive declarative applications with AI Agent DevMode Canvas
        /// Path to .end entrypoint file
        /// Output directory (default: ./ui_build)
        /// Enable live AI DevMode Canvas Overlay with visual pin annotations and task board
        /// Start local interactive DevServer
        /// Port for DevServer (default: 3000)
        /// Automatically open UI in default web browser
        /// Compilation target (web, desktop, flutter)
#[derive(Args, Debug, Clone)]
pub struct UiArgs {
        pub file: PathBuf,
        #[arg(short, long)]
        pub output: Option<PathBuf>,
        #[arg(long, default_value_t = true)]
        pub dev: bool,
        #[arg(long, default_value_t = false)]
        pub serve: bool,
        #[arg(long, default_value_t = 3000)]
        pub port: u16,
        #[arg(long, default_value_t = false)]
        pub open: bool,
        #[arg(long, default_value = "web")]
        pub target: String,
}

    /// AI Coding Agent bidirectional feedback, task planning, and inspection channel
        /// Action: feedback, list, reply, sync, board
        /// Target feedback / task ID
        /// Reply message or resolution note from AI Agent
        /// Status update: Open, In Progress, Resolved
        /// Output report as machine-readable JSON
#[derive(Args, Debug, Clone)]
pub struct AgentArgs {
        #[arg(default_value = "list")]
        pub action: String,
        #[arg(long)]
        pub id: Option<String>,
        #[arg(long)]
        pub message: Option<String>,
        #[arg(long)]
        pub status: Option<String>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Manage End Language AI coding skills (.agents/skills/end-language)
        /// Subcommand: init (copy skill into current project) or path (print skill path)
#[derive(Args, Debug, Clone)]
pub struct SkillArgs {
        #[arg(default_value = "init")]
        pub action: String,
}

    /// Agent Contract System: verify, inspect, submit, and enforce .agents/contract.toml
#[derive(Args, Debug, Clone)]
pub struct ContractArgs {
    #[arg(default_value = "verify")]
    pub action: String,
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

    /// Mine Project DNA, architectural signals, naming conventions, and generate AI prompt guidelines
        /// Path to .end file or project directory
        /// Generate system prompt for AI Pair Programmers
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct DnaArgs {
        #[arg(default_value = ".")]
        pub path: PathBuf,
        #[arg(long, default_value_t = false)]
        pub prompt: bool,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Task-Intent driven Smart Context Extraction (DEC_v2 High-Density Token Compressor)
        /// Path to .end source file
        /// Task intent / prompt (e.g. "Add discount calculation to checkout")
        /// Token budget (default: 500 tokens)
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct ContextArgs {
        pub file: PathBuf,
        pub intent: String,
        #[arg(short, long, default_value_t = 500)]
        pub budget: usize,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Pre-touch impact and blast radius analysis before touching code
        /// Path to .end source file
        /// Target symbol to inspect
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct PrecheckArgs {
        pub file: PathBuf,
        pub symbol: String,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Dynamic Research Memory (DRM) multi-step engineering lifecycle checkpointing
        /// Action: list, show, new
        /// Target Task ID (e.g. task-183)
        /// Requirement description
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct MemoryArgs {
        #[arg(default_value = "list")]
        pub action: String,
        #[arg(long)]
        pub task: Option<String>,
        #[arg(long)]
        pub req: Option<String>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Permissioned Agent Scoping & Capability Boundary Guard
        /// Target Agent Name (e.g. backend_refactor)
        /// Target file to modify
        /// Requested action (e.g. modify_code, read_code)
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct ScopeArgs {
        pub agent: String,
        pub file: PathBuf,
        #[arg(default_value = "modify_code")]
        pub action_type: String,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Semantic Git: compute semantic diffs and generate Cryptographically Verified Commits
        /// Action: diff or commit
        /// Path to .end source file
        /// Task ID
        /// Commit message / requirement
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct SemanticGitArgs {
        #[arg(default_value = "diff")]
        pub action: String,
        pub file: PathBuf,
        #[arg(long)]
        pub task: Option<String>,
        #[arg(short, long)]
        pub message: Option<String>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Export official End Semantic IR (AST, TypeGraph, SymbolGraph, ContractGraph) for DeepSift
        /// Path to .end source file
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct SemanticIrArgs {
        pub file: PathBuf,
        #[arg(long, default_value_t = true)]
        pub json: bool,
}

    /// Run Autonomous Software Engineering Agent on a task intent
        /// Path to .end entrypoint file
        /// Task intent string
        /// Task ID (default: auto-generated)
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct AgentRunArgs {
        pub file: PathBuf,
        pub intent: String,
        #[arg(long)]
        pub task_id: Option<String>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Deeply evaluate evolvable modules, generate extensibility DNA report, and compute impact
        /// Target module name or struct to evaluate (e.g. all, User, Auth)
        /// Path to .end source file
        /// Format as machine-readable JSON
#[derive(Args, Debug, Clone)]
pub struct EvolveArgs {
        #[arg(default_value = "all")]
        pub target: String,
        #[arg(short, long)]
        pub file: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

