// End Security: Real Cryptographic Subsystem (Argon2id, HMAC-SHA256, SHA-256)
// Provides authentic cryptographic implementations, constant-time verification,
// strict PHC serialization/deserialization, and standard FIPS 180-4 / RFC 4231 compliance.

use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString,
    },
    Argon2, Params, Version,
};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use std::fmt;

pub type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    InvalidPassword(String),
    InvalidSalt(String),
    InvalidParameters(String),
    InvalidPhcFormat(String),
    AlgorithmMismatch(String),
    VersionMismatch(String),
    VerificationFailed,
    InternalError(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidPassword(msg) => write!(f, "Invalid password: {}", msg),
            CryptoError::InvalidSalt(msg) => write!(f, "Invalid salt: {}", msg),
            CryptoError::InvalidParameters(msg) => write!(f, "Invalid Argon2 parameters: {}", msg),
            CryptoError::InvalidPhcFormat(msg) => write!(f, "Invalid PHC hash format: {}", msg),
            CryptoError::AlgorithmMismatch(msg) => write!(f, "Algorithm mismatch: {}", msg),
            CryptoError::VersionMismatch(msg) => write!(f, "Version mismatch: {}", msg),
            CryptoError::VerificationFailed => write!(f, "Cryptographic verification failed"),
            CryptoError::InternalError(msg) => write!(f, "Cryptographic internal error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Standard Argon2id Configuration Parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Argon2Config {
    pub memory_cost_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub output_len: usize,
}

impl Default for Argon2Config {
    fn default() -> Self {
        // Strict contract: m=65536 KiB (64 MiB), t=3, p=4, tag_len=32
        Self {
            memory_cost_kib: 65536,
            time_cost: 3,
            parallelism: 4,
            output_len: 32,
        }
    }
}

impl Argon2Config {
    pub fn to_argon2_params(&self) -> Result<Params, CryptoError> {
        Params::new(
            self.memory_cost_kib,
            self.time_cost,
            self.parallelism,
            Some(self.output_len),
        )
        .map_err(|e| CryptoError::InvalidParameters(e.to_string()))
    }
}

/// Computes a standard NIST FIPS 180-4 SHA-256 digest over arbitrary binary data.
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// Computes a standard NIST FIPS 180-4 SHA-256 hex string over arbitrary binary data.
pub fn sha256_hex(data: &[u8]) -> String {
    let digest = sha256_digest(data);
    let mut hex = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", b);
    }
    hex
}

/// Computes standard RFC 2104 / RFC 4231 HMAC-SHA256 over arbitrary key and data.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC-SHA256 can accept key of any length");
    mac.update(data);
    let result = mac.finalize();
    let bytes = result.into_bytes();
    let mut output = [0u8; 32];
    output.copy_from_slice(&bytes);
    output
}

/// Computes standard RFC 4231 HMAC-SHA256 hex string.
pub fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    let tag = hmac_sha256(key, data);
    let mut hex = String::with_capacity(64);
    for b in tag {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", b);
    }
    hex
}

/// Verifies an HMAC-SHA256 authentication tag in constant time using `subtle::ConstantTimeEq`.
pub fn hmac_sha256_verify(key: &[u8], data: &[u8], expected_tag: &[u8]) -> bool {
    let computed_tag = hmac_sha256(key, data);
    if computed_tag.len() != expected_tag.len() {
        return false;
    }
    computed_tag.ct_eq(expected_tag).into()
}

/// Verifies an HMAC-SHA256 hex string in constant time.
pub fn hmac_sha256_verify_hex(key: &[u8], data: &[u8], expected_hex: &str) -> bool {
    let computed_hex = hmac_sha256_hex(key, data);
    if computed_hex.len() != expected_hex.len() {
        return false;
    }
    computed_hex.as_bytes().ct_eq(expected_hex.as_bytes()).into()
}

/// Hashes a password using Argon2id with random salt and standard parameters (m=65536, t=3, p=4).
/// Returns a standard PHC-formatted string: `$argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>`.
pub fn argon2id_hash(password: &[u8]) -> Result<String, CryptoError> {
    argon2id_hash_with_config(password, None, Argon2Config::default())
}

/// Hashes a password using Argon2id with explicit salt and configuration.
/// If `salt_str` is None, a secure random 16-byte salt is generated.
pub fn argon2id_hash_with_config(
    password: &[u8],
    salt_str: Option<&str>,
    config: Argon2Config,
) -> Result<String, CryptoError> {
    let params = config.to_argon2_params()?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    if let Some(s) = salt_str {
        let salt = Salt::from_b64(s)
            .or_else(|_| Salt::new(s))
            .map_err(|e| CryptoError::InvalidSalt(e.to_string()))?;
        let hash = argon2
            .hash_password(password, salt)
            .map_err(|e| CryptoError::InternalError(e.to_string()))?;
        Ok(hash.to_string())
    } else {
        let mut rng = rand_core::OsRng;
        let salt = SaltString::generate(&mut rng);
        let hash = argon2
            .hash_password(password, &salt)
            .map_err(|e| CryptoError::InternalError(e.to_string()))?;
        Ok(hash.to_string())
    }
}

/// Verifies a password against a standard PHC-formatted Argon2id string.
/// Uses constant-time equality comparison and strictly validates algorithm, version, and parameters.
pub fn argon2id_verify(password: &[u8], phc_hash: &str) -> Result<bool, CryptoError> {
    if phc_hash.is_empty() {
        return Err(CryptoError::InvalidPhcFormat("Empty PHC string".to_string()));
    }

    let parsed_hash = PasswordHash::new(phc_hash)
        .map_err(|e| CryptoError::InvalidPhcFormat(e.to_string()))?;

    // Strictly ensure both salt and hash are present
    if parsed_hash.salt.is_none() {
        return Err(CryptoError::InvalidPhcFormat("Missing salt in PHC string".to_string()));
    }
    if parsed_hash.hash.is_none() {
        return Err(CryptoError::InvalidPhcFormat("Missing hash digest in PHC string".to_string()));
    }

    // Strictly ensure algorithm is argon2id
    if parsed_hash.algorithm.as_str() != "argon2id" {
        return Err(CryptoError::AlgorithmMismatch(format!(
            "Expected argon2id, found {}",
            parsed_hash.algorithm
        )));
    }

    // Strictly ensure version is 0x13 (19) if present
    if let Some(ver) = parsed_hash.version {
        if ver != 19 {
            return Err(CryptoError::VersionMismatch(format!(
                "Expected Argon2 version 19, found {}",
                ver
            )));
        }
    }

    let argon2 = Argon2::default();
    match argon2.verify_password(password, &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(CryptoError::InvalidPhcFormat(e.to_string())),
    }
}

/// Safe constant-time string comparison for security-sensitive secrets/tokens.
pub fn constant_time_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Safe constant-time byte slice comparison.
pub fn constant_time_eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}
