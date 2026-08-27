use std::collections::BTreeMap;

use super::types::AttestationKind;

/// Builds the deterministic canonical byte sequence that represents the quote payload.
///
/// Every security-sensitive field (kind, binary_sha256, env_hash, dependency_hashes,
/// timestamp, and public_key) is explicitly encoded in a standard delimiter format.
///
/// Changing ANY field in the quote will alter this byte sequence and cause cryptographic
/// signature verification to fail immediately.
pub fn build_canonical_signing_payload(
    kind: &AttestationKind,
    binary_sha256: &str,
    env_hash: &str,
    dependency_hashes: &BTreeMap<String, String>,
    timestamp: &str,
    public_key: &str,
) -> Vec<u8> {
    let mut payload = Vec::with_capacity(512);

    payload.extend_from_slice(b"END-ATTESTATION-PAYLOAD-V1\n");
    payload.extend_from_slice(b"kind:");
    payload.extend_from_slice(kind.to_string().as_bytes());
    payload.push(b'\n');

    payload.extend_from_slice(b"binary_sha256:");
    payload.extend_from_slice(binary_sha256.as_bytes());
    payload.push(b'\n');

    payload.extend_from_slice(b"env_hash:");
    payload.extend_from_slice(env_hash.as_bytes());
    payload.push(b'\n');

    payload.extend_from_slice(b"dependencies:\n");
    // BTreeMap keys are guaranteed sorted
    for (path, hash) in dependency_hashes {
        payload.extend_from_slice(path.as_bytes());
        payload.push(b'=');
        payload.extend_from_slice(hash.as_bytes());
        payload.push(b'\n');
    }

    payload.extend_from_slice(b"timestamp:");
    payload.extend_from_slice(timestamp.as_bytes());
    payload.push(b'\n');

    payload.extend_from_slice(b"public_key:");
    payload.extend_from_slice(public_key.as_bytes());
    payload.push(b'\n');

    payload
}
