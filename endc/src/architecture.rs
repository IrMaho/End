use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchitectureConfig {
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub strict_mode: Option<bool>,
    #[serde(default)]
    pub max_layer_coupling: Option<usize>,
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
    pub error_code: String, // E0902
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
    pub compliance_score_pct: u32,
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
                let mut config: ArchitectureConfig = toml::from_str(&content)
                    .map_err(|e| format!("Failed to parse architecture config {:?}: {}", path, e))?;
                
                Self::apply_preset_defaults(&mut config);
                return Ok(config);
            }
        }

        let mut default_cfg = ArchitectureConfig::default();
        default_cfg.preset = Some("clean_hexagonal".to_string());
        Self::apply_preset_defaults(&mut default_cfg);
        Ok(default_cfg)
    }

    fn apply_preset_defaults(config: &mut ArchitectureConfig) {
        if let Some(ref preset) = config.preset {
            match preset.as_str() {
                "clean_hexagonal" | "clean" => {
                    config.invariants.entry("**/domain/**".to_string()).or_insert_with(|| RuleInvariant {
                        cannot_import: vec!["**/infrastructure/**".to_string(), "**/controllers/**".to_string(), "std/db/**".to_string()],
                        allowed_effects: None,
                        pure_math_only: None,
                        max_latency_ns: None,
                    });
                    config.invariants.entry("**/controllers/**".to_string()).or_insert_with(|| RuleInvariant {
                        cannot_import: vec!["std/db/**".to_string()],
                        allowed_effects: None,
                        pure_math_only: None,
                        max_latency_ns: None,
                    });
                }
                "game_ecs" | "ecs" => {
                    config.invariants.entry("**/components/**".to_string()).or_insert_with(|| RuleInvariant {
                        cannot_import: vec!["**/systems/**".to_string()],
                        allowed_effects: None,
                        pure_math_only: None,
                        max_latency_ns: None,
                    });
                }
                "event_driven_microservice" => {
                    config.invariants.entry("**/events/**".to_string()).or_insert_with(|| RuleInvariant {
                        cannot_import: vec!["**/handlers/**".to_string(), "**/services/**".to_string()],
                        allowed_effects: None,
                        pure_math_only: None,
                        max_latency_ns: None,
                    });
                }
                _ => {}
            }
        }
    }

    pub fn check_project(
        config: &ArchitectureConfig,
        root_dir: &Path,
    ) -> Result<ArchitectureCheckReport, String> {
        let mut files_scanned = 0;
        let mut violations = Vec::new();

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

        let score = if violations.is_empty() { 100 } else { (100usize.saturating_sub(violations.len() * 15)).max(20) as u32 };
        let status = if violations.is_empty() { "passed".to_string() } else { "violation_detected".to_string() };

        Ok(ArchitectureCheckReport {
            status,
            compliance_score_pct: score,
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
                            error_code: "E0902".to_string(),
                            rule_pattern: pattern.to_string(),
                            file: rel_path.to_string(),
                            line: idx + 1,
                            violation_type: "ArchitectureViolation (Anti-Spaghetti Block)".to_string(),
                            message: format!(
                                "Layer '{}' is strictly prohibited from accessing '{}'",
                                pattern, import_target
                            ),
                            suggested_alternative: format!(
                                "Use the Repository / UseCase pattern: inject an inverted interface via 'std/patterns/repository.end' instead of direct access to '{}'.",
                                import_target
                            ),
                        });
                    }
                }
            }
        }
    }

    pub fn scaffold_feature(feature_name: &str, preset: &str, base_dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut created = Vec::new();
        let feat_dir = base_dir.join("src/features").join(feature_name);

        let domain_entities = feat_dir.join("domain/entities");
        let domain_repos = feat_dir.join("domain/repositories");
        let usecases = feat_dir.join("usecases");
        let infra = feat_dir.join("infrastructure");

        fs::create_dir_all(&domain_entities).map_err(|e| e.to_string())?;
        fs::create_dir_all(&domain_repos).map_err(|e| e.to_string())?;
        fs::create_dir_all(&usecases).map_err(|e| e.to_string())?;
        fs::create_dir_all(&infra).map_err(|e| e.to_string())?;

        // 1. Entity
        let entity_file = domain_entities.join(format!("{}.end", feature_name));
        let entity_code = format!(
            "// Pure Domain Entity: {}\npub st {} {{\n    id: i64,\n    name: str,\n    is_active: bool,\n}}\n",
            capitalize(feature_name), capitalize(feature_name)
        );
        fs::write(&entity_file, entity_code).map_err(|e| e.to_string())?;
        created.push(entity_file);

        // 2. Repository Interface
        let repo_file = domain_repos.join(format!("{}_repo.end", feature_name));
        let repo_code = format!(
            "import \"std/patterns/repository.end\"\n\npub st {}Repository {{\n    repo: Repository<{}, i64>,\n}}\n",
            capitalize(feature_name), capitalize(feature_name)
        );
        fs::write(&repo_file, repo_code).map_err(|e| e.to_string())?;
        created.push(repo_file);

        // 3. UseCase
        let uc_file = usecases.join(format!("{}_usecase.end", feature_name));
        let uc_code = format!(
            "// Business Logic Command UseCase for {}\npub fn execute_{}_flow(id: i64) bool {{\n    ret id > 0\n}}\n",
            feature_name, feature_name
        );
        fs::write(&uc_file, uc_code).map_err(|e| e.to_string())?;
        created.push(uc_file);

        // 4. DB Repo implementation
        let db_repo_file = infra.join(format!("db_{}_repository.end", feature_name));
        let db_repo_code = format!(
            "import \"std/patterns/repository.end\"\n\npub fn create_db_{}_repo() Repository<{}, i64> {{\n    ret repository_create<{}, i64>(\"{}\", \"id\")\n}}\n",
            feature_name, capitalize(feature_name), capitalize(feature_name), feature_name
        );
        fs::write(&db_repo_file, db_repo_code).map_err(|e| e.to_string())?;
        created.push(db_repo_file);

        // 5. Controller
        let ctrl_file = infra.join(format!("http_{}_controller.end", feature_name));
        let ctrl_code = format!(
            "import \"std/patterns/pipeline.end\"\n\npub fn handle_{}_request(req_id: i64) bool {{\n    ret req_id > 0\n}}\n",
            feature_name
        );
        fs::write(&ctrl_file, ctrl_code).map_err(|e| e.to_string())?;
        created.push(ctrl_file);

        Ok(created)
    }

    fn match_glob(pattern: &str, path: &str) -> bool {
        let pat = pattern.replace('\\', "/");
        let target = path.replace('\\', "/");

        if pat == "**" || pat == "*" {
            return true;
        }

        if pat.starts_with("**/") {
            let suffix = &pat[3..];
            return target.contains(suffix.trim_end_matches("/**").trim_end_matches("/*"));
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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
