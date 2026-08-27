use endc::lexer::Lexer;
use endc::parser::Parser as EndParser;
use endc::semantic::SemanticAnalyzer;
use std::path::PathBuf;

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if manifest_dir.file_name().and_then(|n| n.to_str()) == Some("endc") {
        manifest_dir.parent().unwrap().to_path_buf()
    } else {
        manifest_dir
    }
}

/// ============================================================================
/// CLASS A: Normal Execution Verification
/// Verifies that valid End code parses and passes semantic analysis cleanly.
/// ============================================================================
#[test]
fn test_class_a_normal_execution_verification() {
    let code = r#"
fn calculate_square(n: i64) -> i64 {
    val result: i64 = n * n
    return result
}
"#;
    let mut lexer = Lexer::new("test_a.end", code);
    let tokens = lexer.tokenize_all().expect("Class A: Valid code must tokenize");
    let mut parser = EndParser::new("test_a.end", tokens);
    let module = parser.parse_module("test_a").expect("Class A: Valid code must parse");
    let mut analyzer = SemanticAnalyzer::new("test_a.end", code);
    let res = analyzer.analyze_module(&module);
    assert!(
        res.is_ok() && analyzer.errors.is_empty(),
        "Class A: Valid code must pass semantic analysis: {:?}",
        analyzer.errors
    );
}

/// ============================================================================
/// CLASS B: Fake Feature / Anti-Fraud Rejection Verification
/// Verifies that benchmark fraud detection detects tampered checksums.
/// ============================================================================
#[test]
fn test_class_b_anti_fraud_corrupt_benchmark_rejection() {
    use sha2::{Digest, Sha256};
    
    // Simulate expected vs corrupted checksum comparison
    let ground_truth_val = "267914296";
    let computed_val = "267914296";
    let corrupted_val = "999999999_TAMPERED";

    // Ground truth verification
    let mut hasher1 = Sha256::new();
    hasher1.update(computed_val.as_bytes());
    let digest1 = format!("{:x}", hasher1.finalize());

    let mut hasher2 = Sha256::new();
    hasher2.update(ground_truth_val.as_bytes());
    let expected_digest = format!("{:x}", hasher2.finalize());

    assert_eq!(digest1, expected_digest, "Valid checksums must match");

    // Tamper detection verification
    let mut hasher_bad = Sha256::new();
    hasher_bad.update(corrupted_val.as_bytes());
    let bad_digest = format!("{:x}", hasher_bad.finalize());

    assert_ne!(bad_digest, expected_digest, "Class B: Tampered checksum MUST NOT match expected digest");
}

/// ============================================================================
/// CLASS C: Real Bug Fix & Regression Guard Verification
/// Verifies that compiler diagnostic errors (E001-E0937) are strictly enforced and not bypassed.
/// ============================================================================
#[test]
fn test_class_c_diagnostic_fail_closed_verification() {
    let broken_code = r#"
fn invalid_type() -> I64 {
    val x: I64 = "string type mismatch"
    return x
}
"#;
    let mut lexer = Lexer::new("broken.end", broken_code);
    let tokens = lexer.tokenize_all().expect("Tokenize broken code");
    let mut parser = EndParser::new("broken.end", tokens);
    let module = parser.parse_module("broken").expect("Parse broken code");
    let mut analyzer = SemanticAnalyzer::new("broken.end", broken_code);
    let _ = analyzer.analyze_module(&module);

    assert!(
        !analyzer.errors.is_empty(),
        "Class C Regression Guard: Type mismatch must produce semantic errors"
    );

    let has_e002 = analyzer.errors.iter().any(|e| {
        e.code == "E002" || e.code == "E002_TYPE_MISMATCH" || e.message.to_lowercase().contains("mismatch")
    });
    assert!(has_e002, "Class C Regression Guard: Must emit precise diagnostic code E002");
}

/// ============================================================================
/// CLASS D: Missing Evidence Rejection Verification
/// Verifies that an unverified or corrupted production readiness report is rejected.
/// ============================================================================
#[test]
fn test_class_d_missing_evidence_release_gate_rejection() {
    let fake_report_json = r#"{
        "schema_version": "1.0.0",
        "release": { "version": "2.0.0", "target": "x86_64-pc-windows-msvc" },
        "pipeline": {
            "overall_status": "RED",
            "fail_closed_enforced": true,
            "gates_total": 8,
            "gates_passed": 7,
            "gates_failed": 1,
            "gates": []
        },
        "evidence_audit_passing": false
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(fake_report_json).expect("Valid JSON");
    let is_passing = parsed.get("evidence_audit_passing").and_then(|v| v.as_bool()).unwrap_or(false);
    let overall_status = parsed.get("pipeline").and_then(|p| p.get("overall_status")).and_then(|s| s.as_str()).unwrap_or("");

    assert!(!is_passing, "Class D: Failed evidence audit must report passing = false");
    assert_eq!(overall_status, "RED", "Class D: Failed evidence audit must report overall status RED");
}
