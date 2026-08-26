pub mod agent_args;
pub mod ai_args;
pub mod build_args;
pub mod commands;
pub mod db_args;
pub mod dev_args;
pub mod gpu_args;
pub mod package_args;
pub mod query_args;
pub mod security_args;

use clap::Parser;
pub use commands::Commands;

#[derive(Parser, Debug)]
#[command(
    name = "endc",
    author = "End Language Core Team",
    version = "2.0.0",
    about = "Autonomous, Multi-Backend, 120 FPS Compiler & Real-time AI Agent Runtime for the End Language",
    long_about = None
)]
pub struct Cli {
    /// Build with release optimizations (opt-level=3, LTO)
    #[arg(long, global = true, default_value_t = false)]
    pub release: bool,

    #[command(subcommand)]
    pub command: Commands,
}
