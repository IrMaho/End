use super::error::AiError;
use super::model::LlmModel;
use super::tokenizer::LlmTokenizer;
use candle_core::Tensor;
use candle_transformers::generation::LogitsProcessor;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: Option<f64>,
    pub seed: u64,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_tokens: 32,
            temperature: 0.0, // Default deterministic greedy decoding
            top_p: None,
            seed: 42,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub model: String,
    pub architecture: String,
    pub prompt: String,
    pub prompt_tokens: Vec<u32>,
    pub seed: u64,
    pub temperature: f64,
    pub max_tokens: usize,
    pub generated_tokens: Vec<String>,
    pub generated_token_ids: Vec<u32>,
    pub output_text: String,
    pub duration_ms: u128,
    pub tokens_per_second: f64,
    pub stopped_on_eos: bool,
    pub status: String,
}

pub fn execute_inference(
    model: &mut LlmModel,
    tokenizer: &LlmTokenizer,
    prompt: &str,
    config: &InferenceConfig,
) -> Result<InferenceResult, AiError> {
    let start_time = Instant::now();

    // 1. Tokenize prompt
    let prompt_tokens = tokenizer.encode(prompt, true)?;
    if prompt_tokens.is_empty() {
        return Err(AiError::InferenceError("Empty tokenized prompt".to_string()));
    }

    let eos_token_id = tokenizer.eos_token_id();
    let mut logits_processor = if config.temperature <= 0.0 {
        LogitsProcessor::new(config.seed, None, None)
    } else {
        LogitsProcessor::new(config.seed, Some(config.temperature), config.top_p)
    };

    let device = model.device.clone();
    let mut generated_token_ids: Vec<u32> = Vec::new();
    let mut generated_tokens: Vec<String> = Vec::new();
    let mut all_tokens = prompt_tokens.clone();
    let mut stopped_on_eos = false;

    // 2. Pre-fill prompt forward pass
    let prompt_len = prompt_tokens.len();
    let input_tensor = Tensor::new(prompt_tokens.as_slice(), &device)
        .map_err(|e| AiError::InferenceError(format!("Failed to create prompt tensor: {}", e)))?
        .unsqueeze(0)
        .map_err(|e| AiError::InferenceError(format!("Failed to unsqueeze prompt tensor: {}", e)))?;

    let mut logits = model.forward(&input_tensor, 0)?;

    // 3. Auto-regressive generation loop
    for i in 0..config.max_tokens {
        let next_token_id = {
            let logits_sq = logits
                .squeeze(0)
                .map_err(|e| AiError::InferenceError(format!("Failed to squeeze logits: {}", e)))?;

            // Extract logits of last position
            let last_logits = if logits_sq.dims().len() > 1 {
                let last_idx = logits_sq.dim(0).map_err(|e| AiError::InferenceError(e.to_string()))? - 1;
                logits_sq
                    .get(last_idx)
                    .map_err(|e| AiError::InferenceError(format!("Failed to index logits: {}", e)))?
            } else {
                logits_sq
            };

            // Apply repetition penalty if configured
            let penalized_logits = if config.repeat_penalty != 1.0 && !all_tokens.is_empty() {
                let start_at = all_tokens.len().saturating_sub(config.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &last_logits,
                    config.repeat_penalty,
                    &all_tokens[start_at..],
                )
                .unwrap_or(last_logits)
            } else {
                last_logits
            };

            // Sample next token ID
            logits_processor
                .sample(&penalized_logits)
                .map_err(|e| AiError::SamplingError(format!("Failed to sample next token: {}", e)))?
        };

        all_tokens.push(next_token_id);
        generated_token_ids.push(next_token_id);

        let tok_str = tokenizer.decode_single_token(next_token_id).unwrap_or_default();
        generated_tokens.push(tok_str);

        // Check EOS condition
        if Some(next_token_id) == eos_token_id {
            stopped_on_eos = true;
            break;
        }

        // Forward single next token
        if i + 1 < config.max_tokens {
            let next_input = Tensor::new(&[next_token_id], &device)
                .map_err(|e| AiError::InferenceError(format!("Failed to create token tensor: {}", e)))?
                .unsqueeze(0)
                .map_err(|e| AiError::InferenceError(format!("Failed to unsqueeze token tensor: {}", e)))?;

            logits = model.forward(&next_input, prompt_len + i)?;
        }
    }

    let duration_ms = start_time.elapsed().as_millis();
    let duration_sec = start_time.elapsed().as_secs_f64();
    let tokens_per_second = if duration_sec > 0.0 {
        generated_token_ids.len() as f64 / duration_sec
    } else {
        0.0
    };

    let output_text = tokenizer
        .decode(&generated_token_ids)
        .unwrap_or_else(|_| generated_tokens.join(""));

    Ok(InferenceResult {
        model: model.metadata.model_name.clone().unwrap_or_else(|| "GGUF-Model".to_string()),
        architecture: model.architecture.clone(),
        prompt: prompt.to_string(),
        prompt_tokens,
        seed: config.seed,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        generated_tokens,
        generated_token_ids,
        output_text,
        duration_ms,
        tokens_per_second,
        stopped_on_eos,
        status: "success".to_string(),
    })
}
