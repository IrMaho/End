use serde::{Deserialize, Serialize};
use std::fmt;

/// Strongly-typed, actionable AI Runtime Errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiError {
    ModelNotFound(String),
    InvalidGguf(String),
    UnsupportedArchitecture {
        found: String,
        supported: Vec<String>,
    },
    TokenizerError(String),
    ModelLoadError(String),
    InferenceError(String),
    SamplingError(String),
    DeviceError(String),
    IoError(String),
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::ModelNotFound(path) => write!(f, "AI Model file not found: {}", path),
            AiError::InvalidGguf(msg) => write!(f, "Invalid or corrupt GGUF file: {}", msg),
            AiError::UnsupportedArchitecture { found, supported } => write!(
                f,
                "Unsupported GGUF model architecture '{}'. Supported architectures: [{}]",
                found,
                supported.join(", ")
            ),
            AiError::TokenizerError(msg) => write!(f, "Tokenizer error: {}", msg),
            AiError::ModelLoadError(msg) => write!(f, "Failed to load model weights: {}", msg),
            AiError::InferenceError(msg) => write!(f, "Inference forward pass failed: {}", msg),
            AiError::SamplingError(msg) => write!(f, "Logits sampling error: {}", msg),
            AiError::DeviceError(msg) => write!(f, "Compute device error: {}", msg),
            AiError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for AiError {}

impl From<std::io::Error> for AiError {
    fn from(err: std::io::Error) -> Self {
        AiError::IoError(err.to_string())
    }
}

impl From<candle_core::Error> for AiError {
    fn from(err: candle_core::Error) -> Self {
        AiError::InferenceError(err.to_string())
    }
}
