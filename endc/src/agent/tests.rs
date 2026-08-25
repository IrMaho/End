use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::contract::{AgentContract, ContractError};
use super::evidence::ContractEvidence;
use super::lifecycle::{LifecycleError, LifecycleState};
use super::provenance::Provenance;
use super::stale::{check_stale_against_disk, compute_file_hash, StaleCheckResult};
use super::verifier::{ContractVerifier, VerificationOutcome};

static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn create_temp_test_dir(prefix: &str) -> PathBuf {
    let id = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("end_contract_test_{}_{}_{}", prefix, std::process::id(), id));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("failed to create temp test directory");
    path
}

// ============================================================================
// CATEGORY A — PARSER & SCHEMA VALIDATION TESTS
// ============================================================================

#[test]
fn test_cat_a_01_valid_contract_parses() {
    let toml_str = r#"
    task_id = "task-auth-001"
    intent = "implement user authentication"
    requirements = [
        "POST /login returns 200 on valid credentials",
        "POST /login returns 401 on invalid credentials"
    ]
    preconditions = ["db migration applied"]
    postconditions = ["user session active"]
    allowed_operations = ["file_read", "crypto_hash", "time_read"]
    required_tests = ["tests/test_auth.end"]
    evidence_requirements = ["test_output", "artifact_hash"]
    security_boundaries = ["no_outbound_network", "no_exec_subprocess"]
    target_files = ["src/auth.end"]

    [provenance]
    agent = "antigravity-code-agent-v1"
    prompt_hash = "a1b2c3d4e5f67890"
    model_version = "gemini-3.7-flash"

    lifecycle = "DRAFT"
    "#;

    let contract = AgentContract::from_toml(toml_str).expect("Valid contract should parse");
    assert_eq!(contract.task_id, "task-auth-001");
    assert_eq!(contract.intent, "implement user authentication");
    assert_eq!(contract.requirements.len(), 2);
    assert_eq!(contract.allowed_operations, vec!["file_read", "crypto_hash", "time_read"]);
    assert_eq!(contract.provenance.agent, "antigravity-code-agent-v1");
    assert_eq!(contract.lifecycle, LifecycleState::Draft);
}

#[test]
fn test_cat_a_02_toml_roundtrip_preserves_semantics() {
    let contract = AgentContract {
        task_id: "task-roundtrip-99".to_string(),
        intent: "verify toml round-trip".to_string(),
        requirements: vec!["serialize toml".to_string(), "deserialize toml".to_string()],
        preconditions: vec!["schema ready".to_string()],
        postconditions: vec!["semantic identity preserved".to_string()],
        allowed_operations: vec!["file_read".to_string(), "db_query".to_string()],
        required_tests: vec!["tests/roundtrip.end".to_string()],
        evidence_requirements: vec!["artifact_hash".to_string()],
        security_boundaries: vec!["no_exec_subprocess".to_string()],
        target_files: vec!["src/lib.end".to_string()],
        artifact_hashes: {
            let mut m = HashMap::new();
            m.insert("src/lib.end".to_string(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string());
            m
        },
        provenance: Provenance::new("agent-tester", "prompt for roundtrip", "gemini-3.7"),
        lifecycle: LifecycleState::Submitted,
    };

    let toml_str = contract.to_toml().expect("serialization should succeed");
    let deserialized = AgentContract::from_toml(&toml_str).expect("deserialization should succeed");

    assert_eq!(contract.task_id, deserialized.task_id);
    assert_eq!(contract.intent, deserialized.intent);
    assert_eq!(contract.requirements, deserialized.requirements);
    assert_eq!(contract.allowed_operations, deserialized.allowed_operations);
    assert_eq!(contract.security_boundaries, deserialized.security_boundaries);
    assert_eq!(contract.artifact_hashes, deserialized.artifact_hashes);
    assert_eq!(contract.lifecycle, deserialized.lifecycle);
    assert_eq!(contract.provenance.agent, deserialized.provenance.agent);
}

#[test]
fn test_cat_a_03_missing_required_field_rejects() {
    // Missing task_id
    let invalid_toml = r#"
    task_id = ""
    intent = "do something"
    requirements = ["req 1"]
    [provenance]
    agent = "agent1"
    prompt_hash = "12345678"
    model_version = "v1"
    "#;

    let res = AgentContract::from_toml(invalid_toml);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("task_id"));
}

#[test]
fn test_cat_a_04_malformed_toml_rejects() {
    let malformed = "task_id = unquoted_bad_syntax [[[";
    let res = AgentContract::from_toml(malformed);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ContractError::ParseError(_)));
}

// ============================================================================
// CATEGORY B — LIFECYCLE STATE MACHINE TESTS
// ============================================================================

#[test]
fn test_cat_b_05_draft_to_submitted_succeeds() {
    let mut state = LifecycleState::Draft;
    assert!(state.transition(LifecycleState::Submitted).is_ok());
    assert_eq!(state, LifecycleState::Submitted);
}

#[test]
fn test_cat_b_06_submitted_to_verifying_succeeds() {
    let mut state = LifecycleState::Submitted;
    assert!(state.transition(LifecycleState::Verifying).is_ok());
    assert_eq!(state, LifecycleState::Verifying);
}

#[test]
fn test_cat_b_07_verifying_to_verified_succeeds() {
    let mut state = LifecycleState::Verifying;
    assert!(state.transition(LifecycleState::Verified).is_ok());
    assert_eq!(state, LifecycleState::Verified);
}

#[test]
fn test_cat_b_08_verifying_to_rejected_succeeds() {
    let mut state = LifecycleState::Verifying;
    assert!(state.transition(LifecycleState::Rejected).is_ok());
    assert_eq!(state, LifecycleState::Rejected);
}

#[test]
fn test_cat_b_09_verified_to_stale_succeeds() {
    let mut state = LifecycleState::Verified;
    assert!(state.transition(LifecycleState::Stale).is_ok());
    assert_eq!(state, LifecycleState::Stale);
}

#[test]
fn test_cat_b_10_stale_to_submitted_succeeds() {
    let mut state = LifecycleState::Stale;
    assert!(state.transition(LifecycleState::Submitted).is_ok());
    assert_eq!(state, LifecycleState::Submitted);
}

#[test]
fn test_cat_b_11_illegal_transitions_are_rejected() {
    // Draft -> Verified directly (must go through Submitted and Verifying)
    let mut state = LifecycleState::Draft;
    let err = state.transition(LifecycleState::Verified).unwrap_err();
    assert_eq!(err.current, LifecycleState::Draft);
    assert_eq!(err.target, LifecycleState::Verified);

    // Draft -> Stale
    let mut state2 = LifecycleState::Draft;
    assert!(state2.transition(LifecycleState::Stale).is_err());

    // Rejected -> Verified directly
    let mut state3 = LifecycleState::Rejected;
    assert!(state3.transition(LifecycleState::Verified).is_err());

    // Verified -> Draft
    let mut state4 = LifecycleState::Verified;
    assert!(state4.transition(LifecycleState::Draft).is_err());
}

// ============================================================================
// CATEGORY C — VERIFICATION ORCHESTRATION TESTS
// ============================================================================

#[test]
fn test_cat_c_12_valid_contract_with_passing_test_verifies() {
    let dir = create_temp_test_dir("passing_test");
    let contract_path = dir.join(".agents/contract.toml");
    let test_path = dir.join("tests/test_math.end");
    let src_path = dir.join("src/math.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();

    fs::write(&src_path, "fn add(a: i64, b: i64) -> i64 { return a + b; }\n").unwrap();
    fs::write(
        &test_path,
        r#"
        fn test_addition() -> bool {
            let res = 2 + 2;
            return res == 4;
        }
        "#,
    )
    .unwrap();

    let contract = AgentContract {
        task_id: "task-math-100".to_string(),
        intent: "implement math addition".to_string(),
        requirements: vec!["2 + 2 == 4".to_string()],
        preconditions: vec![],
        postconditions: vec!["addition verified".to_string()],
        allowed_operations: vec!["time_read".to_string()],
        required_tests: vec!["tests/test_math.end".to_string()],
        evidence_requirements: vec!["test_output".to_string(), "artifact_hash".to_string()],
        security_boundaries: vec!["no_outbound_network".to_string(), "no_exec_subprocess".to_string()],
        target_files: vec!["src/math.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("math-agent-v1", "build math module", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(outcome.is_verified(), "Expected outcome to be VERIFIED, got {:?}", outcome);

    // Verify contract on disk is now VERIFIED and evidence file is created
    let loaded = AgentContract::from_file(&contract_path).unwrap();
    assert_eq!(loaded.lifecycle, LifecycleState::Verified);
    assert!(!loaded.artifact_hashes.is_empty());
    assert!(dir.join(".agents/evidence.json").exists());
}

#[test]
fn test_cat_c_13_failing_required_test_rejects() {
    let dir = create_temp_test_dir("failing_test");
    let contract_path = dir.join(".agents/contract.toml");
    let test_path = dir.join("tests/test_fail.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    fs::write(
        &test_path,
        r#"
        fn test_assertion() -> bool {
            // Intentionally returns false to indicate failure
            return false;
        }
        "#,
    )
    .unwrap();

    let contract = AgentContract {
        task_id: "task-fail-101".to_string(),
        intent: "test failing verification".to_string(),
        requirements: vec!["must pass".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec![],
        required_tests: vec!["tests/test_fail.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec![],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("test-agent-real", "prompt hash 12345678", "model-v1"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(matches!(outcome, VerificationOutcome::Rejected { .. }));

    let loaded = AgentContract::from_file(&contract_path).unwrap();
    assert_eq!(loaded.lifecycle, LifecycleState::Rejected);
}

#[test]
fn test_cat_c_14_missing_required_test_rejects() {
    let dir = create_temp_test_dir("missing_test");
    let contract_path = dir.join(".agents/contract.toml");

    fs::create_dir_all(dir.join(".agents")).unwrap();

    let contract = AgentContract {
        task_id: "task-missing-102".to_string(),
        intent: "test missing test file".to_string(),
        requirements: vec!["must run missing test".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec![],
        required_tests: vec!["tests/nonexistent_test.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec![],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("agent-xyz", "prompt-abc-12345", "v1.0"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(matches!(outcome, VerificationOutcome::Rejected { .. }));
}

#[test]
fn test_cat_c_15_missing_provenance_rejects() {
    let dir = create_temp_test_dir("missing_provenance");
    let contract_path = dir.join(".agents/contract.toml");
    let test_path = dir.join("tests/test_ok.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(&test_path, "fn test_ok() -> bool { return true; }\n").unwrap();

    // Banned placeholder provenance "unknown"
    let contract = AgentContract {
        task_id: "task-bad-prov-103".to_string(),
        intent: "test bad provenance".to_string(),
        requirements: vec!["some req".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec![],
        required_tests: vec!["tests/test_ok.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec![],
        artifact_hashes: HashMap::new(),
        provenance: Provenance {
            agent: "unknown".to_string(),
            prompt_hash: "".to_string(),
            model_version: "default".to_string(),
            timestamp: None,
        },
        lifecycle: LifecycleState::Draft,
    };

    let toml_str = toml::to_string_pretty(&contract).unwrap();
    fs::write(&contract_path, toml_str).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    // Should fail as InvalidContract or Rejected due to bad provenance
    assert!(!outcome.is_verified());
}

// ============================================================================
// CATEGORY D — BUILD GATE TESTS
// ============================================================================

#[test]
fn test_cat_d_16_no_contract_preserves_behavior() {
    let dir = create_temp_test_dir("no_contract");
    let dummy_file = dir.join("src/main.end");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(&dummy_file, "fn main() -> i64 { return 0; }\n").unwrap();

    // find_contract_file should return None
    let found = AgentContract::find_contract_file(&dummy_file);
    assert!(found.is_none());
}

#[test]
fn test_cat_d_17_contract_discovery_finds_project_contract() {
    let dir = create_temp_test_dir("contract_discovery");
    let contract_path = dir.join(".agents/contract.toml");
    let source_file = dir.join("src/features/auth/login.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src/features/auth")).unwrap();

    let contract = AgentContract {
        task_id: "task-login-200".to_string(),
        intent: "user login".to_string(),
        requirements: vec!["req".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec![],
        required_tests: vec![],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec![],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("real-agent-1", "prompt-hash-11223344", "model-v2"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let found = AgentContract::find_contract_file(&source_file);
    assert!(found.is_some());
    assert_eq!(found.unwrap().canonicalize().unwrap(), contract_path.canonicalize().unwrap());
}

// ============================================================================
// CATEGORY E — SECURITY BOUNDARY ENFORCEMENT TESTS
// ============================================================================

#[test]
fn test_cat_e_20_allowed_operations_accepted() {
    let dir = create_temp_test_dir("sec_allowed");
    let contract_path = dir.join(".agents/contract.toml");
    let src_path = dir.join("src/compute.end");
    let test_path = dir.join("tests/test_compute.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    // Source code only does time_read and file_read
    fs::write(&src_path, "fn do_work() -> i64 { let t = time_read(); return 42; }\n").unwrap();
    fs::write(&test_path, "fn test_work() -> bool { return true; }\n").unwrap();

    let contract = AgentContract {
        task_id: "task-sec-allowed".to_string(),
        intent: "pure computation with allowed time_read".to_string(),
        requirements: vec!["return 42".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec!["time_read".to_string()],
        required_tests: vec!["tests/test_compute.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec!["no_outbound_network".to_string(), "no_exec_subprocess".to_string()],
        target_files: vec!["src/compute.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("sec-agent-1", "prompt-hash-aabbccdd", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(outcome.is_verified(), "Allowed operations should verify successfully");
}

#[test]
fn test_cat_e_21_disallowed_operation_rejected() {
    let dir = create_temp_test_dir("sec_disallowed");
    let contract_path = dir.join(".agents/contract.toml");
    let src_path = dir.join("src/secret_sender.end");
    let test_path = dir.join("tests/test_sender.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    // Code tries to do net_connect which is not in allowed_operations
    fs::write(&src_path, "fn send() { let sock = net_connect(\"https://api.external.com\"); }\n").unwrap();
    fs::write(&test_path, "fn test_sender() -> bool { return true; }\n").unwrap();

    let contract = AgentContract {
        task_id: "task-disallowed-op".to_string(),
        intent: "disallowed op detection".to_string(),
        requirements: vec!["no net".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec!["file_read".to_string()], // net_connect is NOT allowed!
        required_tests: vec!["tests/test_sender.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec!["src/secret_sender.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("sec-agent-2", "prompt-hash-11335577", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(matches!(outcome, VerificationOutcome::Rejected { .. }));
}

#[test]
fn test_cat_e_22_security_boundary_no_outbound_network_rejected() {
    let dir = create_temp_test_dir("boundary_net");
    let contract_path = dir.join(".agents/contract.toml");
    let src_path = dir.join("src/net_leak.end");
    let test_path = dir.join("tests/test_leak.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    fs::write(&src_path, "fn fetch_data() { let client = HttpClient::get(\"http://evil.com\"); }\n").unwrap();
    fs::write(&test_path, "fn test_leak() -> bool { return true; }\n").unwrap();

    let contract = AgentContract {
        task_id: "task-net-boundary".to_string(),
        intent: "test network security boundary".to_string(),
        requirements: vec!["isolated computation".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec!["file_read".to_string()],
        required_tests: vec!["tests/test_leak.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec!["no_outbound_network".to_string()],
        target_files: vec!["src/net_leak.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("sec-agent-3", "prompt-hash-99887766", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(matches!(outcome, VerificationOutcome::Rejected { .. }));
}

#[test]
fn test_cat_e_23_security_boundary_no_exec_subprocess_rejected() {
    let dir = create_temp_test_dir("boundary_exec");
    let contract_path = dir.join(".agents/contract.toml");
    let src_path = dir.join("src/runner.end");
    let test_path = dir.join("tests/test_runner.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    fs::write(&src_path, "fn run_sh() { let cmd = Command::new(\"bash\"); }\n").unwrap();
    fs::write(&test_path, "fn test_runner() -> bool { return true; }\n").unwrap();

    let contract = AgentContract {
        task_id: "task-exec-boundary".to_string(),
        intent: "test subprocess boundary".to_string(),
        requirements: vec!["no subprocess execution".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec!["time_read".to_string()],
        required_tests: vec!["tests/test_runner.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec!["no_exec_subprocess".to_string()],
        target_files: vec!["src/runner.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("sec-agent-4", "prompt-hash-55443322", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    let outcome = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(matches!(outcome, VerificationOutcome::Rejected { .. }));
}

// ============================================================================
// CATEGORY F — STALE CONTRACT DETECTION TESTS
// ============================================================================

#[test]
fn test_cat_f_24_verified_contract_with_unchanged_files_remains_fresh() {
    let dir = create_temp_test_dir("stale_unchanged");
    let contract_path = dir.join(".agents/contract.toml");
    let src_path = dir.join("src/lib.end");
    let test_path = dir.join("tests/test_lib.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    fs::write(&src_path, "fn value() -> i64 { return 100; }\n").unwrap();
    fs::write(&test_path, "fn test_value() -> bool { return true; }\n").unwrap();

    let contract = AgentContract {
        task_id: "task-stale-24".to_string(),
        intent: "stale freshness verification".to_string(),
        requirements: vec!["req".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec![],
        required_tests: vec!["tests/test_lib.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec!["src/lib.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("stale-agent-1", "prompt-hash-12344321", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    // 1. First verification -> VERIFIED
    let outcome1 = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(outcome1.is_verified());

    // 2. Check disk status -> remains fresh
    let loaded = AgentContract::from_file(&contract_path).unwrap();
    assert_eq!(loaded.lifecycle, LifecycleState::Verified);

    let stale_check = check_stale_against_disk(&dir, &loaded.artifact_hashes);
    assert!(stale_check.is_fresh());
}

#[test]
fn test_cat_f_25_verified_contract_with_modified_source_becomes_stale() {
    let dir = create_temp_test_dir("stale_modified");
    let contract_path = dir.join(".agents/contract.toml");
    let src_path = dir.join("src/lib.end");
    let test_path = dir.join("tests/test_lib.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    fs::write(&src_path, "fn initial() -> i64 { return 1; }\n").unwrap();
    fs::write(&test_path, "fn test_initial() -> bool { return true; }\n").unwrap();

    let contract = AgentContract {
        task_id: "task-stale-25".to_string(),
        intent: "test modification triggers STALE".to_string(),
        requirements: vec!["req".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec![],
        required_tests: vec!["tests/test_lib.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec!["src/lib.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("stale-agent-2", "prompt-hash-23455432", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    // 1. Initial verification -> VERIFIED
    let outcome1 = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(outcome1.is_verified());

    // 2. Modify source file!
    fs::write(&src_path, "fn modified() -> i64 { return 2; }\n").unwrap();

    // 3. Re-running verifier on VERIFIED contract detects stale and transitions to STALE
    let outcome2 = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(matches!(outcome2, VerificationOutcome::Stale { .. }), "Expected STALE, got {:?}", outcome2);

    let loaded = AgentContract::from_file(&contract_path).unwrap();
    assert_eq!(loaded.lifecycle, LifecycleState::Stale);
}

#[test]
fn test_cat_f_26_resubmission_after_stale_reverifies() {
    let dir = create_temp_test_dir("stale_resubmit");
    let contract_path = dir.join(".agents/contract.toml");
    let src_path = dir.join("src/lib.end");
    let test_path = dir.join("tests/test_lib.end");

    fs::create_dir_all(dir.join(".agents")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();

    fs::write(&src_path, "fn v1() -> i64 { return 1; }\n").unwrap();
    fs::write(&test_path, "fn test_v1() -> bool { return true; }\n").unwrap();

    let contract = AgentContract {
        task_id: "task-stale-26".to_string(),
        intent: "test re-submission after stale".to_string(),
        requirements: vec!["req".to_string()],
        preconditions: vec![],
        postconditions: vec![],
        allowed_operations: vec![],
        required_tests: vec!["tests/test_lib.end".to_string()],
        evidence_requirements: vec![],
        security_boundaries: vec![],
        target_files: vec!["src/lib.end".to_string()],
        artifact_hashes: HashMap::new(),
        provenance: Provenance::new("stale-agent-3", "prompt-hash-34566543", "gemini-3.7"),
        lifecycle: LifecycleState::Draft,
    };
    contract.save_to_file(&contract_path).unwrap();

    // Step 1: Verify -> VERIFIED
    let _ = ContractVerifier::verify(&contract_path, Some(&dir), true);

    // Step 2: Modify -> STALE
    fs::write(&src_path, "fn v2() -> i64 { return 2; }\n").unwrap();
    let outcome_stale = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(matches!(outcome_stale, VerificationOutcome::Stale { .. }));

    // Step 3: Update contract provenance/re-verify on STALE contract -> VERIFIED with new hashes!
    let outcome_reverified = ContractVerifier::verify(&contract_path, Some(&dir), true);
    assert!(outcome_reverified.is_verified());

    let loaded = AgentContract::from_file(&contract_path).unwrap();
    assert_eq!(loaded.lifecycle, LifecycleState::Verified);
    let new_src_hash = compute_file_hash(&src_path).unwrap();
    assert_eq!(loaded.artifact_hashes.get("src/lib.end"), Some(&new_src_hash));
}
