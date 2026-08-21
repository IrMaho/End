use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchitectureConfig {
    #[serde(default)]
    pub invariants: HashMap<String, RuleInvariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleInvariant {
    #[serde(default)]
    pub cannot_import: Vec<String>,
    #[serde(default)]
    pub allowed_effects: Option<Vec<String>>,
    #[serde(default)]
    pub pure_math_only: Option<bool>,
    #[serde(default)]
    pub max_latency_ns: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureViolation {
    pub rule_pattern: String,
    pub file: String,
    pub line: usize,
    pub violation_type: String,
    pub message: String,
    pub suggested_alternative: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureCheckReport {
    pub status: String,
    pub rules_checked: usize,
    pub files_scanned: usize,
    pub violations_count: usize,
    pub violations: Vec<ArchitectureViolation>,
}

pub struct ArchitectureEngine;

impl ArchitectureEngine {
    pub fn load_config(config_path: Option<&Path>) -> Result<ArchitectureConfig, String> {
        let candidate_paths = if let Some(p) = config_path {
            vec![p.to_path_buf()]
        } else {
            vec![
                PathBuf::from("Architecture.toml"),
                PathBuf::from("architecture.toml"),
                PathBuf::from("end.toml"),
            ]
        };

        for path in candidate_paths {
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read architecture config {:?}: {}", path, e))?;
                let config: ArchitectureConfig = toml::from_str(&content)
                    .map_err(|e| format!("Failed to parse architecture config {:?}: {}", path, e))?;
                return Ok(config);
            }
        }

        // Return default empty config if no file found
        Ok(ArchitectureConfig::default())
    }

    pub fn check_project(
        config: &ArchitectureConfig,
        root_dir: &Path,
    ) -> Result<ArchitectureCheckReport, String> {
        let mut files_scanned = 0;
        let mut violations = Vec::new();

        if config.invariants.is_empty() {
            return Ok(ArchitectureCheckReport {
                status: "passed".to_string(),
                rules_checked: 0,
                files_scanned: 0,
                violations_count: 0,
                violations: Vec::new(),
            });
        }

        let mut end_files = Vec::new();
        Self::collect_end_files(root_dir, &mut end_files);

        for file_path in &end_files {
            files_scanned += 1;
            let rel_path_str = file_path
                .strip_prefix(root_dir)
                .unwrap_or(file_path)
                .to_string_lossy()
                .replace('\\', "/");

            for (pattern, rule) in &config.invariants {
                if Self::match_glob(pattern, &rel_path_str) {
                    Self::check_file_against_rule(file_path, &rel_path_str, pattern, rule, &mut violations);
                }
            }
        }

        let status = if violations.is_empty() {
            "passed".to_string()
        } else {
            "violation_detected".to_string()
        };

        Ok(ArchitectureCheckReport {
            status,
            rules_checked: config.invariants.len(),
            files_scanned,
            violations_count: violations.len(),
            violations,
        })
    }

    fn check_file_against_rule(
        file_path: &Path,
        rel_path: &str,
        pattern: &str,
        rule: &RuleInvariant,
        violations: &mut Vec<ArchitectureViolation>,
    ) {
        let source = match fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(_) => return,
        };

        let lines: Vec<&str> = source.lines().collect();

        // 1. Check imports against `cannot_import`
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                let import_target = trimmed
                    .trim_start_matches("import ")
                    .trim_matches(|c| c == '"' || c == ';' || c == ' ')
                    .replace('\\', "/");

                for forbidden in &rule.cannot_import {
                    let forb_norm = forbidden.replace('\\', "/");
                    if Self::match_glob(&forb_norm, &import_target) || import_target.contains(&forb_norm.trim_end_matches("/**").trim_end_matches("/*")) {
                        violations.push(ArchitectureViolation {
                            rule_pattern: pattern.to_string(),
                            file: rel_path.to_string(),
                            line: idx + 1,
                            violation_type: "ForbiddenImport".to_string(),
                            message: format!(
                                "Layer '{}' is strictly prohibited from importing '{}'",
                                pattern, import_target
                            ),
                            suggested_alternative: format!(
                                "Access '{}' through an inverted interface, service contract, or DTO adapter layer instead of direct import.",
                                import_target
                            ),
                        });
                    }
                }
            }
        }

        // 2. Check pure_math_only rule
        if rule.pure_math_only == Some(true) {
            for (idx, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.contains("@ws") || trimmed.contains("@post") || trimmed.contains("@get") || trimmed.contains("safe_socket") {
                    violations.push(ArchitectureViolation {
                        rule_pattern: pattern.to_string(),
                        file: rel_path.to_string(),
                        line: idx + 1,
                        violation_type: "ImpureEffectInPureMathLayer".to_string(),
                        message: format!(
                            "File in pure math module '{}' performs I/O or network operations",
                            pattern
                        ),
                        suggested_alternative: "Move I/O and socket operations to 'server/**' or 'net/**' and keep math computations 100% pure.".to_string(),
                    });
                }
            }
        }
    }

    fn match_glob(pattern: &str, path: &str) -> bool {
        let pat = pattern.replace('\\', "/");
        let target = path.replace('\\', "/");

        if pat == "**" || pat == "*" {
            return true;
        }

        if pat.ends_with("/**") {
            let prefix = &pat[..pat.len() - 3];
            return target.starts_with(prefix);
        }

        if pat.ends_with("/*") {
            let prefix = &pat[..pat.len() - 2];
            return target.starts_with(prefix);
        }

        target == pat || target.contains(&pat)
    }

    fn collect_end_files(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if name != "target" && name != "node_modules" && name != ".git" {
                        Self::collect_end_files(&path, files);
                    }
                } else if path.extension().and_then(|s| s.to_str()) == Some("end") {
                    files.push(path);
                }
            }
        }
    }
}
