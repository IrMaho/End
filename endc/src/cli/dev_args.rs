use clap::Args;
use std::path::PathBuf;

    /// Run comprehensive unit tests with parallel test runner and @test attribute support
        /// Path to .end source file or test file
        /// Filter test names
        /// Format as JSON for AI Agent
#[derive(Args, Debug, Clone)]
pub struct TestArgs {
        pub file: Option<PathBuf>,
        #[arg(short, long)]
        pub filter: Option<String>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Zero-downtime development server with instant hot-reload and state preservation
        /// Entry .end source file
        /// Port for development server
        /// Enable automated nanosecond function profiling and .end_diagnostics/perf_audit log generation
        /// Run single startup pass without blocking server loop
#[derive(Args, Debug, Clone)]
pub struct DevArgs {
        pub file: PathBuf,
        #[arg(short, long, default_value_t = 5000)]
        pub port: u16,
        #[arg(long, default_value_t = true)]
        pub profile: bool,
        #[arg(long, default_value_t = false)]
        pub once: bool,
}

    /// Continuous background health patrol testing runner
        /// Path to .end source file
        /// Health check interval in seconds (default: 5)
        /// Number of patrol iterations (default: 3)
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct PatrolArgs {
        pub file: PathBuf,
        #[arg(short, long, default_value_t = 5)]
        pub interval: u64,
        #[arg(short, long, default_value_t = 3)]
        pub iterations: u64,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Watch directory or files for sub-millisecond change detection
        /// Directory or file path to watch
        /// Run single inspection pass without blocking loop
#[derive(Args, Debug, Clone)]
pub struct WatchArgs {
        #[arg(default_value = ".")]
        pub path: PathBuf,
        #[arg(long, default_value_t = false)]
        pub once: bool,
}

    /// Interactive "What-If" differential mutation and simulation sandbox
        /// Path to .end source file
        /// Scenario or parameter mutation (e.g. friction=0.05)
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct SimulateArgs {
        pub file: PathBuf,
        #[arg(short, long)]
        pub scenario: Option<String>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Virtual high-scale stress testing engine (up to 1,000,000 cycles with P99 metrics)
        /// Path to .end source file
        /// Number of virtual operations (default: 1,000,000)
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct StressArgs {
        pub file: PathBuf,
        #[arg(short, long, default_value_t = 1000000)]
        pub iterations: u64,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Format End source code canonically
        /// File or directory to format
        /// Check formatting without modifying files (CI mode)
#[derive(Args, Debug, Clone)]
pub struct FmtArgs {
        #[arg(default_value = ".")]
        pub path: PathBuf,
        #[arg(long, default_value_t = false)]
        pub check: bool,
}

    /// Profile program execution (CPU, memory, SVG flamegraph)
        /// Target executable or .end source file
        /// Generate interactive SVG flamegraph
#[derive(Args, Debug, Clone)]
pub struct ProfileArgs {
        pub target: String,
        #[arg(long, default_value_t = true)]
        pub flamegraph: bool,
}

    /// Automated security fuzz testing engine (AddressSanitizer & UBSan enabled)
        /// Path to .end source file
        /// Number of fuzzing iterations (default: 10,000)
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct FuzzArgs {
        pub file: PathBuf,
        #[arg(short, long, default_value_t = 10000)]
        pub iterations: usize,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

