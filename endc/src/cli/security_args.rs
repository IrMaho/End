use clap::Args;
use std::path::PathBuf;

    /// Verify cognitive alignment between implementation and @intent formal contracts
        /// Path to .end source file
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct IntentVerifyArgs {
        pub file: PathBuf,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Formal Compiler Skill & Contract Verification (PaymentSafe, Idempotent, AuditLogged, etc.)
        /// Path to .end source file
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct VerifyArgs {
        pub file: PathBuf,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Security-by-Construction Scanner & Verified Build Gate
        /// Path to .end source file
        /// Security Level: standard, strict, paranoid, critical, absolute
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct SecurityArgs {
        pub file: PathBuf,
        #[arg(long, default_value = "paranoid")]
        pub level: String,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

/// Cryptographic Verified Build & Runtime Attestation Generator and Verifier
#[derive(Args, Debug, Clone)]
pub struct AttestArgs {
    /// Path to .end source file or target binary
    pub file: Option<PathBuf>,
    /// Path to target binary file to attest
    #[arg(long)]
    pub binary: Option<PathBuf>,
    /// Path to quote JSON file to verify against target binary
    #[arg(long)]
    pub verify: Option<PathBuf>,
    /// Force hardware TPM 2.0 attestation mode
    #[arg(long)]
    pub tpm: bool,
    /// Force software Ed25519 attestation mode
    #[arg(long)]
    pub software: bool,
    /// Path to write output quote or verification result JSON
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Output result formatted as JSON
    #[arg(long, default_value_t = true)]
    pub json: bool,
}

    /// Manage API stability, snapshots, SemVer diffs, and migration paths
        /// Action: snapshot, diff, verify, migrate
        /// Primary .end source file or v1 snapshot
        /// Secondary .end source file or v2 snapshot for diffing
        /// Format as JSON
#[derive(Args, Debug, Clone)]
pub struct ApiArgs {
        #[arg(default_value = "snapshot")]
        pub action: String,
        pub file: PathBuf,
        #[arg(short, long)]
        pub target_file: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

    /// Feature-Oriented Paradigm Lifecycle Engine (create, list, evolve, impact)
        /// Subcommand: create, list, evolve, impact
        /// Feature name, target symbol, or target path
        /// Architecture template or preset
        /// Format output as JSON
#[derive(Args, Debug, Clone)]
pub struct FeatureArgs {
        pub action: String,
        pub target: Option<String>,
        #[arg(short, long)]
        pub template: Option<String>,
        #[arg(long, default_value_t = false)]
        pub json: bool,
}

