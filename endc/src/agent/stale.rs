use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Compute SHA-256 hash of a file on disk.
pub fn compute_file_hash(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute SHA-256 hash of raw bytes.
pub fn compute_bytes_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Compute hashes for a list of file paths relative to a base directory.
pub fn compute_artifact_hashes(
    base_dir: &Path,
    rel_paths: &[String],
) -> HashMap<String, Result<String, String>> {
    let mut map = HashMap::new();
    for p in rel_paths {
        let full_path = if Path::new(p).is_absolute() {
            PathBuf::from(p)
        } else {
            base_dir.join(p)
        };

        if !full_path.exists() {
            map.insert(p.clone(), Err(format!("File does not exist: {:?}", full_path)));
        } else {
            match compute_file_hash(&full_path) {
                Ok(hash) => {
                    map.insert(p.clone(), Ok(hash));
                }
                Err(e) => {
                    map.insert(p.clone(), Err(format!("Read error for {:?}: {}", full_path, e)));
                }
            }
        }
    }
    map
}

/// Result of checking whether a verified contract has become stale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaleCheckResult {
    /// Contract is fresh; all verified hashes match disk contents.
    Fresh,
    /// Contract is stale because source files, tests, or dependencies were modified or missing.
    Stale {
        modified_files: Vec<String>,
        missing_files: Vec<String>,
        details: Vec<String>,
    },
    /// No previous artifact hashes recorded.
    Unrecorded,
}

impl StaleCheckResult {
    pub fn is_stale(&self) -> bool {
        matches!(self, StaleCheckResult::Stale { .. })
    }

    pub fn is_fresh(&self) -> bool {
        matches!(self, StaleCheckResult::Fresh)
    }
}

/// Compare recorded artifact hashes against current disk contents.
pub fn check_stale_against_disk<'a, I>(
    base_dir: &Path,
    recorded_hashes: I,
) -> StaleCheckResult
where
    I: IntoIterator<Item = (&'a String, &'a String)>,
{
    let mut modified_files = Vec::new();
    let mut missing_files = Vec::new();
    let mut details = Vec::new();
    let mut count = 0;

    for (rel_path, expected_hash) in recorded_hashes {
        count += 1;
        let full_path = if Path::new(rel_path).is_absolute() {
            PathBuf::from(rel_path)
        } else {
            base_dir.join(rel_path)
        };

        if !full_path.exists() {
            missing_files.push(rel_path.clone());
            details.push(format!("File missing from disk: {}", rel_path));
            continue;
        }

        match compute_file_hash(&full_path) {
            Ok(current_hash) => {
                if current_hash != *expected_hash {
                    modified_files.push(rel_path.clone());
                    details.push(format!(
                        "Hash mismatch for '{}': recorded {}, current {}",
                        rel_path, expected_hash, current_hash
                    ));
                }
            }
            Err(e) => {
                modified_files.push(rel_path.clone());
                details.push(format!("Failed to hash '{}': {}", rel_path, e));
            }
        }
    }

    if count == 0 {
        return StaleCheckResult::Unrecorded;
    }

    if modified_files.is_empty() && missing_files.is_empty() {
        StaleCheckResult::Fresh
    } else {
        StaleCheckResult::Stale {
            modified_files,
            missing_files,
            details,
        }
    }
}
