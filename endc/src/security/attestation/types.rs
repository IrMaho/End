use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// The security provenance and trust level of the cryptographic quote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttestationKind {
    /// Cryptographic attestation backed by genuine hardware TPM 2.0.
    #[serde(rename = "tpm2")]
    Tpm2,
    /// Cryptographic software attestation (integrity signing; NOT hardware-rooted).
    #[serde(rename = "software")]
    Software,
}

impl fmt::Display for AttestationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttestationKind::Tpm2 => write!(f, "tpm2"),
            AttestationKind::Software => write!(f, "software"),
        }
    }
}

/// Hardware-rooted TPM 2.0 attestation evidence and PCR state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TpmEvidence {
    pub manufacturer: String,
    pub tpm_version: String,
    pub spec_version: String,
    pub pcr_algorithm: String,
    pub pcr_digest: String,
    pub pcr_values: BTreeMap<u32, String>,
    pub quote_signature: String,
    pub is_hardware: bool,
}

/// Canonical Cryptographic Attestation Quote.
///
/// Binds target binary measurement, environment hash, dependency hashes,
/// timestamp, and public key under an authentic cryptographic signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationQuote {
    pub kind: AttestationKind,
    pub binary_sha256: String,
    pub env_hash: String,
    pub dependency_hashes: BTreeMap<String, String>,
    pub timestamp: String,
    pub public_key: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpm_evidence: Option<TpmEvidence>,
}

/// Complete result of independent attestation verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationResult {
    pub attested: bool,
    pub kind: AttestationKind,
    pub quote: AttestationQuote,
    pub verified_at: String,
    pub summary: String,
}

/// Strongly-typed verification failure reasons (failing closed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationError {
    BinaryNotFound(String),
    BinaryIoError(String),
    TamperedBinary { expected: String, actual: String },
    EnvHashMismatch { expected: String, actual: String },
    DependencyMismatch { path: String, expected: String, actual: String },
    DependencyMissing(String),
    SignatureInvalid(String),
    FalseTpmClaim(String),
    MissingTpmEvidence(String),
    MalformedQuote(String),
    InvalidPublicKey(String),
    UnsupportedAlgorithm(String),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationError::BinaryNotFound(p) => write!(f, "Target binary not found: {}", p),
            VerificationError::BinaryIoError(e) => write!(f, "I/O error reading binary: {}", e),
            VerificationError::TamperedBinary { expected, actual } => write!(
                f,
                "TAMPER DETECTED: Binary SHA-256 mismatch (quote: {}, disk: {})",
                expected, actual
            ),
            VerificationError::EnvHashMismatch { expected, actual } => write!(
                f,
                "TAMPER DETECTED: Environment hash mismatch (quote: {}, computed: {})",
                expected, actual
            ),
            VerificationError::DependencyMismatch { path, expected, actual } => write!(
                f,
                "TAMPER DETECTED: Dependency '{}' hash mismatch (quote: {}, disk: {})",
                path, expected, actual
            ),
            VerificationError::DependencyMissing(p) => write!(f, "Dependency file missing on disk: {}", p),
            VerificationError::SignatureInvalid(msg) => write!(f, "Cryptographic signature verification failed: {}", msg),
            VerificationError::FalseTpmClaim(msg) => write!(f, "SECURITY VIOLATION: False TPM 2.0 claim detected: {}", msg),
            VerificationError::MissingTpmEvidence(msg) => write!(f, "TPM quote missing mandatory evidence: {}", msg),
            VerificationError::MalformedQuote(msg) => write!(f, "Malformed attestation quote: {}", msg),
            VerificationError::InvalidPublicKey(msg) => write!(f, "Invalid public key in quote: {}", msg),
            VerificationError::UnsupportedAlgorithm(msg) => write!(f, "Unsupported signing algorithm: {}", msg),
        }
    }
}

impl std::error::Error for VerificationError {}

/// Attestation generation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttestationError {
    TpmUnavailable(String),
    SigningFailed(String),
    IoError(String),
    MeasurementFailed(String),
}

impl fmt::Display for AttestationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttestationError::TpmUnavailable(msg) => write!(f, "TPM 2.0 is unavailable: {}", msg),
            AttestationError::SigningFailed(msg) => write!(f, "Cryptographic signing failed: {}", msg),
            AttestationError::IoError(msg) => write!(f, "I/O error: {}", msg),
            AttestationError::MeasurementFailed(msg) => write!(f, "Measurement failed: {}", msg),
        }
    }
}

impl std::error::Error for AttestationError {}
