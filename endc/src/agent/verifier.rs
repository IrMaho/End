use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::ast::Module;
use crate::codegen::interpreter::{Interpreter, Value};
use crate::lexer::Lexer;
use crate::loader::load_and_analyze;
use crate::parser::Parser;

use super::contract::{AgentContract, CONTRACT_REL_PATH};
use super::evidence::{
    ContractEvidence, PostconditionCheckResult, SecurityBoundaryCheckResult, TestExecutionRecord,
    EVIDENCE_REL_PATH,
};
use super::lifecycle::LifecycleState;
use super::stale::{check_stale_against_disk, compute_file_hash, StaleCheckResult};

/// Structured outcome of running the contract verifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationOutcome {
    /// Contract passed all schema, provenance, security, test, and postcondition checks.
    Verified(ContractEvidence),
    /// Contract was rejected due to one or more check failures.
    Rejected {
        evidence: ContractEvidence,
        reasons: Vec<String>,
    },
    /// Contract was previously verified but target files or dependencies have changed.
    Stale {
        evidence: ContractEvidence,
        modified_files: Vec<String>,
        missing_files: Vec<String>,
    },
    /// Contract TOML is malformed or violates schema invariants.
    InvalidContract(String),
    /// Environmental or I/O error during verification execution.
    VerificationError(String),
}

impl VerificationOutcome {
    pub fn is_verified(&self) -> bool {
        matches!(self, VerificationOutcome::Verified(_))
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            VerificationOutcome::Verified(_) => 0,
            VerificationOutcome::Rejected { .. } => 1,
            VerificationOutcome::Stale { .. } => 2,
            VerificationOutcome::InvalidContract(_) => 3,
            VerificationOutcome::VerificationError(_) => 4,
        }
    }

    pub fn lifecycle_state(&self) -> LifecycleState {
        match self {
            VerificationOutcome::Verified(ev) => ev.lifecycle_state,
            VerificationOutcome::Rejected { evidence, .. } => evidence.lifecycle_state,
            VerificationOutcome::Stale { evidence, .. } => evidence.lifecycle_state,
            VerificationOutcome::InvalidContract(_) => LifecycleState::Draft,
            VerificationOutcome::VerificationError(_) => LifecycleState::Draft,
        }
    }
}

/// Orchestrator for verifying Agent Contracts against project sources and test suites.
pub struct ContractVerifier;

impl ContractVerifier {
    /// Verify the contract at `contract_path` for the project rooted at `project_root`.
    pub fn verify(
        contract_path: &Path,
        project_root: Option<&Path>,
        update_disk: bool,
    ) -> VerificationOutcome {
        let base_dir = project_root.unwrap_or_else(|| {
            contract_path
                .parent()
                .and_then(|p| if p.file_name().and_then(|s| s.to_str()) == Some(".agents") { p.parent() } else { Some(p) })
                .unwrap_or_else(|| Path::new("."))
        });

        // 1. Load and parse contract
        let mut contract = match AgentContract::from_file(contract_path) {
            Ok(c) => c,
            Err(e) => return VerificationOutcome::InvalidContract(e.to_string()),
        };

        // 2. Validate lifecycle transition
        let mut state = contract.lifecycle;

        // If previously verified, perform stale check first
        if state == LifecycleState::Verified && !contract.artifact_hashes.is_empty() {
            let stale_check = check_stale_against_disk(base_dir, &contract.artifact_hashes);
            if let StaleCheckResult::Stale {
                modified_files,
                missing_files,
                details,
            } = stale_check
            {
                let _ = state.transition(LifecycleState::Stale);
                contract.lifecycle = LifecycleState::Stale;
                if update_disk {
                    let _ = contract.save_to_file(contract_path);
                }

                let mut evidence =
                    ContractEvidence::new(contract.task_id.clone(), contract.intent.clone(), contract.provenance.clone());
                evidence.lifecycle_state = LifecycleState::Stale;
                evidence.failure_reasons = details;

                return VerificationOutcome::Stale {
                    evidence,
                    modified_files,
                    missing_files,
                };
            }
        }

        // Advance lifecycle to SUBMITTED and then VERIFYING
        if state == LifecycleState::Draft {
            if let Err(e) = state.transition(LifecycleState::Submitted) {
                return VerificationOutcome::VerificationError(e.to_string());
            }
        } else if state == LifecycleState::Stale || state == LifecycleState::Rejected {
            if let Err(e) = state.transition(LifecycleState::Submitted) {
                return VerificationOutcome::VerificationError(e.to_string());
            }
        }

        if state == LifecycleState::Submitted {
            if let Err(e) = state.transition(LifecycleState::Verifying) {
                return VerificationOutcome::VerificationError(e.to_string());
            }
        }

        contract.lifecycle = LifecycleState::Verifying;

        let mut evidence =
            ContractEvidence::new(contract.task_id.clone(), contract.intent.clone(), contract.provenance.clone());
        evidence.lifecycle_state = LifecycleState::Verifying;

        // 3. Provenance validation
        if let Err(prov_err) = contract.provenance.validate() {
            evidence.failure_reasons.push(format!("Provenance validation failed: {}", prov_err));
        }

        // 4. Discover and scan target files for security boundary & allowed operations
        let target_files = if !contract.target_files.is_empty() {
            contract.target_files.clone()
        } else {
            discover_project_end_files(base_dir)
        };

        let mut detected_ops_by_file: HashMap<String, HashSet<String>> = HashMap::new();
        for tf in &target_files {
            let full_target = if Path::new(tf).is_absolute() {
                PathBuf::from(tf)
            } else {
                base_dir.join(tf)
            };

            if full_target.exists() {
                if let Ok(source) = fs::read_to_string(&full_target) {
                    let ops = scan_operations_in_source(&source);
                    detected_ops_by_file.insert(tf.clone(), ops);
                }
            }
        }

        // Check Allowed Operations
        if !contract.allowed_operations.is_empty() {
            let allowed_set: HashSet<String> = contract.allowed_operations.iter().cloned().collect();
            for (file, ops) in &detected_ops_by_file {
                for op in ops {
                    if !allowed_set.contains(op) {
                        let reason = format!(
                            "Disallowed Operation: operation '{}' detected in '{}' but is not in allowed_operations whitelist {:?}",
                            op, file, contract.allowed_operations
                        );
                        evidence.failure_reasons.push(reason);
                    }
                }
            }
        }

        // Check Security Boundaries
        for boundary in &contract.security_boundaries {
            let mut violated = false;
            let mut detected_violations = Vec::new();
            let mut locs = Vec::new();

            for (file, ops) in &detected_ops_by_file {
                match boundary.as_str() {
                    "no_outbound_network" => {
                        if ops.contains("net_connect") {
                            violated = true;
                            detected_violations.push("net_connect".to_string());
                            locs.push(file.clone());
                        }
                    }
                    "no_inbound_network" => {
                        if ops.contains("net_listen") {
                            violated = true;
                            detected_violations.push("net_listen".to_string());
                            locs.push(file.clone());
                        }
                    }
                    "no_exec_subprocess" => {
                        if ops.contains("exec_subprocess") {
                            violated = true;
                            detected_violations.push("exec_subprocess".to_string());
                            locs.push(file.clone());
                        }
                    }
                    "no_env_access" => {
                        if ops.contains("env_read") || ops.contains("env_write") {
                            violated = true;
                            detected_violations.push("env_access".to_string());
                            locs.push(file.clone());
                        }
                    }
                    "no_file_write" => {
                        if ops.contains("file_write") {
                            violated = true;
                            detected_violations.push("file_write".to_string());
                            locs.push(file.clone());
                        }
                    }
                    "pure_computation" => {
                        let io_ops = ["net_connect", "net_listen", "exec_subprocess", "env_read", "env_write", "file_read", "file_write", "db_query"];
                        for io in &io_ops {
                            if ops.contains(*io) {
                                violated = true;
                                detected_violations.push(io.to_string());
                                locs.push(file.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }

            if violated {
                let diagnostic = format!(
                    "Security Boundary '{}' violated by operations {:?} in files {:?}",
                    boundary, detected_violations, locs
                );
                evidence.failure_reasons.push(diagnostic.clone());
                evidence.security_boundary_checks.push(SecurityBoundaryCheckResult {
                    boundary: boundary.clone(),
                    satisfied: false,
                    detected_operations: detected_violations,
                    violating_locations: locs,
                    diagnostic,
                });
            } else {
                evidence.security_boundary_checks.push(SecurityBoundaryCheckResult {
                    boundary: boundary.clone(),
                    satisfied: true,
                    detected_operations: Vec::new(),
                    violating_locations: Vec::new(),
                    diagnostic: format!("Security Boundary '{}' verified.", boundary),
                });
            }
        }

        // 5. Execute Required Tests
        let total_suite_start = Instant::now();

        if contract.required_tests.is_empty() {
            evidence.failure_reasons.push("Contract requires at least one test in 'required_tests' to achieve VERIFIED state".to_string());
        }

        for test_rel in &contract.required_tests {
            let full_test_path = if Path::new(test_rel).is_absolute() {
                PathBuf::from(test_rel)
            } else {
                base_dir.join(test_rel)
            };

            evidence.total_tests += 1;

            if !full_test_path.exists() {
                evidence.failed_tests += 1;
                let err_msg = format!("Required test file not found: {:?}", full_test_path);
                evidence.failure_reasons.push(err_msg.clone());
                evidence.tests_executed.push(TestExecutionRecord {
                    test_name: test_rel.clone(),
                    path: full_test_path.to_string_lossy().to_string(),
                    passed: false,
                    duration_ms: 0,
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: err_msg.clone(),
                    error_message: Some(err_msg),
                });
                continue;
            }

            // Real execution of required test
            let test_record = execute_test_file(&full_test_path, test_rel);
            if test_record.passed {
                evidence.passed_tests += 1;
            } else {
                evidence.failed_tests += 1;
                let reason = format!(
                    "Required test '{}' failed: {}",
                    test_rel,
                    test_record.error_message.as_deref().unwrap_or("Execution failure")
                );
                evidence.failure_reasons.push(reason);
            }
            evidence.tests_executed.push(test_record);
        }

        evidence.total_duration_ms = total_suite_start.elapsed().as_millis() as u64;

        // 6. Artifact Hashing
        let mut computed_hashes = HashMap::new();
        for tf in &target_files {
            let full_p = if Path::new(tf).is_absolute() {
                PathBuf::from(tf)
            } else {
                base_dir.join(tf)
            };
            if full_p.exists() {
                if let Ok(h) = compute_file_hash(&full_p) {
                    computed_hashes.insert(tf.clone(), h);
                }
            }
        }
        for tr in &contract.required_tests {
            let full_p = if Path::new(tr).is_absolute() {
                PathBuf::from(tr)
            } else {
                base_dir.join(tr)
            };
            if full_p.exists() {
                if let Ok(h) = compute_file_hash(&full_p) {
                    computed_hashes.insert(tr.clone(), h);
                }
            }
        }
        evidence.artifact_hashes = computed_hashes.clone();

        // 7. Postcondition verification
        for post in &contract.postconditions {
            // Postconditions are considered satisfied when all required tests pass and no security violations occurred
            let satisfied = evidence.failed_tests == 0 && evidence.failure_reasons.is_empty();
            evidence.postcondition_checks.push(PostconditionCheckResult {
                description: post.clone(),
                satisfied,
                details: if satisfied {
                    format!("Postcondition '{}' satisfied by passing verification suite", post)
                } else {
                    format!("Postcondition '{}' unsatisfied due to test or security failures", post)
                },
            });
        }

        // 8. Determine Final State
        if evidence.failure_reasons.is_empty() && evidence.failed_tests == 0 && evidence.passed_tests > 0 {
            let _ = state.transition(LifecycleState::Verified);
            contract.lifecycle = LifecycleState::Verified;
            contract.artifact_hashes = computed_hashes;
            evidence.lifecycle_state = LifecycleState::Verified;
            evidence.verified = true;

            if update_disk {
                let _ = contract.save_to_file(contract_path);
                let evidence_path = base_dir.join(EVIDENCE_REL_PATH);
                let _ = evidence.save_to_file(&evidence_path);
            }

            VerificationOutcome::Verified(evidence)
        } else {
            let _ = state.transition(LifecycleState::Rejected);
            contract.lifecycle = LifecycleState::Rejected;
            evidence.lifecycle_state = LifecycleState::Rejected;
            evidence.verified = false;

            if update_disk {
                let _ = contract.save_to_file(contract_path);
                let evidence_path = base_dir.join(EVIDENCE_REL_PATH);
                let _ = evidence.save_to_file(&evidence_path);
            }

            let reasons = evidence.failure_reasons.clone();
            VerificationOutcome::Rejected { evidence, reasons }
        }
    }
}

/// Execute an individual `.end` test file, measuring execution duration and verifying correctness.
fn execute_test_file(path: &Path, test_name: &str) -> TestExecutionRecord {
    let start = Instant::now();
    let path_str = path.to_string_lossy().to_string();

    let path_buf = PathBuf::from(path);
    let (module, _) = match load_and_analyze(&path_buf) {
        Ok(res) => res,
        Err(e) => {
            let dur = start.elapsed().as_millis() as u64;
            return TestExecutionRecord {
                test_name: test_name.to_string(),
                path: path_str,
                passed: false,
                duration_ms: dur,
                exit_code: 1,
                stdout: String::new(),
                stderr: format!("Compiler error during test analysis: {}", e),
                error_message: Some(format!("Analysis failure: {}", e)),
            };
        }
    };

    let mut vm = Interpreter::new();
    let mut passed = true;
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut err_msg = None;

    // Discover test functions or main
    let mut executed_any = false;

    for func in &module.functions {
        let is_test_attr = func.directives.iter().any(|d| {
            d.name == "@test" || d.name == "@scenario" || d.name == "@bench" || d.name == "@patrol"
        });
        let is_test_name = func.name.starts_with("test_")
            || func.name.starts_with("bench_")
            || func.name.starts_with("patrol_");

        if is_test_attr || is_test_name {
            executed_any = true;
            match vm.eval_named_function(&module, &func.name, vec![]) {
                Ok(val) => match val {
                    Value::Bool(b) => {
                        if !b {
                            passed = false;
                            err_msg = Some(format!("Test function '{}' returned false", func.name));
                        }
                    }
                    Value::Int(n) => {
                        if n != 0 {
                            passed = false;
                            err_msg = Some(format!(
                                "Test function '{}' returned non-zero exit code: {}",
                                func.name, n
                            ));
                        }
                    }
                    Value::Void => {}
                    _ => {}
                },
                Err(err) => {
                    passed = false;
                    err_msg = Some(format!("Test function '{}' panic: {}", func.name, err));
                }
            }
        }
    }

    if !executed_any {
        // Evaluate whole module / main
        match vm.run(&module) {
            Ok(val) => match val {
                Value::Bool(b) => {
                    if !b {
                        passed = false;
                        err_msg = Some("Module main returned false".to_string());
                    }
                }
                Value::Int(n) => {
                    if n != 0 {
                        passed = false;
                        err_msg = Some(format!("Module main returned non-zero code: {}", n));
                    }
                }
                Value::Void => {}
                _ => {}
            },
            Err(err) => {
                passed = false;
                err_msg = Some(format!("Module execution panic: {}", err));
            }
        }
    }

    let dur = start.elapsed().as_millis() as u64;

    if passed {
        stdout.push_str(&format!("Test '{}' passed in {} ms", test_name, dur));
    } else if let Some(ref m) = err_msg {
        stderr.push_str(m);
    }

    TestExecutionRecord {
        test_name: test_name.to_string(),
        path: path_str,
        passed,
        duration_ms: dur,
        exit_code: if passed { 0 } else { 1 },
        stdout,
        stderr,
        error_message: err_msg,
    }
}

/// Helper to scan source code for operation keywords and signatures.
pub fn scan_operations_in_source(source: &str) -> HashSet<String> {
    let mut ops = HashSet::new();

    // Check lexer tokens or keywords in source
    let source_lower = source.to_lowercase();

    // Network Connect
    if source_lower.contains("net_connect")
        || source_lower.contains("net::connect")
        || source_lower.contains("tcpstream::connect")
        || source_lower.contains("httpclient")
        || source_lower.contains("fetch(")
        || source_lower.contains("curl(")
        || source_lower.contains("http_get")
        || source_lower.contains("http_post")
        || source_lower.contains("socket_connect")
        || source_lower.contains("net_outbound")
    {
        ops.insert("net_connect".to_string());
    }

    // Network Listen
    if source_lower.contains("net_listen")
        || source_lower.contains("net::listen")
        || source_lower.contains("tcplistener::bind")
        || source_lower.contains("listen_tcp")
        || source_lower.contains("http_server")
        || source_lower.contains("server::bind")
    {
        ops.insert("net_listen".to_string());
    }

    // Subprocess Exec
    if source_lower.contains("exec_subprocess")
        || source_lower.contains("command::new")
        || source_lower.contains("exec_cmd")
        || source_lower.contains("spawn_process")
        || source_lower.contains("system(")
        || source_lower.contains("popen(")
    {
        ops.insert("exec_subprocess".to_string());
    }

    // Environment Read
    if source_lower.contains("env_read")
        || source_lower.contains("std::env::var")
        || source_lower.contains("getenv")
        || source_lower.contains("env::get")
        || source_lower.contains("read_env")
    {
        ops.insert("env_read".to_string());
    }

    // Environment Write
    if source_lower.contains("env_write")
        || source_lower.contains("std::env::set_var")
        || source_lower.contains("setenv")
        || source_lower.contains("env::set")
        || source_lower.contains("write_env")
    {
        ops.insert("env_write".to_string());
    }

    // File Read
    if source_lower.contains("file_read")
        || source_lower.contains("read_to_string")
        || source_lower.contains("file::open")
        || source_lower.contains("fs::read")
        || source_lower.contains("read_file")
    {
        ops.insert("file_read".to_string());
    }

    // File Write
    if source_lower.contains("file_write")
        || source_lower.contains("write_to_file")
        || source_lower.contains("file::create")
        || source_lower.contains("fs::write")
        || source_lower.contains("write_file")
    {
        ops.insert("file_write".to_string());
    }

    // Database Query
    if source_lower.contains("db_query")
        || source_lower.contains("db::execute")
        || source_lower.contains("db_execute")
        || source_lower.contains("sql_query")
        || source_lower.contains("dbtable")
    {
        ops.insert("db_query".to_string());
    }

    // Crypto Hash
    if source_lower.contains("crypto_hash")
        || source_lower.contains("sha256(")
        || source_lower.contains("sha512(")
        || source_lower.contains("md5(")
        || source_lower.contains("hash_bytes")
    {
        ops.insert("crypto_hash".to_string());
    }

    // Crypto Sign
    if source_lower.contains("crypto_sign")
        || source_lower.contains("sign_message")
        || source_lower.contains("verify_signature")
        || source_lower.contains("rsa_sign")
        || source_lower.contains("ed25519_sign")
    {
        ops.insert("crypto_sign".to_string());
    }

    // Time Read
    if source_lower.contains("time_read")
        || source_lower.contains("instant::now")
        || source_lower.contains("systemtime::now")
        || source_lower.contains("time_now")
        || source_lower.contains("get_time")
    {
        ops.insert("time_read".to_string());
    }

    ops
}

fn discover_project_end_files(dir: &Path) -> Vec<String> {
    let mut res = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("end") {
                if let Ok(rel) = p.strip_prefix(dir) {
                    res.push(rel.to_string_lossy().replace('\\', "/"));
                }
            } else if p.is_dir() && p.file_name().and_then(|s| s.to_str()) == Some("src") {
                if let Ok(src_entries) = fs::read_dir(&p) {
                    for se in src_entries.flatten() {
                        let sp = se.path();
                        if sp.is_file() && sp.extension().and_then(|s| s.to_str()) == Some("end") {
                            if let Ok(rel) = sp.strip_prefix(dir) {
                                res.push(rel.to_string_lossy().replace('\\', "/"));
                            }
                        }
                    }
                }
            }
        }
    }
    res
}
