use super::error::AiError;
use super::gguf::{parse_gguf_metadata, GgufMetadata};
use candle_core::quantized::gguf_file::Content;
use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama::ModelWeights;
use std::fs::File;
use std::path::Path;

pub const SUPPORTED_ARCHITECTURES: &[&str] = &["llama", "mistral", "vicuna"];

pub struct LlmModel {
    pub architecture: String,
    pub metadata: GgufMetadata,
    pub weights: ModelWeights,
    pub device: Device,
}

impl LlmModel {
    /// Load a GGUF model from disk with architecture validation and tensor initialization
    pub fn load_from_file(model_path: &Path, device: &Device) -> Result<Self, AiError> {
        if !model_path.exists() {
            return Err(AiError::ModelNotFound(model_path.display().to_string()));
        }

        // 1. Parse & validate GGUF metadata
        let mut meta_file = File::open(model_path)
            .map_err(|e| AiError::IoError(format!("Failed to open '{}': {}", model_path.display(), e)))?;
        let metadata = parse_gguf_metadata(&mut meta_file)?;

        // 2. Architecture detection and dispatch gate
        let arch = metadata.architecture.to_lowercase();
        if !SUPPORTED_ARCHITECTURES.iter().any(|&a| a == arch) {
            return Err(AiError::UnsupportedArchitecture {
                found: metadata.architecture.clone(),
                supported: SUPPORTED_ARCHITECTURES.iter().map(|&s| s.to_string()).collect(),
            });
        }

        // 3. Load GGUF quantized tensors via Candle
        let mut model_file = File::open(model_path)
            .map_err(|e| AiError::IoError(format!("Failed to open '{}': {}", model_path.display(), e)))?;
        let content = Content::read(&mut model_file)
            .map_err(|e| AiError::ModelLoadError(format!("Failed to parse GGUF content: {}", e)))?;

        let weights = ModelWeights::from_gguf(content, &mut model_file, device)
            .map_err(|e| AiError::ModelLoadError(format!("Failed to initialize Llama weights from GGUF: {}", e)))?;

        Ok(Self {
            architecture: metadata.architecture.clone(),
            metadata,
            weights,
            device: device.clone(),
        })
    }

    /// Forward pass through the quantized model
    pub fn forward(&mut self, input_tokens: &Tensor, pos: usize) -> Result<Tensor, AiError> {
        self.weights
            .forward(input_tokens, pos)
            .map_err(|e| AiError::InferenceError(format!("Model forward pass failed at pos {}: {}", pos, e)))
    }
}
