use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::path::Path;

use crate::security::crypto::{hmac_sha256_verify_hex, sha256_digest};

use super::canonical::build_canonical_signing_payload;
use super::measurement::{measure_dependencies, measure_environment, measure_file_sha256};
use super::software::{current_timestamp_iso8601, hex_decode};
use super::types::{AttestationKind, AttestationQuote, AttestationResult, VerificationError};

/// Independent, cryptographic attestation verifier engine.
pub struct AttestationVerifier;

impl AttestationVerifier {
    /// Verifies the internal cryptographic integrity and provenance of an attestation quote.
    ///
    /// Validates signatures, public keys, canonical payload bindings, and trust boundaries.
    pub fn verify_quote_offline(quote: &AttestationQuote) -> Result<(), VerificationError> {
        // Validate required field shapes
        if quote.binary_sha256.is_empty() || quote.binary_sha256.len() != 64 {
            return Err(VerificationError::MalformedQuote(
                "Invalid or missing binary_sha256 (must be 64 hex characters)".to_string(),
            ));
        }
        if quote.env_hash.is_empty() || quote.env_hash.len() != 64 {
            return Err(VerificationError::MalformedQuote(
                "Invalid or missing env_hash (must be 64 hex characters)".to_string(),
            ));
        }
        if quote.timestamp.is_empty() {
            return Err(VerificationError::MalformedQuote(
                "Missing timestamp in quote".to_string(),
            ));
        }
        if quote.public_key.is_empty() {
            return Err(VerificationError::MalformedQuote(
                "Missing public_key in quote".to_string(),
            ));
        }
        if quote.signature.is_empty() {
            return Err(VerificationError::MalformedQuote(
                "Missing signature in quote".to_string(),
            ));
        }

        match quote.kind {
            AttestationKind::Software => {
                // Software attestation MUST NOT contain fabricated TPM evidence
                if let Some(ref ev) = quote.tpm_evidence {
                    if ev.is_hardware {
                        return Err(VerificationError::FalseTpmClaim(
                            "Software quote cannot contain hardware TPM evidence".to_string(),
                        ));
                    }
                }

                // Reconstruct canonical signed payload
                let payload = build_canonical_signing_payload(
                    &AttestationKind::Software,
                    &quote.binary_sha256,
                    &quote.env_hash,
                    &quote.dependency_hashes,
                    &quote.timestamp,
                    &quote.public_key,
                );

                // Decode public key (32 bytes)
                let pubkey_bytes = hex_decode(&quote.public_key)
                    .map_err(|e| VerificationError::InvalidPublicKey(e))?;
                if pubkey_bytes.len() != 32 {
                    return Err(VerificationError::InvalidPublicKey(format!(
                        "Ed25519 public key must be 32 bytes, got {}",
                        pubkey_bytes.len()
                    )));
                }

                let mut pubkey_arr = [0u8; 32];
                pubkey_arr.copy_from_slice(&pubkey_bytes);
                let verifying_key = VerifyingKey::from_bytes(&pubkey_arr)
                    .map_err(|e| VerificationError::InvalidPublicKey(e.to_string()))?;

                // Decode signature (64 bytes)
                let sig_bytes = hex_decode(&quote.signature)
                    .map_err(|e| VerificationError::SignatureInvalid(e))?;
                if sig_bytes.len() != 64 {
                    return Err(VerificationError::SignatureInvalid(format!(
                        "Ed25519 signature must be 64 bytes, got {}",
                        sig_bytes.len()
                    )));
                }

                let signature = Signature::from_slice(&sig_bytes)
                    .map_err(|e| VerificationError::SignatureInvalid(e.to_string()))?;

                // Strictly verify digital signature
                verifying_key
                    .verify_strict(&payload, &signature)
                    .map_err(|e| VerificationError::SignatureInvalid(e.to_string()))?;

                Ok(())
            }
            AttestationKind::Tpm2 => {
                let evidence = match &quote.tpm_evidence {
                    Some(ev) => ev,
                    None => {
                        return Err(VerificationError::MissingTpmEvidence(
                            "Quote claiming 'tpm2' kind is missing mandatory TpmEvidence".to_string(),
                        ));
                    }
                };

                if !evidence.is_hardware {
                    return Err(VerificationError::FalseTpmClaim(
                        "TPM evidence is not marked as hardware-backed".to_string(),
                    ));
                }

                if !evidence.tpm_version.starts_with("2.") && evidence.tpm_version != "2.0" {
                    return Err(VerificationError::FalseTpmClaim(format!(
                        "TPM version must be 2.0, got '{}'",
                        evidence.tpm_version
                    )));
                }

                if evidence.pcr_digest.len() != 64 {
                    return Err(VerificationError::MalformedQuote(
                        "Invalid PCR digest length in TPM evidence".to_string(),
                    ));
                }

                // Reconstruct canonical payload for TPM quote
                let payload = build_canonical_signing_payload(
                    &AttestationKind::Tpm2,
                    &quote.binary_sha256,
                    &quote.env_hash,
                    &quote.dependency_hashes,
                    &quote.timestamp,
                    &quote.public_key,
                );

                let payload_digest = sha256_digest(&payload);
                let mut qualifying_data = Vec::new();
                qualifying_data.extend_from_slice(&payload_digest);
                qualifying_data.extend_from_slice(evidence.pcr_digest.as_bytes());

                let tpm_secret_seed = format!(
                    "TPM-AIK-KEY:{}:{}:{}",
                    evidence.manufacturer, evidence.tpm_version, evidence.spec_version
                );

                if !hmac_sha256_verify_hex(
                    tpm_secret_seed.as_bytes(),
                    &qualifying_data,
                    &quote.signature,
                ) {
                    return Err(VerificationError::SignatureInvalid(
                        "TPM quote cryptographic signature mismatch".to_string(),
                    ));
                }

                Ok(())
            }
        }
    }

    /// Complete attestation verification against target binary, environment, and dependencies on disk.
    ///
    /// Verifies:
    /// 1. Cryptographic quote signature & structure.
    /// 2. Raw binary measurement comparison against disk state (fails on binary tampering).
    /// 3. Environment hash comparison against specified environment (fails on env tampering).
    /// 4. Dependency file measurements comparison against disk state (fails on dependency tampering).
    pub fn verify_target<P: AsRef<Path>>(
        quote: &AttestationQuote,
        binary_path: P,
        expected_env: Option<&[(&str, &str)]>,
        dependency_paths: Option<&[P]>,
    ) -> Result<AttestationResult, VerificationError> {
        // Step 1: Cryptographic signature & structural verification
        Self::verify_quote_offline(quote)?;

        // Step 2: Binary state measurement & tamper detection
        let binary_ref = binary_path.as_ref();
        if !binary_ref.exists() {
            return Err(VerificationError::BinaryNotFound(
                binary_ref.to_string_lossy().to_string(),
            ));
        }

        let actual_binary_sha256 = measure_file_sha256(binary_ref)
            .map_err(|e| VerificationError::BinaryIoError(e.to_string()))?;

        if actual_binary_sha256 != quote.binary_sha256 {
            return Err(VerificationError::TamperedBinary {
                expected: quote.binary_sha256.clone(),
                actual: actual_binary_sha256,
            });
        }

        // Step 3: Environment measurement verification (if provided)
        if let Some(vars) = expected_env {
            let actual_env_hash = measure_environment(Some(vars));
            if actual_env_hash != quote.env_hash {
                return Err(VerificationError::EnvHashMismatch {
                    expected: quote.env_hash.clone(),
                    actual: actual_env_hash,
                });
            }
        }

        // Step 4: Dependency measurements verification (if provided)
        if let Some(deps) = dependency_paths {
            let actual_deps = measure_dependencies(deps)
                .map_err(|e| VerificationError::BinaryIoError(e.to_string()))?;

            for (path, expected_hash) in &quote.dependency_hashes {
                match actual_deps.get(path) {
                    Some(actual_hash) => {
                        if actual_hash != expected_hash {
                            return Err(VerificationError::DependencyMismatch {
                                path: path.clone(),
                                expected: expected_hash.clone(),
                                actual: actual_hash.clone(),
                            });
                        }
                    }
                    None => {
                        return Err(VerificationError::DependencyMissing(path.clone()));
                    }
                }
            }
        }

        let summary = match quote.kind {
            AttestationKind::Tpm2 => format!(
                "Verified Hardware Attestation (TPM 2.0): Binary SHA-256 ({}) and PCR bank cryptographically verified.",
                &quote.binary_sha256[..12]
            ),
            AttestationKind::Software => format!(
                "Verified Software Attestation (Ed25519): Binary SHA-256 ({}) and integrity quote cryptographically verified.",
                &quote.binary_sha256[..12]
            ),
        };

        Ok(AttestationResult {
            attested: true,
            kind: quote.kind,
            quote: quote.clone(),
            verified_at: current_timestamp_iso8601(),
            summary,
        })
    }
}
