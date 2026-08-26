pub mod canonical;
pub mod legacy;
pub mod measurement;
pub mod software;
pub mod tpm;
pub mod types;
pub mod verifier;

#[cfg(test)]
pub mod tests;

use std::collections::BTreeMap;
use std::path::Path;

pub use legacy::{VerifiedBuildManifest, VerifiedBuildStatus};
pub use measurement::{
    measure_bytes_sha256, measure_dependencies, measure_environment, measure_file_sha256,
    SystemMeasurement,
};
pub use software::SoftwareAttestationSigner;
pub use tpm::{TpmAttestationEngine, TpmDetector, TpmStatus};
pub use types::{
    AttestationError, AttestationKind, AttestationQuote, AttestationResult, TpmEvidence,
    VerificationError,
};
pub use verifier::AttestationVerifier;

use crate::security::types::*;

/// Unified Cryptographic Attestation Engine.
pub struct AttestationEngine;

impl AttestationEngine {
    /// Generates a real cryptographic attestation quote for the given target binary or source file.
    ///
    /// If `mode` is `Some(AttestationKind::Tpm2)`, strictly requires hardware TPM 2.0.
    /// If `mode` is `Some(AttestationKind::Software)`, produces software Ed25519 attestation.
    /// If `mode` is `None` (auto), prefers TPM 2.0 if available, falling back to software.
    pub fn attest_target<P: AsRef<Path>>(
        target_path: P,
        mode: Option<AttestationKind>,
        env_vars: Option<&[(&str, &str)]>,
        dep_paths: Option<&[P]>,
        custom_signer: Option<&SoftwareAttestationSigner>,
    ) -> Result<AttestationQuote, AttestationError> {
        let binary_sha256 = measure_file_sha256(target_path.as_ref())
            .map_err(|e| AttestationError::IoError(format!("Failed to measure target binary: {}", e)))?;

        let env_hash = measure_environment(env_vars);

        let dependency_hashes = match dep_paths {
            Some(paths) => measure_dependencies(paths)
                .map_err(|e| AttestationError::IoError(format!("Failed to measure dependencies: {}", e)))?,
            None => BTreeMap::new(),
        };

        let target_kind = match mode {
            Some(k) => k,
            None => {
                let status = TpmDetector::detect();
                if status.is_present && status.is_ready {
                    AttestationKind::Tpm2
                } else {
                    AttestationKind::Software
                }
            }
        };

        match target_kind {
            AttestationKind::Tpm2 => {
                TpmAttestationEngine::sign_quote(&binary_sha256, &env_hash, &dependency_hashes, None)
            }
            AttestationKind::Software => {
                if let Some(signer) = custom_signer {
                    signer.sign_quote(&binary_sha256, &env_hash, &dependency_hashes, None)
                } else {
                    let ephemeral_signer = SoftwareAttestationSigner::generate();
                    ephemeral_signer.sign_quote(&binary_sha256, &env_hash, &dependency_hashes, None)
                }
            }
        }
    }

    /// Independently verifies a cryptographic quote against target binary, environment, and dependencies.
    pub fn verify_target<P: AsRef<Path>>(
        quote: &AttestationQuote,
        target_path: P,
        env_vars: Option<&[(&str, &str)]>,
        dep_paths: Option<&[P]>,
    ) -> Result<AttestationResult, VerificationError> {
        AttestationVerifier::verify_target(quote, target_path, env_vars, dep_paths)
    }

    /// Evaluates verified build status for compiler security gate (Pillars 4 & 5).
    pub fn evaluate_verified_build(
        source: &str,
        filename: &str,
        security_level: SecurityLevel,
        violations: &[SecurityViolation],
        proofs: &[String],
        capabilities: &[String],
    ) -> VerifiedBuildStatus {
        VerifiedBuildManifest::evaluate(source, filename, security_level, violations, proofs, capabilities)
    }
}
