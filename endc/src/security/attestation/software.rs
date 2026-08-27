use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand_core::OsRng;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::canonical::build_canonical_signing_payload;
use super::types::{AttestationError, AttestationKind, AttestationQuote};

/// Software-level cryptographic attestation signer.
///
/// Emits authentic Ed25519 digital signatures over deterministic canonical payloads.
/// The resulting quote is explicitly labeled `attestation_kind = "software"` to provide
/// truthful security provenance without claiming hardware-rooted trust.
pub struct SoftwareAttestationSigner {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl SoftwareAttestationSigner {
    /// Generates a new cryptographically secure random Ed25519 keypair using OS entropy.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Initializes a signer from a 32-byte raw private key seed.
    pub fn from_seed_bytes(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Initializes a signer from a 64-character hex seed string.
    pub fn from_hex_seed(hex_str: &str) -> Result<Self, AttestationError> {
        let bytes = hex_decode(hex_str)
            .map_err(|e| AttestationError::SigningFailed(format!("Invalid hex seed: {}", e)))?;
        if bytes.len() != 32 {
            return Err(AttestationError::SigningFailed(format!(
                "Seed must be 32 bytes (64 hex characters), got {} bytes",
                bytes.len()
            )));
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&bytes);
        Ok(Self::from_seed_bytes(&seed))
    }

    /// Returns the hex-encoded 32-byte Ed25519 public key.
    pub fn public_key_hex(&self) -> String {
        hex_encode(self.verifying_key.as_bytes())
    }

    /// Generates a real signed `AttestationQuote` with `kind: AttestationKind::Software`.
    pub fn sign_quote(
        &self,
        binary_sha256: &str,
        env_hash: &str,
        dependency_hashes: &BTreeMap<String, String>,
        custom_timestamp: Option<&str>,
    ) -> Result<AttestationQuote, AttestationError> {
        let timestamp = match custom_timestamp {
            Some(ts) => ts.to_string(),
            None => current_timestamp_iso8601(),
        };

        let pubkey_hex = self.public_key_hex();
        let payload = build_canonical_signing_payload(
            &AttestationKind::Software,
            binary_sha256,
            env_hash,
            dependency_hashes,
            &timestamp,
            &pubkey_hex,
        );

        let signature = self.signing_key.sign(&payload);
        let signature_hex = hex_encode(&signature.to_bytes());

        Ok(AttestationQuote {
            kind: AttestationKind::Software,
            binary_sha256: binary_sha256.to_string(),
            env_hash: env_hash.to_string(),
            dependency_hashes: dependency_hashes.clone(),
            timestamp,
            public_key: pubkey_hex,
            signature: signature_hex,
            tpm_evidence: None,
        })
    }
}

pub fn current_timestamp_iso8601() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Deterministic ISO-8601 formatting without external chrono dependency
    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    // Approximate civil year/month/day calculation
    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn civil_from_days(n: i64) -> (i64, u32, u32) {
    let n = n + 719468;
    let era = if n >= 0 { n } else { n - 146096 } / 146097;
    let doe = (n - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

pub fn hex_decode(hex_str: &str) -> Result<Vec<u8>, String> {
    let trimmed = hex_str.trim();
    if trimmed.len() % 2 != 0 {
        return Err("Odd hex length".to_string());
    }
    let mut bytes = Vec::with_capacity(trimmed.len() / 2);
    for i in (0..trimmed.len()).step_by(2) {
        let byte_str = &trimmed[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .map_err(|e| format!("Invalid hex byte '{}': {}", byte_str, e))?;
        bytes.push(byte);
    }
    Ok(bytes)
}
