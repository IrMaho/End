pub mod error;
pub mod gguf;
pub mod inference;
pub mod model;
pub mod tokenizer;
pub mod ai_tests;

pub use error::AiError;
pub use gguf::{parse_gguf_metadata, validate_gguf_file, GgufMetadata, GgufTensorMeta, GGUF_MAGIC};
pub use inference::{execute_inference, InferenceConfig, InferenceResult};
pub use model::{LlmModel, SUPPORTED_ARCHITECTURES};
pub use tokenizer::LlmTokenizer;

use candle_core::Device;
use std::path::Path;

/// Convenient single-call GGUF model loader & inference executor
pub fn load_and_infer(
    model_path: &Path,
    tokenizer_path: Option<&Path>,
    prompt: &str,
    config: &InferenceConfig,
) -> Result<InferenceResult, AiError> {
    let device = Device::Cpu;
    let mut model = LlmModel::load_from_file(model_path, &device)?;

    let tokenizer = if let Some(tok_path) = tokenizer_path {
        LlmTokenizer::from_file(tok_path)?
    } else {
        // Build fallback tokenizer from GGUF metadata or default
        LlmTokenizer::from_vocab(vec![], Some(1), Some(2))
    };

    execute_inference(&mut model, &tokenizer, prompt, config)
}
