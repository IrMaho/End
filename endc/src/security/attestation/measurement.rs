use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use crate::security::crypto::{sha256_digest, sha256_hex};

/// Computes a standard NIST FIPS 180-4 SHA-256 hex string over raw file bytes.
pub fn measure_file_sha256<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(sha256_hex(&bytes))
}

/// Computes SHA-256 hex string over raw memory bytes.
pub fn measure_bytes_sha256(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// Computes a deterministic canonical environment measurement.
///
/// If explicit `vars` are provided, sorts them by key and hashes `KEY=VALUE\n`.
/// Otherwise, reads current process environment variables, sorts them lexicographically,
/// and hashes their canonical serialization.
pub fn measure_environment(vars: Option<&[(&str, &str)]>) -> String {
    let mut sorted_vars: Vec<(String, String)> = match vars {
        Some(v) => v
            .iter()
            .map(|(k, val)| (k.to_string(), val.to_string()))
            .collect(),
        None => std::env::vars().collect(),
    };

    // Deterministic sorting by key
    sorted_vars.sort_by(|a, b| a.0.cmp(&b.0));

    let mut canonical_bytes = Vec::new();
    for (k, v) in sorted_vars {
        canonical_bytes.extend_from_slice(k.as_bytes());
        canonical_bytes.push(b'=');
        canonical_bytes.extend_from_slice(v.as_bytes());
        canonical_bytes.push(b'\n');
    }

    sha256_hex(&canonical_bytes)
}

/// Computes deterministic SHA-256 hashes for all specified dependency files.
///
/// Paths are canonicalized (or normalized with forward slashes) and stored in a
/// sorted `BTreeMap` to ensure independent, deterministic verification across runs.
pub fn measure_dependencies<P: AsRef<Path>>(paths: &[P]) -> io::Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    for p in paths {
        let path = p.as_ref();
        let hash = measure_file_sha256(path)?;
        let key = path
            .to_string_lossy()
            .replace('\\', "/");
        map.insert(key, hash);
    }
    Ok(map)
}

/// Combined system measurement snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemMeasurement {
    pub binary_sha256: String,
    pub env_hash: String,
    pub dependency_hashes: BTreeMap<String, String>,
}

impl SystemMeasurement {
    pub fn measure_target<P: AsRef<Path>>(
        binary_path: P,
        env_vars: Option<&[(&str, &str)]>,
        dep_paths: &[P],
    ) -> io::Result<Self> {
        let binary_sha256 = measure_file_sha256(binary_path)?;
        let env_hash = measure_environment(env_vars);
        let dependency_hashes = measure_dependencies(dep_paths)?;

        Ok(Self {
            binary_sha256,
            env_hash,
            dependency_hashes,
        })
    }

    /// Computes the overall unified measurement digest over all components.
    pub fn combined_digest(&self) -> [u8; 32] {
        let mut combined = Vec::new();
        combined.extend_from_slice(self.binary_sha256.as_bytes());
        combined.push(b':');
        combined.extend_from_slice(self.env_hash.as_bytes());
        combined.push(b':');
        for (k, v) in &self.dependency_hashes {
            combined.extend_from_slice(k.as_bytes());
            combined.push(b'=');
            combined.extend_from_slice(v.as_bytes());
            combined.push(b',');
        }
        sha256_digest(&combined)
    }

    pub fn combined_digest_hex(&self) -> String {
        sha256_hex(&self.combined_digest())
    }
}
