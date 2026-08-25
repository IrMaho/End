use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Provenance metadata establishing the origin of code and contract changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub agent: String,
    pub prompt_hash: String,
    pub model_version: String,
    #[serde(default)]
    pub timestamp: Option<String>,
}

impl Provenance {
    pub fn new(agent: impl Into<String>, prompt: &str, model: impl Into<String>) -> Self {
        Self {
            agent: agent.into(),
            prompt_hash: Self::hash_prompt(prompt),
            model_version: model.into(),
            timestamp: Some(chrono_like_timestamp()),
        }
    }

    /// Compute a deterministic SHA-256 hash of the input prompt.
    pub fn hash_prompt(prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Validate that all required provenance fields are present and valid.
    pub fn validate(&self) -> Result<(), ProvenanceError> {
        let agent = self.agent.trim();
        if agent.is_empty() {
            return Err(ProvenanceError::MissingField("agent".to_string()));
        }
        if is_banned_placeholder(agent) {
            return Err(ProvenanceError::InvalidPlaceholder {
                field: "agent".to_string(),
                value: agent.to_string(),
            });
        }

        let prompt_hash = self.prompt_hash.trim();
        if prompt_hash.is_empty() {
            return Err(ProvenanceError::MissingField("prompt_hash".to_string()));
        }
        if is_banned_placeholder(prompt_hash) || prompt_hash.len() < 8 {
            return Err(ProvenanceError::InvalidPlaceholder {
                field: "prompt_hash".to_string(),
                value: prompt_hash.to_string(),
            });
        }

        let model = self.model_version.trim();
        if model.is_empty() {
            return Err(ProvenanceError::MissingField("model_version".to_string()));
        }
        if is_banned_placeholder(model) {
            return Err(ProvenanceError::InvalidPlaceholder {
                field: "model_version".to_string(),
                value: model.to_string(),
            });
        }

        Ok(())
    }
}

fn is_banned_placeholder(s: &str) -> bool {
    let lower = s.to_lowercase();
    matches!(
        lower.as_str(),
        "unknown"
            | "default"
            | "test-agent"
            | "placeholder"
            | "none"
            | "n/a"
            | "todo"
            | "null"
            | "nil"
            | "fake"
            | "dummy"
    )
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvenanceError {
    MissingField(String),
    InvalidPlaceholder { field: String, value: String },
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProvenanceError::MissingField(field) => {
                write!(f, "Provenance error: required field '{}' is missing or empty", field)
            }
            ProvenanceError::InvalidPlaceholder { field, value } => {
                write!(
                    f,
                    "Provenance error: field '{}' contains prohibited placeholder value '{}'",
                    field, value
                )
            }
        }
    }
}

impl std::error::Error for ProvenanceError {}
