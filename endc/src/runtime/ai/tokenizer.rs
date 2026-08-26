use super::error::AiError;
use std::path::Path;
use tokenizers::Tokenizer as HfTokenizer;

#[derive(Debug, Clone)]
pub struct LlmTokenizer {
    inner: Option<HfTokenizer>,
    vocab_map: Option<Vec<String>>,
    bos_token_id: Option<u32>,
    eos_token_id: Option<u32>,
}

impl LlmTokenizer {
    /// Create from a HuggingFace Tokenizer JSON file or raw JSON string
    pub fn from_hf_json(json_str: &str) -> Result<Self, AiError> {
        let hf_tok = HfTokenizer::from_bytes(json_str.as_bytes())
            .map_err(|e| AiError::TokenizerError(format!("Failed to parse HF tokenizer JSON: {}", e)))?;
        
        let eos_token_id = hf_tok.token_to_id("</s>")
            .or_else(|| hf_tok.token_to_id("<|endoftext|>"))
            .or_else(|| hf_tok.token_to_id("<|eot_id|>"));

        let bos_token_id = hf_tok.token_to_id("<s>")
            .or_else(|| hf_tok.token_to_id("<|begin_of_text|>"));

        Ok(Self {
            inner: Some(hf_tok),
            vocab_map: None,
            bos_token_id,
            eos_token_id,
        })
    }

    /// Create from a HuggingFace tokenizer.json file on disk
    pub fn from_file(path: &Path) -> Result<Self, AiError> {
        let hf_tok = HfTokenizer::from_file(path)
            .map_err(|e| AiError::TokenizerError(format!("Failed to load tokenizer from '{}': {}", path.display(), e)))?;

        let eos_token_id = hf_tok.token_to_id("</s>")
            .or_else(|| hf_tok.token_to_id("<|endoftext|>"))
            .or_else(|| hf_tok.token_to_id("<|eot_id|>"));

        let bos_token_id = hf_tok.token_to_id("<s>")
            .or_else(|| hf_tok.token_to_id("<|begin_of_text|>"));

        Ok(Self {
            inner: Some(hf_tok),
            vocab_map: None,
            bos_token_id,
            eos_token_id,
        })
    }

    /// Create from GGUF embedded vocabulary tokens
    pub fn from_vocab(tokens: Vec<String>, bos_id: Option<u32>, eos_id: Option<u32>) -> Self {
        Self {
            inner: None,
            vocab_map: Some(tokens),
            bos_token_id: bos_id.or(Some(1)),
            eos_token_id: eos_id.or(Some(2)),
        }
    }

    pub fn bos_token_id(&self) -> Option<u32> {
        self.bos_token_id
    }

    pub fn eos_token_id(&self) -> Option<u32> {
        self.eos_token_id
    }

    pub fn encode(&self, text: &str, add_bos: bool) -> Result<Vec<u32>, AiError> {
        if let Some(ref hf) = self.inner {
            let encoding = hf
                .encode(text, add_bos)
                .map_err(|e| AiError::TokenizerError(format!("Tokenization encoding error: {}", e)))?;
            Ok(encoding.get_ids().to_vec())
        } else if let Some(ref vocab) = self.vocab_map {
            // Simple direct vocabulary lookup / character fallback
            let mut ids = Vec::new();
            if add_bos {
                if let Some(bos) = self.bos_token_id {
                    ids.push(bos);
                }
            }
            // Byte-level tokenization or word lookup
            for word in text.split_inclusive(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
                let mut found = false;
                for (idx, tok) in vocab.iter().enumerate() {
                    if tok == word || tok == &format!(" {}", word) {
                        ids.push(idx as u32);
                        found = true;
                        break;
                    }
                }
                if !found {
                    // Byte fallback
                    for b in word.bytes() {
                        let b_tok = format!("<0x{:02X}>", b);
                        if let Some(idx) = vocab.iter().position(|t| t == &b_tok) {
                            ids.push(idx as u32);
                        } else {
                            ids.push(b as u32);
                        }
                    }
                }
            }
            Ok(ids)
        } else {
            Err(AiError::TokenizerError("No tokenizer backend initialized".to_string()))
        }
    }

    pub fn decode(&self, token_ids: &[u32]) -> Result<String, AiError> {
        if let Some(ref hf) = self.inner {
            hf.decode(token_ids, true)
                .map_err(|e| AiError::TokenizerError(format!("Token decoding error: {}", e)))
        } else if let Some(ref vocab) = self.vocab_map {
            let mut result = String::new();
            for &id in token_ids {
                if let Some(tok) = vocab.get(id as usize) {
                    if tok.starts_with("<0x") && tok.ends_with('>') && tok.len() == 6 {
                        if let Ok(byte_val) = u8::from_str_radix(&tok[3..5], 16) {
                            result.push(byte_val as char);
                            continue;
                        }
                    }
                    if tok == "<s>" || tok == "</s>" || tok == "<unk>" {
                        continue;
                    }
                    // Handle SentencePiece leading space underscore ' ' (U+2581)
                    let clean = tok.replace('\u{2581}', " ");
                    result.push_str(&clean);
                }
            }
            Ok(result)
        } else {
            Err(AiError::TokenizerError("No tokenizer backend initialized".to_string()))
        }
    }

    pub fn decode_single_token(&self, token_id: u32) -> Result<String, AiError> {
        self.decode(&[token_id])
    }
}
