use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Canonical relative path for project-local secret signing key.
pub const SECRET_KEY_REL_PATH: &str = ".agents/secret.key";

/// Compute HMAC-SHA256 according to RFC 2104.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let block_size = 64;
    let mut key_block = [0u8; 64];

    if key.len() > block_size {
        let mut hasher = Sha256::new();
        hasher.update(key);
        let hash = hasher.finalize();
        key_block[..32].copy_from_slice(&hash);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0u8; 64];
    let mut opad = [0u8; 64];
    for i in 0..64 {
        ipad[i] = key_block[i] ^ 0x36;
        opad[i] = key_block[i] ^ 0x5c;
    }

    // Inner hash: H((K' ^ ipad) || data)
    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&ipad);
    inner_hasher.update(data);
    let inner_hash = inner_hasher.finalize();

    // Outer hash: H((K' ^ opad) || inner_hash)
    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&opad);
    outer_hasher.update(&inner_hash);
    let outer_hash = outer_hasher.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(&outer_hash);
    result
}

/// Compute HMAC-SHA256 returning a formatted signature string.
pub fn compute_signature_string(key: &[u8], canonical_payload: &[u8]) -> String {
    let digest = hmac_sha256(key, canonical_payload);
    format!("hmac-sha256:{}", hex_encode(&digest))
}

/// Encode bytes into lowercase hexadecimal string.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Get or initialize the project-local secret key at `.agents/secret.key`.
pub fn get_or_create_project_key(base_dir: &Path) -> Result<Vec<u8>, std::io::Error> {
    let key_path = base_dir.join(SECRET_KEY_REL_PATH);
    if key_path.exists() {
        let content = fs::read_to_string(&key_path)?;
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.as_bytes().to_vec());
        }
    }

    // Create directory if needed
    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Generate a deterministic 256-bit key from system entropy / timestamp & path
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = format!(
        "endc-agent-secret-{}-{:?}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos(),
        key_path
    );
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let key_hex = format!("{:x}", hasher.finalize());

    fs::write(&key_path, format!("{}\n", key_hex))?;

    // Also ensure .agents/.gitignore ignores secret.key
    let gitignore_path = base_dir.join(".agents/.gitignore");
    if !gitignore_path.exists() {
        let _ = fs::write(&gitignore_path, "secret.key\n");
    }

    Ok(key_hex.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc4231_hmac_sha256_case_1() {
        // Test Vector 1 from RFC 4231
        let key = [0x0b; 20];
        let data = b"Hi There";
        let expected = "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7";
        let sig = compute_signature_string(&key, data);
        assert_eq!(sig, format!("hmac-sha256:{}", expected));
    }

    #[test]
    fn test_rfc4231_hmac_sha256_case_2() {
        // Test Vector 2 from RFC 4231
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let expected = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843";
        let sig = compute_signature_string(key, data);
        assert_eq!(sig, format!("hmac-sha256:{}", expected));
    }
}
