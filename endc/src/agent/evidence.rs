use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::lifecycle::LifecycleState;
use super::provenance::Provenance;
use super::signing::{compute_signature_string, hex_encode};

/// Supported schema versions for agent evidence bundles.
pub const SUPPORTED_SCHEMA_VERSIONS: &[&str] = &["1.0", "1.0.0"];

/// Default schema version for new evidence bundles.
pub const DEFAULT_SCHEMA_VERSION: &str = "1.0";

/// Structured error conditions for evidence verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    Tampered {
        expected_sig: String,
        actual_sig: String,
    },
    IncompatibleSchemaVersion {
        version: String,
        supported: Vec<String>,
    },
    Stale {
        details: Vec<String>,
    },
    IoError(String),
    ParseError(String),
    MissingSignature,
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvidenceError::Tampered { .. } => write!(f, "EVIDENCE_TAMPERED"),
            EvidenceError::IncompatibleSchemaVersion { version, .. } => {
                write!(f, "SCHEMA_VERSION_INCOMPATIBLE: version '{}' is not supported", version)
            }
            EvidenceError::Stale { details } => {
                write!(f, "STALE: evidence out of date with disk: {}", details.join("; "))
            }
            EvidenceError::IoError(msg) => write!(f, "IO Error: {}", msg),
            EvidenceError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            EvidenceError::MissingSignature => write!(f, "EVIDENCE_TAMPERED: missing signature field"),
        }
    }
}

impl std::error::Error for EvidenceError {}

/// Test execution record within the evidence bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestExecutionRecord {
    pub name: String,
    pub pass: bool,
    pub duration_ms: u64,
    pub stdout_hash: String,
    pub stderr_hash: String,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Artifact hash bundle categorized by source, generated C, and native binaries.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ArtifactBundle {
    #[serde(default)]
    pub source_files: BTreeMap<String, String>,
    #[serde(default)]
    pub generated_c: BTreeMap<String, String>,
    #[serde(default)]
    pub binaries: BTreeMap<String, String>,
}

/// Code coverage metrics captured during verification.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CoverageInfo {
    pub lines_total: usize,
    pub lines_covered: usize,
    pub branches_total: usize,
    pub branches_covered: usize,
}

/// Environment metadata of the host compiler toolchain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub endc_version: String,
    pub gcc_version: String,
    pub target: String,
    pub os: String,
}

impl Default for EnvironmentInfo {
    fn default() -> Self {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let target = format!("{}-{}", arch, os);
        Self {
            endc_version: env!("CARGO_PKG_VERSION").to_string(),
            gcc_version: detect_gcc_version(),
            target,
            os: os.to_string(),
        }
    }
}

fn detect_gcc_version() -> String {
    let output = std::process::Command::new("gcc").arg("--version").output();
    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines().next().unwrap_or("gcc (detected)").trim().to_string()
        }
        _ => "gcc 14.2.0 (fallback)".to_string(),
    }
}

/// Assertion comparison details in structured repair feedback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionDetail {
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
}

/// Suggested source fix area in structured repair feedback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedFixArea {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub hint: String,
}

/// A persistent record of a verification repair attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairAttempt {
    pub attempt_number: usize,
    pub timestamp: String,
    pub failure_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_test: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertion: Option<AssertionDetail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_fix_area: Option<SuggestedFixArea>,
    pub resolved: bool,
}

/// Canonical payload of evidence fields for deterministic HMAC signing.
/// Excludes the `signature` field to prevent circular self-reference.
#[derive(Debug, Clone, Serialize)]
struct CanonicalEvidencePayload<'a> {
    schema_version: &'a str,
    contract_id: &'a str,
    state: &'a LifecycleState,
    signed_at: &'a str,
    tests: &'a [TestExecutionRecord],
    artifacts: &'a ArtifactBundle,
    coverage: &'a CoverageInfo,
    environment: &'a EnvironmentInfo,
    rebuild_deterministic: bool,
    repair_attempts: &'a [RepairAttempt],
}

/// Complete Evidence Bundle representing a verified or audited contract execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub schema_version: String,
    pub contract_id: String,
    pub state: LifecycleState,
    pub signed_at: String,
    #[serde(default)]
    pub signature: String,
    #[serde(default)]
    pub tests: Vec<TestExecutionRecord>,
    #[serde(default)]
    pub artifacts: ArtifactBundle,
    #[serde(default)]
    pub coverage: CoverageInfo,
    pub environment: EnvironmentInfo,
    pub rebuild_deterministic: bool,
    #[serde(default)]
    pub repair_attempts: Vec<RepairAttempt>,
}

impl EvidenceBundle {
    /// Create a new evidence bundle for a contract.
    pub fn new(
        contract_id: impl Into<String>,
        state: LifecycleState,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let signed_at = format!("{}.{:03}Z", now.as_secs(), now.subsec_millis());

        Self {
            schema_version: DEFAULT_SCHEMA_VERSION.to_string(),
            contract_id: contract_id.into(),
            state,
            signed_at,
            signature: String::new(),
            tests: Vec::new(),
            artifacts: ArtifactBundle::default(),
            coverage: CoverageInfo::default(),
            environment: EnvironmentInfo::default(),
            rebuild_deterministic: false,
            repair_attempts: Vec::new(),
        }
    }

    /// Check if the bundle's schema version is supported.
    pub fn check_schema_version(&self) -> Result<(), EvidenceError> {
        if !SUPPORTED_SCHEMA_VERSIONS.contains(&self.schema_version.as_str()) {
            return Err(EvidenceError::IncompatibleSchemaVersion {
                version: self.schema_version.clone(),
                supported: SUPPORTED_SCHEMA_VERSIONS.iter().map(|s| s.to_string()).collect(),
            });
        }
        Ok(())
    }

    /// Generate canonical serialized JSON bytes over all fields except `signature`.
    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, EvidenceError> {
        let payload = CanonicalEvidencePayload {
            schema_version: &self.schema_version,
            contract_id: &self.contract_id,
            state: &self.state,
            signed_at: &self.signed_at,
            tests: &self.tests,
            artifacts: &self.artifacts,
            coverage: &self.coverage,
            environment: &self.environment,
            rebuild_deterministic: self.rebuild_deterministic,
            repair_attempts: &self.repair_attempts,
        };
        serde_json::to_vec(&payload).map_err(|e| EvidenceError::ParseError(e.to_string()))
    }

    /// Sign the evidence bundle with HMAC-SHA256 using the provided secret key.
    pub fn sign(&mut self, secret_key: &[u8]) -> Result<(), EvidenceError> {
        let canonical_bytes = self.canonical_payload_bytes()?;
        self.signature = compute_signature_string(secret_key, &canonical_bytes);
        Ok(())
    }

    /// Verify the evidence bundle's signature against the provided secret key.
    /// Rejects if signature is missing or tampered.
    pub fn verify_signature(&self, secret_key: &[u8]) -> Result<(), EvidenceError> {
        if self.signature.is_empty() {
            return Err(EvidenceError::MissingSignature);
        }

        let canonical_bytes = self.canonical_payload_bytes()?;
        let expected_sig = compute_signature_string(secret_key, &canonical_bytes);

        if self.signature != expected_sig {
            return Err(EvidenceError::Tampered {
                expected_sig,
                actual_sig: self.signature.clone(),
            });
        }

        Ok(())
    }

    /// Load and strictly verify an evidence bundle from JSON file.
    pub fn load_and_verify(
        file_path: &Path,
        secret_key: &[u8],
    ) -> Result<Self, EvidenceError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| EvidenceError::IoError(e.to_string()))?;

        let bundle: Self = serde_json::from_str(&content)
            .map_err(|e| EvidenceError::ParseError(e.to_string()))?;

        // 1. Schema check
        bundle.check_schema_version()?;

        // 2. Tamper check via HMAC signature
        bundle.verify_signature(secret_key)?;

        Ok(bundle)
    }

    /// Save the evidence bundle to disk at `.agents/evidence/<contract_id>.json` and `.agents/evidence.json`.
    pub fn save_to_dir(&self, base_dir: &Path) -> Result<PathBuf, std::io::Error> {
        let evidence_dir = base_dir.join(".agents/evidence");
        fs::create_dir_all(&evidence_dir)?;

        let target_path = evidence_dir.join(format!("{}.json", self.contract_id));
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&target_path, &json_str)?;

        // Also write to .agents/evidence.json for backwards-compatibility
        let legacy_path = base_dir.join(".agents/evidence.json");
        let _ = fs::write(&legacy_path, &json_str);

        Ok(target_path)
    }
}

// Backwards compatibility aliases for existing codebase
pub type ContractEvidence = EvidenceBundle;
pub type SecurityBoundaryCheckResult = SecurityCheckStub;
pub type PostconditionCheckResult = PostconditionStub;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SecurityCheckStub {
    pub boundary: String,
    pub satisfied: bool,
    pub detected_operations: Vec<String>,
    pub violating_locations: Vec<String>,
    pub diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PostconditionStub {
    pub description: String,
    pub satisfied: bool,
    pub details: String,
}
