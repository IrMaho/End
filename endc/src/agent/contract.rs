use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use super::lifecycle::LifecycleState;
use super::provenance::{Provenance, ProvenanceError};

/// Canonical relative path for the project agent contract file.
pub const CONTRACT_REL_PATH: &str = ".agents/contract.toml";

/// Supported operation identifiers for the `allowed_operations` whitelist.
pub const SUPPORTED_OPERATIONS: &[&str] = &[
    "file_read",
    "file_write",
    "net_listen",
    "net_connect",
    "db_query",
    "env_read",
    "env_write",
    "exec_subprocess",
    "crypto_hash",
    "crypto_sign",
    "time_read",
];

/// Supported security boundary rule identifiers.
pub const SUPPORTED_SECURITY_BOUNDARIES: &[&str] = &[
    "no_outbound_network",
    "no_inbound_network",
    "no_exec_subprocess",
    "no_env_access",
    "no_file_write",
    "read_only_fs",
    "pure_computation",
];

/// Data model representing an AI Agent Task Contract (`.agents/contract.toml`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentContract {
    pub task_id: String,
    pub intent: String,

    #[serde(default)]
    pub requirements: Vec<String>,

    #[serde(default)]
    pub preconditions: Vec<String>,

    #[serde(default)]
    pub postconditions: Vec<String>,

    #[serde(default)]
    pub allowed_operations: Vec<String>,

    #[serde(default)]
    pub required_tests: Vec<String>,

    #[serde(default)]
    pub evidence_requirements: Vec<String>,

    #[serde(default)]
    pub security_boundaries: Vec<String>,

    #[serde(default)]
    pub target_files: Vec<String>,

    #[serde(default)]
    pub artifact_hashes: HashMap<String, String>,

    pub provenance: Provenance,

    #[serde(default = "default_lifecycle")]
    pub lifecycle: LifecycleState,
}

fn default_lifecycle() -> LifecycleState {
    LifecycleState::Draft
}

impl AgentContract {
    /// Parse a contract from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, ContractError> {
        let contract: AgentContract = toml::from_str(content)
            .map_err(|e| ContractError::ParseError(e.to_string()))?;
        contract.validate()?;
        Ok(contract)
    }

    /// Load and parse a contract from a file.
    pub fn from_file(path: &Path) -> Result<Self, ContractError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ContractError::IoError(format!("Failed to read contract from {:?}: {}", path, e)))?;
        Self::from_toml(&content)
    }

    /// Serialize the contract to a formatted TOML string.
    pub fn to_toml(&self) -> Result<String, ContractError> {
        toml::to_string_pretty(self)
            .map_err(|e| ContractError::SerializationError(e.to_string()))
    }

    /// Save the contract to a file on disk.
    pub fn save_to_file(&self, path: &Path) -> Result<(), ContractError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ContractError::IoError(format!("Failed to create directory {:?}: {}", parent, e))
            })?;
        }
        let toml_str = self.to_toml()?;
        fs::write(path, toml_str).map_err(|e| {
            ContractError::IoError(format!("Failed to write contract to {:?}: {}", path, e))
        })
    }

    /// Validate the contract structure, required fields, and semantic consistency.
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.task_id.trim().is_empty() {
            return Err(ContractError::ValidationError(
                "Contract 'task_id' must not be empty".to_string(),
            ));
        }

        if self.intent.trim().is_empty() {
            return Err(ContractError::ValidationError(
                "Contract 'intent' must not be empty".to_string(),
            ));
        }

        // Validate provenance
        self.provenance.validate().map_err(ContractError::Provenance)?;

        // Validate allowed operations
        for op in &self.allowed_operations {
            let op_clean = op.trim();
            if op_clean.is_empty() {
                return Err(ContractError::ValidationError(
                    "Empty operation name in 'allowed_operations'".to_string(),
                ));
            }
            if !SUPPORTED_OPERATIONS.contains(&op_clean) {
                return Err(ContractError::ValidationError(format!(
                    "Unknown operation '{}' in 'allowed_operations'. Supported operations: {:?}",
                    op_clean, SUPPORTED_OPERATIONS
                )));
            }
        }

        // Validate security boundaries
        for boundary in &self.security_boundaries {
            let b_clean = boundary.trim();
            if b_clean.is_empty() {
                return Err(ContractError::ValidationError(
                    "Empty security boundary in 'security_boundaries'".to_string(),
                ));
            }
            if !SUPPORTED_SECURITY_BOUNDARIES.contains(&b_clean) {
                return Err(ContractError::ValidationError(format!(
                    "Unknown security boundary '{}'. Supported boundaries: {:?}",
                    b_clean, SUPPORTED_SECURITY_BOUNDARIES
                )));
            }
        }

        // Check boundary vs allowed_operations conflicts
        if self.security_boundaries.contains(&"no_outbound_network".to_string())
            && self.allowed_operations.contains(&"net_connect".to_string())
        {
            return Err(ContractError::ValidationError(
                "Conflict: 'security_boundaries' specifies 'no_outbound_network' but 'allowed_operations' includes 'net_connect'".to_string(),
            ));
        }

        if self.security_boundaries.contains(&"no_inbound_network".to_string())
            && self.allowed_operations.contains(&"net_listen".to_string())
        {
            return Err(ContractError::ValidationError(
                "Conflict: 'security_boundaries' specifies 'no_inbound_network' but 'allowed_operations' includes 'net_listen'".to_string(),
            ));
        }

        if self.security_boundaries.contains(&"no_exec_subprocess".to_string())
            && self.allowed_operations.contains(&"exec_subprocess".to_string())
        {
            return Err(ContractError::ValidationError(
                "Conflict: 'security_boundaries' specifies 'no_exec_subprocess' but 'allowed_operations' includes 'exec_subprocess'".to_string(),
            ));
        }

        if self.security_boundaries.contains(&"no_file_write".to_string())
            && self.allowed_operations.contains(&"file_write".to_string())
        {
            return Err(ContractError::ValidationError(
                "Conflict: 'security_boundaries' specifies 'no_file_write' but 'allowed_operations' includes 'file_write'".to_string(),
            ));
        }

        Ok(())
    }

    /// Helper to find `.agents/contract.toml` starting from `start_dir` and scanning ancestors.
    pub fn find_contract_file(start_dir: &Path) -> Option<PathBuf> {
        let mut curr = if start_dir.is_file() {
            start_dir.parent()
        } else {
            Some(start_dir)
        };

        while let Some(dir) = curr {
            let candidate = dir.join(CONTRACT_REL_PATH);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
            // Also check direct contract.toml if in .agents directory
            let direct_contract = dir.join("contract.toml");
            if dir.file_name().and_then(|s| s.to_str()) == Some(".agents")
                && direct_contract.exists()
                && direct_contract.is_file()
            {
                return Some(direct_contract);
            }
            curr = dir.parent();
        }
        None
    }
}

/// Comprehensive errors occurring during contract lifecycle, parsing, or validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractError {
    ParseError(String),
    SerializationError(String),
    ValidationError(String),
    Provenance(ProvenanceError),
    IoError(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractError::ParseError(msg) => write!(f, "Contract Parse Error: {}", msg),
            ContractError::SerializationError(msg) => {
                write!(f, "Contract Serialization Error: {}", msg)
            }
            ContractError::ValidationError(msg) => write!(f, "Contract Validation Error: {}", msg),
            ContractError::Provenance(err) => write!(f, "{}", err),
            ContractError::IoError(msg) => write!(f, "Contract I/O Error: {}", msg),
        }
    }
}

impl std::error::Error for ContractError {}
