use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::contract::AgentContract;
use super::evidence::{
    ArtifactBundle, AssertionDetail, CoverageInfo, EnvironmentInfo, EvidenceBundle, EvidenceError,
    RepairAttempt, SuggestedFixArea, TestExecutionRecord, DEFAULT_SCHEMA_VERSION,
};
use super::lifecycle::LifecycleState;
use super::provenance::Provenance;
use super::signing::{compute_signature_string, get_or_create_project_key, hex_encode};
use super::stale::{check_stale_against_disk, compute_file_hash, StaleCheckResult};

use crate::codegen::interpreter::{Interpreter, Value};
use crate::loader::load_and_analyze;

/// Result outcome from contract and evidence verification.
#[derive(Debug, Clone)]
pub enum VerificationOutcome {
    Verified(EvidenceBundle),
    Rejected {
        evidence: EvidenceBundle,
        feedback: Option<RepairAttempt>,
        reasons: Vec<String>,
    },
    Stale {
        evidence: Option<EvidenceBundle>,
        modified_files: Vec<String>,
        missing_files: Vec<String>,
    },
    Tampered {
        file: PathBuf,
        error: EvidenceError,
    },
    IncompatibleSchema {
        file: PathBuf,
        version: String,
    },
    InvalidContract(String),
    VerificationError(String),
}

impl VerificationOutcome {
    pub fn is_verified(&self) -> bool {
        matches!(self, VerificationOutcome::Verified(_))
    }
}

pub struct ContractVerifier;

impl ContractVerifier {
    /// Perform end-to-end verification of an agent contract.
    pub fn verify(
        contract_path: &Path,
        project_root_override: Option<&Path>,
        save_evidence: bool,
    ) -> VerificationOutcome {
        let start_time = Instant::now();

        // 1. Determine Project Root
        let project_root = project_root_override.unwrap_or_else(|| {
            contract_path
                .parent()
                .and_then(|p| {
                    if p.file_name().and_then(|s| s.to_str()) == Some(".agents") {
                        p.parent()
                    } else {
                        Some(p)
                    }
                })
                .unwrap_or_else(|| Path::new("."))
        });

        // 2. Load and Validate Contract TOML
        let mut contract = match AgentContract::from_file(contract_path) {
            Ok(c) => c,
            Err(e) => return VerificationOutcome::InvalidContract(e.to_string()),
        };

        // 3. Schema & Provenance Validation
        if let Err(e) = contract.validate() {
            return VerificationOutcome::InvalidContract(e.to_string());
        }

        // 4. Load or create Project Secret Signing Key
        let secret_key = match get_or_create_project_key(project_root) {
            Ok(k) => k,
            Err(e) => {
                return VerificationOutcome::VerificationError(format!(
                    "Failed to initialize secret signing key: {}",
                    e
                ))
            }
        };

        // 5. Load prior evidence if present to preserve repair_attempts history
        let mut prior_repair_history = Vec::new();
        let evidence_file = project_root
            .join(".agents/evidence")
            .join(format!("{}.json", contract.task_id));
        if evidence_file.exists() {
            if let Ok(content) = fs::read_to_string(&evidence_file) {
                if let Ok(existing_bundle) = serde_json::from_str::<EvidenceBundle>(&content) {
                    prior_repair_history = existing_bundle.repair_attempts;
                }
            }
        } else {
            let legacy_evidence_file = project_root.join(".agents/evidence.json");
            if legacy_evidence_file.exists() {
                if let Ok(content) = fs::read_to_string(&legacy_evidence_file) {
                    if let Ok(existing_bundle) = serde_json::from_str::<EvidenceBundle>(&content) {
                        prior_repair_history = existing_bundle.repair_attempts;
                    }
                }
            }
        }

        // 6. Check Stale if contract was previously VERIFIED
        if contract.lifecycle == LifecycleState::Verified && !contract.artifact_hashes.is_empty() {
            let stale_result =
                check_stale_against_disk(project_root, &contract.artifact_hashes);
            match stale_result {
                StaleCheckResult::Fresh => {
                    // Up-to-date
                }
                StaleCheckResult::Stale {
                    modified_files,
                    missing_files,
                    ..
                } => {
                    let _ = contract.lifecycle.transition(LifecycleState::Stale);
                    contract.lifecycle = LifecycleState::Stale;
                    let _ = contract.save_to_file(contract_path);

                    return VerificationOutcome::Stale {
                        evidence: None,
                        modified_files,
                        missing_files,
                    };
                }
                StaleCheckResult::Unrecorded => {}
            }
        }

        // Transition: -> Submitted -> Verifying
        if contract.lifecycle == LifecycleState::Draft
            || contract.lifecycle == LifecycleState::Stale
            || contract.lifecycle == LifecycleState::Rejected
        {
            let _ = contract.lifecycle.transition(LifecycleState::Submitted);
            contract.lifecycle = LifecycleState::Submitted;
        }
        let _ = contract.lifecycle.transition(LifecycleState::Verifying);
        contract.lifecycle = LifecycleState::Verifying;

        let mut failure_reasons: Vec<String> = Vec::new();
        let mut test_records: Vec<TestExecutionRecord> = Vec::new();
        let mut structured_feedback: Option<RepairAttempt> = None;

        // 7. Security Boundary and Operation Scanning
        let target_files = if contract.target_files.is_empty() {
            Self::discover_source_files(project_root)
        } else {
            contract.target_files.clone()
        };

        for rel_file in &target_files {
            let abs_file = project_root.join(rel_file);
            if !abs_file.exists() {
                failure_reasons.push(format!("Target file '{}' does not exist on disk", rel_file));
                continue;
            }

            if let Ok(source_code) = fs::read_to_string(&abs_file) {
                let detected_ops = Self::scan_operations_in_source(&source_code);

                // Whitelist check
                for op in &detected_ops {
                    if !contract.allowed_operations.contains(op) {
                        failure_reasons.push(format!(
                            "Disallowed Operation: operation '{}' detected in '{}' but is not in allowed_operations whitelist {:?}",
                            op, rel_file, contract.allowed_operations
                        ));
                    }
                }

                // Boundary check
                for boundary in &contract.security_boundaries {
                    if boundary == "no_outbound_network"
                        && (detected_ops.contains(&"net_connect".to_string())
                            || detected_ops.contains(&"net_listen".to_string()))
                    {
                        failure_reasons.push(format!(
                            "Security Boundary 'no_outbound_network' violated by operations {:?} in files {:?}",
                            detected_ops, rel_file
                        ));
                    }
                    if boundary == "no_exec_subprocess"
                        && detected_ops.contains(&"exec_subprocess".to_string())
                    {
                        failure_reasons.push(format!(
                            "Security Boundary 'no_exec_subprocess' violated by operations {:?} in files {:?}",
                            detected_ops, rel_file
                        ));
                    }
                    if boundary == "no_env_access"
                        && (detected_ops.contains(&"env_read".to_string())
                            || detected_ops.contains(&"env_write".to_string()))
                    {
                        failure_reasons.push(format!(
                            "Security Boundary 'no_env_access' violated by operations {:?} in files {:?}",
                            detected_ops, rel_file
                        ));
                    }
                }
            }
        }

        // 8. Execute Required Tests
        let mut total_lines_covered = 0;
        let mut total_lines_counted = 0;

        for test_rel_path in &contract.required_tests {
            let test_abs_path = project_root.join(test_rel_path);
            let test_name = test_rel_path.clone();

            if !test_abs_path.exists() {
                failure_reasons.push(format!("Required test file '{}' does not exist", test_rel_path));
                test_records.push(TestExecutionRecord {
                    name: test_name.clone(),
                    pass: false,
                    duration_ms: 0,
                    stdout_hash: Self::compute_text_hash(""),
                    stderr_hash: Self::compute_text_hash("Test file not found"),
                    exit_code: 1,
                    error_message: Some(format!("Test file '{}' does not exist", test_rel_path)),
                });

                if structured_feedback.is_none() {
                    structured_feedback = Some(RepairAttempt {
                        attempt_number: prior_repair_history.len() + 1,
                        timestamp: format!("{:?}", Instant::now()),
                        failure_reason: "required_test_missing".to_string(),
                        failed_test: Some(test_name),
                        assertion: None,
                        suggested_fix_area: Some(SuggestedFixArea {
                            file: test_rel_path.clone(),
                            line_start: 1,
                            line_end: 1,
                            hint: format!("Create required test file '{}'", test_rel_path),
                        }),
                        resolved: false,
                    });
                }
                continue;
            }

            let test_start = Instant::now();
            let test_source = fs::read_to_string(&test_abs_path).unwrap_or_default();
            let line_count = test_source.lines().count();
            total_lines_counted += line_count;

            match load_and_analyze(&test_abs_path) {
                Ok((module, _)) => {
                    let mut interpreter = Interpreter::new();
                    let mut test_passed = true;
                    let mut err_msg = None;

                    // Locate and execute test functions
                    let test_fn_names: Vec<String> = module
                        .functions
                        .iter()
                        .map(|f| f.name.clone())
                        .filter(|n| n.starts_with("test_") || n == "main")
                        .collect();

                    if test_fn_names.is_empty() {
                        match interpreter.run(&module) {
                            Ok(Value::Int(code)) if code != 0 => {
                                test_passed = false;
                                err_msg = Some(format!("main returned non-zero exit code: {}", code));
                            }
                            Ok(_) => {
                                test_passed = true;
                            }
                            Err(e) => {
                                test_passed = false;
                                err_msg = Some(format!("Execution failed: {}", e));
                            }
                        }
                    } else {
                        for fn_name in &test_fn_names {
                            match interpreter.eval_named_function(&module, fn_name, Vec::new()) {
                                Ok(Value::Bool(false)) => {
                                    test_passed = false;
                                    err_msg = Some(format!(
                                        "Test function '{}' returned false",
                                        fn_name
                                    ));
                                    break;
                                }
                                Ok(Value::Int(code)) if code != 0 => {
                                    test_passed = false;
                                    err_msg = Some(format!(
                                        "Test function '{}' returned non-zero code: {}",
                                        fn_name, code
                                    ));
                                    break;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    test_passed = false;
                                    err_msg = Some(format!(
                                        "Test function '{}' threw error: {}",
                                        fn_name, e
                                    ));
                                    break;
                                }
                            }
                        }
                    }

                    let duration_ms = test_start.elapsed().as_millis() as u64;

                    if test_passed {
                        total_lines_covered += line_count;
                        test_records.push(TestExecutionRecord {
                            name: test_name.clone(),
                            pass: true,
                            duration_ms,
                            stdout_hash: Self::compute_text_hash(&format!("Test '{}' passed", test_name)),
                            stderr_hash: Self::compute_text_hash(""),
                            exit_code: 0,
                            error_message: None,
                        });
                    } else {
                        let failure_desc = err_msg
                            .clone()
                            .unwrap_or_else(|| "Test assertion failed".to_string());
                        failure_reasons.push(format!(
                            "Required test '{}' failed: {}",
                            test_rel_path, failure_desc
                        ));

                        test_records.push(TestExecutionRecord {
                            name: test_name.clone(),
                            pass: false,
                            duration_ms,
                            stdout_hash: Self::compute_text_hash(""),
                            stderr_hash: Self::compute_text_hash(&failure_desc),
                            exit_code: 1,
                            error_message: Some(failure_desc.clone()),
                        });

                        if structured_feedback.is_none() {
                            let (line_start, line_end) = Self::locate_failing_line(&test_source);
                            structured_feedback = Some(RepairAttempt {
                                attempt_number: prior_repair_history.len() + 1,
                                timestamp: format!("{:?}", Instant::now()),
                                failure_reason: "required_test_failed".to_string(),
                                failed_test: Some(test_name),
                                assertion: Some(AssertionDetail {
                                    expected: serde_json::json!(true),
                                    actual: serde_json::json!(false),
                                }),
                                suggested_fix_area: Some(SuggestedFixArea {
                                    file: test_rel_path.clone(),
                                    line_start,
                                    line_end,
                                    hint: format!(
                                        "Review logic in '{}' or corresponding implementation in target files",
                                        test_rel_path
                                    ),
                                }),
                                resolved: false,
                            });
                        }
                    }
                }
                Err(e) => {
                    let err_str = format!("Failed to parse/analyze test '{}': {}", test_rel_path, e);
                    failure_reasons.push(err_str.clone());
                    test_records.push(TestExecutionRecord {
                        name: test_name.clone(),
                        pass: false,
                        duration_ms: test_start.elapsed().as_millis() as u64,
                        stdout_hash: Self::compute_text_hash(""),
                        stderr_hash: Self::compute_text_hash(&err_str),
                        exit_code: 1,
                        error_message: Some(err_str),
                    });

                    if structured_feedback.is_none() {
                        structured_feedback = Some(RepairAttempt {
                            attempt_number: prior_repair_history.len() + 1,
                            timestamp: format!("{:?}", Instant::now()),
                            failure_reason: "test_parse_error".to_string(),
                            failed_test: Some(test_name),
                            assertion: None,
                            suggested_fix_area: Some(SuggestedFixArea {
                                file: test_rel_path.clone(),
                                line_start: 1,
                                line_end: 10,
                                hint: "Syntax or type error in test file".to_string(),
                            }),
                            resolved: false,
                        });
                    }
                }
            }
        }

        // 9. Compute Source Artifact Hashes
        let mut source_hashes: BTreeMap<String, String> = BTreeMap::new();
        let mut all_tracked_files = target_files.clone();
        for t in &contract.required_tests {
            if !all_tracked_files.contains(t) {
                all_tracked_files.push(t.clone());
            }
        }

        for rel_file in &all_tracked_files {
            let abs_file = project_root.join(rel_file);
            if abs_file.exists() {
                if let Ok(hash) = compute_file_hash(&abs_file) {
                    source_hashes.insert(rel_file.clone(), hash);
                }
            }
        }

        // Update contract artifact hashes
        contract.artifact_hashes = source_hashes.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        // 10. Deterministic Rebuild Verification
        let (rebuild_deterministic, generated_c_hashes, binary_hashes) =
            Self::verify_deterministic_rebuild(project_root, &target_files);

        // 11. Assemble Artifact Bundle
        let artifacts = ArtifactBundle {
            source_files: source_hashes,
            generated_c: generated_c_hashes,
            binaries: binary_hashes,
        };

        // 12. Coverage Info
        let coverage = CoverageInfo {
            lines_total: total_lines_counted.max(1),
            lines_covered: total_lines_covered,
            branches_total: test_records.len() * 2,
            branches_covered: test_records.iter().filter(|t| t.pass).count() * 2,
        };

        // 13. Determine Final Lifecycle State
        let final_state = if failure_reasons.is_empty() {
            LifecycleState::Verified
        } else {
            LifecycleState::Rejected
        };

        let _ = contract.lifecycle.transition(final_state);
        contract.lifecycle = final_state;

        // 14. Update Repair Attempts History
        let mut repair_attempts = prior_repair_history;
        if let Some(mut feedback) = structured_feedback.clone() {
            feedback.attempt_number = repair_attempts.len() + 1;
            repair_attempts.push(feedback);
        } else if final_state == LifecycleState::Verified && !repair_attempts.is_empty() {
            // Mark last attempt as resolved
            if let Some(last) = repair_attempts.last_mut() {
                last.resolved = true;
            }
        }

        // 15. Create Evidence Bundle
        let mut evidence = EvidenceBundle::new(&contract.task_id, final_state);
        evidence.schema_version = DEFAULT_SCHEMA_VERSION.to_string();
        evidence.tests = test_records;
        evidence.artifacts = artifacts;
        evidence.coverage = coverage;
        evidence.environment = EnvironmentInfo::default();
        evidence.rebuild_deterministic = rebuild_deterministic;
        evidence.repair_attempts = repair_attempts;

        // 16. HMAC Sign the Evidence Bundle
        let _ = evidence.sign(&secret_key);

        // 17. Persist to disk if requested
        if save_evidence {
            let _ = evidence.save_to_dir(project_root);
            let _ = contract.save_to_file(contract_path);
        }

        // 18. Return Outcome
        if final_state == LifecycleState::Verified {
            VerificationOutcome::Verified(evidence)
        } else {
            VerificationOutcome::Rejected {
                evidence,
                feedback: structured_feedback,
                reasons: failure_reasons,
            }
        }
    }

    /// Perform two real builds from source and compare artifact hashes for determinism.
    fn verify_deterministic_rebuild(
        project_root: &Path,
        target_files: &[String],
    ) -> (bool, BTreeMap<String, String>, BTreeMap<String, String>) {
        let mut generated_c_hashes = BTreeMap::new();
        let mut binary_hashes = BTreeMap::new();

        if target_files.is_empty() {
            return (true, generated_c_hashes, binary_hashes);
        }

        let first_file = &target_files[0];
        let abs_source = project_root.join(first_file);
        if !abs_source.exists() {
            return (false, generated_c_hashes, binary_hashes);
        }

        // Perform Build #1: Generate C in memory / write
        let build1_c = match load_and_analyze(&abs_source) {
            Ok((ast, _)) => {
                let mut backend = crate::codegen::c_backend::CBackend::new();
                let (c_code, _) = backend.generate_with_options(&ast, false);
                c_code
            }
            Err(_) => return (false, generated_c_hashes, binary_hashes),
        };
        let hash_c1 = Self::compute_text_hash(&build1_c);

        // Perform Build #2: Clean/re-analyze and generate C again
        let build2_c = match load_and_analyze(&abs_source) {
            Ok((ast, _)) => {
                let mut backend = crate::codegen::c_backend::CBackend::new();
                let (c_code, _) = backend.generate_with_options(&ast, false);
                c_code
            }
            Err(_) => return (false, generated_c_hashes, binary_hashes),
        };
        let hash_c2 = Self::compute_text_hash(&build2_c);

        let deterministic_c = hash_c1 == hash_c2;
        let c_rel_name = format!("{}.c", Path::new(first_file).file_stem().unwrap_or_default().to_string_lossy());
        generated_c_hashes.insert(c_rel_name, hash_c1);

        // Check native binary hashes if binary exists on disk
        let bin_path = abs_source.with_extension(if cfg!(windows) { "exe" } else { "" });
        if bin_path.exists() {
            if let Ok(bin_hash) = compute_file_hash(&bin_path) {
                let bin_rel_name = bin_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                binary_hashes.insert(bin_rel_name, bin_hash);
            }
        }

        (deterministic_c, generated_c_hashes, binary_hashes)
    }

    /// Discover source files ending in `.end` under project root.
    fn discover_source_files(project_root: &Path) -> Vec<String> {
        let mut results = Vec::new();
        let src_dir = project_root.join("src");
        if src_dir.exists() {
            Self::collect_end_files(&src_dir, project_root, &mut results);
        }
        if results.is_empty() {
            Self::collect_end_files(project_root, project_root, &mut results);
        }
        results
    }

    fn collect_end_files(dir: &Path, base: &Path, results: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    if name != ".agents" && name != "target" && name != ".git" {
                        Self::collect_end_files(&path, base, results);
                    }
                } else if path.extension().and_then(|s| s.to_str()) == Some("end") {
                    if let Ok(rel) = path.strip_prefix(base) {
                        results.push(rel.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
        }
    }

    /// Scan operations in source text for security boundary enforcement.
    pub fn scan_operations_in_source(source: &str) -> Vec<String> {
        let mut ops = Vec::new();

        if source.contains("net_connect") || source.contains("HttpClient") || source.contains("TcpStream") || source.contains("fetch(") {
            ops.push("net_connect".to_string());
        }
        if source.contains("net_listen") || source.contains("TcpListener") || source.contains("HttpServer") {
            ops.push("net_listen".to_string());
        }
        if source.contains("exec_subprocess") || source.contains("Command::new") || source.contains("system(") {
            ops.push("exec_subprocess".to_string());
        }
        if source.contains("env_read") || source.contains("std::env::var") || source.contains("getenv") {
            ops.push("env_read".to_string());
        }
        if source.contains("env_write") || source.contains("std::env::set_var") || source.contains("setenv") {
            ops.push("env_write".to_string());
        }
        if source.contains("file_read") || source.contains("read_to_string") || source.contains("File::open") {
            ops.push("file_read".to_string());
        }
        if source.contains("file_write") || source.contains("write_to_file") || source.contains("File::create") {
            ops.push("file_write".to_string());
        }
        if source.contains("db_query") || source.contains("sql_query") || source.contains("db_exec") {
            ops.push("db_query".to_string());
        }
        if source.contains("crypto_hash") || source.contains("sha256") || source.contains("md5") {
            ops.push("crypto_hash".to_string());
        }
        if source.contains("time_read") || source.contains("now()") || source.contains("SystemTime") {
            ops.push("time_read".to_string());
        }

        ops.sort();
        ops.dedup();
        ops
    }

    fn compute_text_hash(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("sha256:{}", hex_encode(&hasher.finalize()))
    }

    fn locate_failing_line(source: &str) -> (usize, usize) {
        for (i, line) in source.lines().enumerate() {
            if line.contains("return false") || line.contains("assert") {
                return (i + 1, i + 1);
            }
        }
        (1, source.lines().count().max(1))
    }
}
