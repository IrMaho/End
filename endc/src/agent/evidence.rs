use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::lifecycle::LifecycleState;
use super::provenance::Provenance;

/// Canonical relative path for evidence persistence.
pub const EVIDENCE_REL_PATH: &str = ".agents/evidence.json";

/// Record of an executed required test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionRecord {
    pub test_name: String,
    pub path: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
}

/// Result of checking a security boundary rule against scanned code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityBoundaryCheckResult {
    pub boundary: String,
    pub satisfied: bool,
    pub detected_operations: Vec<String>,
    pub violating_locations: Vec<String>,
    pub diagnostic: String,
}

/// Result of postcondition / requirement verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostconditionCheckResult {
    pub description: String,
    pub satisfied: bool,
    pub details: String,
}

/// Complete verifiable execution evidence captured during contract verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvidence {
    pub contract_id: String,
    pub intent: String,
    pub lifecycle_state: LifecycleState,
    pub verified: bool,
    pub tests_executed: Vec<TestExecutionRecord>,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub total_duration_ms: u64,
    pub artifact_hashes: HashMap<String, String>,
    pub provenance: Provenance,
    pub security_boundary_checks: Vec<SecurityBoundaryCheckResult>,
    pub postcondition_checks: Vec<PostconditionCheckResult>,
    pub collection_timestamp: String,
    pub failure_reasons: Vec<String>,
}

impl ContractEvidence {
    pub fn new(contract_id: String, intent: String, provenance: Provenance) -> Self {
        Self {
            contract_id,
            intent,
            lifecycle_state: LifecycleState::Verifying,
            verified: false,
            tests_executed: Vec::new(),
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            total_duration_ms: 0,
            artifact_hashes: HashMap::new(),
            provenance,
            security_boundary_checks: Vec::new(),
            postcondition_checks: Vec::new(),
            collection_timestamp: chrono_like_timestamp(),
            failure_reasons: Vec::new(),
        }
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = self.to_json_pretty();
        fs::write(path, json)
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}
