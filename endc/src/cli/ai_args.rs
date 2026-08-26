use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct AiArgs {
    /// Action: inspect, infer, validate
    #[arg(default_value = "infer")]
    pub action: String,

    /// Path to .gguf model file
    #[arg(short, long)]
    pub model: String,

    /// Input prompt for inference
    #[arg(short, long, default_value = "Hello")]
    pub prompt: String,

    /// Maximum tokens to generate
    #[arg(long, default_value_t = 32)]
    pub max_tokens: usize,

    /// Sampling temperature (0.0 = deterministic)
    #[arg(long, default_value_t = 0.0)]
    pub temperature: f64,

    /// Random seed
    #[arg(long, default_value_t = 42)]
    pub seed: u64,

    /// Output report in JSON format
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
