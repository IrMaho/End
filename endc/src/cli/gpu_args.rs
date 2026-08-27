use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct GpuArgs {
    #[command(subcommand)]
    pub action: GpuAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GpuAction {
    /// Inspect available GPU adapters, compute capabilities, and active backend
    Info {
        /// Output information as machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Run real GPU vector addition test benchmark
    VecAdd {
        /// Number of float32 elements (e.g. 1000, 1000000, 64000000)
        #[arg(short, long, default_value = "1000000")]
        size: usize,
        /// Output execution report as machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Run real GPU matrix multiplication benchmark
    Matmul {
        /// Matrix dimension M
        #[arg(short, long, default_value = "256")]
        m: u32,
        /// Matrix dimension K
        #[arg(short, long, default_value = "256")]
        k: u32,
        /// Matrix dimension N
        #[arg(short, long, default_value = "256")]
        n: u32,
        /// Output execution report as machine-readable JSON
        #[arg(long)]
        json: bool,
    },
}
