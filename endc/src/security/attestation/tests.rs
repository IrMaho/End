#[cfg(test)]
mod attestation_unit_and_integration_tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use crate::security::attestation::measurement::{
        measure_bytes_sha256, measure_dependencies, measure_environment, measure_file_sha256,
        SystemMeasurement,
    };
    use crate::security::attestation::software::{
        current_timestamp_iso8601, hex_decode, hex_encode, SoftwareAttestationSigner,
    };
    use crate::security::attestation::tpm::{TpmAttestationEngine, TpmDetector};
    use crate::security::attestation::types::{
        AttestationKind, AttestationQuote, TpmEvidence, VerificationError,
    };
    use crate::security::attestation::verifier::AttestationVerifier;
    use crate::security::attestation::AttestationEngine;

    fn temp_test_file(name: &str, content: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join("endc_attestation_tests");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join(name);
        fs::write(&file_path, content).expect("Failed to write temp test file");
        file_path
    }

    // =========================================================================
    // TEST GROUP A: MEASUREMENT DETERMINISM & SENSITIVITY
    // =========================================================================

    #[test]
    fn test_01_measurement_determinism_and_sensitivity() {
        // 1. Binary measurement
        let bin1 = temp_test_file("binary_a.bin", b"\x7fELF_END_BINARY_V1_PAYLOAD");
        let bin2 = temp_test_file("binary_b.bin", b"\x7fELF_END_BINARY_V1_PAYLOAD");
        let bin_mutated = temp_test_file("binary_mut.bin", b"\x7fELF_END_BINARY_V1_TAMPERED");

        let hash1 = measure_file_sha256(&bin1).unwrap();
        let hash2 = measure_file_sha256(&bin2).unwrap();
        let hash_mut = measure_file_sha256(&bin_mutated).unwrap();

        assert_eq!(hash1, hash2, "Identical content must produce identical SHA-256");
        assert_ne!(hash1, hash_mut, "Modified content must produce different SHA-256");
        assert_eq!(hash1.len(), 64);

        // 2. Environment canonicalization & sensitivity
        let env_a = vec![("FOO", "123"), ("BAR", "456"), ("BAZ", "789")];
        let env_a_reordered = vec![("BAZ", "789"), ("FOO", "123"), ("BAR", "456")];
        let env_b_modified = vec![("FOO", "123"), ("BAR", "999"), ("BAZ", "789")];

        let env_hash1 = measure_environment(Some(&env_a));
        let env_hash2 = measure_environment(Some(&env_a_reordered));
        let env_hash_mut = measure_environment(Some(&env_b_modified));

        assert_eq!(env_hash1, env_hash2, "Canonical environment sorting must produce identical hash regardless of input order");
        assert_ne!(env_hash1, env_hash_mut, "Modified environment variable must alter environment hash");

        // 3. Dependency measurement
        let dep1 = temp_test_file("dep1.end", b"pub fn helper() {}");
        let dep2 = temp_test_file("dep2.end", b"pub val MAX = 100");

        let deps_map = measure_dependencies(&[&dep1, &dep2]).unwrap();
        assert_eq!(deps_map.len(), 2);
        assert!(deps_map.values().all(|h| h.len() == 64));

        let _ = fs::remove_file(bin1);
        let _ = fs::remove_file(bin2);
        let _ = fs::remove_file(bin_mutated);
        let _ = fs::remove_file(dep1);
        let _ = fs::remove_file(dep2);
    }

    // =========================================================================
    // TEST GROUP B: SOFTWARE QUOTE GENERATION & VERIFICATION
    // =========================================================================

    #[test]
    fn test_02_software_quote_generation_and_verification() {
        let bin = temp_test_file("app.bin", b"END_COMPILED_EXECUTABLE_CODE");
        let dep = temp_test_file("stdlib.end", b"pub fn print() {}");

        let bin_hash = measure_file_sha256(&bin).unwrap();
        let env_hash = measure_environment(Some(&[("ENV", "production")]));
        let dep_hashes = measure_dependencies(&[&dep]).unwrap();

        let signer = SoftwareAttestationSigner::generate();
        let quote = signer
            .sign_quote(&bin_hash, &env_hash, &dep_hashes, None)
            .expect("Signing failed");

        assert_eq!(quote.kind, AttestationKind::Software);
        assert_eq!(quote.binary_sha256, bin_hash);
        assert_eq!(quote.env_hash, env_hash);
        assert_eq!(quote.dependency_hashes, dep_hashes);
        assert!(!quote.public_key.is_empty());
        assert!(!quote.signature.is_empty());
        assert!(quote.tpm_evidence.is_none(), "Software quote must not contain TPM evidence");

        // Offline verification
        AttestationVerifier::verify_quote_offline(&quote)
            .expect("Offline signature verification failed");

        // Active target verification
        let res = AttestationVerifier::verify_target(
            &quote,
            &bin,
            Some(&[("ENV", "production")]),
            Some(&[&dep]),
        )
        .expect("Target verification failed");

        assert!(res.attested, "Verification result must be attested: true");
        assert_eq!(res.kind, AttestationKind::Software);
        assert!(res.summary.contains("Verified Software Attestation"));

        let _ = fs::remove_file(bin);
        let _ = fs::remove_file(dep);
    }

    // =========================================================================
    // TEST GROUP C: ADVERSARIAL BINARY TAMPER REJECTION (BLOCKING GATE)
    // =========================================================================

    #[test]
    fn test_03_adversarial_binary_tamper_rejection() {
        let bin_path = temp_test_file("secure_app.bin", b"AUTHENTIC_ORIGINAL_BINARY_BYTES");
        let bin_hash = measure_file_sha256(&bin_path).unwrap();
        let env_hash = measure_environment(None);
        let dep_hashes = BTreeMap::new();

        let signer = SoftwareAttestationSigner::generate();
        let quote = signer
            .sign_quote(&bin_hash, &env_hash, &dep_hashes, None)
            .unwrap();

        // 1. Verify original binary passes
        let res_before = AttestationVerifier::verify_target(&quote, &bin_path, None, None);
        assert!(res_before.is_ok(), "Original untouched binary must verify");

        // 2. Tamper: mutate binary on disk after quote generation
        fs::write(&bin_path, b"TAMPERED_MALICIOUS_INJECTED_BYTES").unwrap();

        // 3. Verify tampered binary MUST fail closed
        let res_after = AttestationVerifier::verify_target(&quote, &bin_path, None, None);
        assert!(res_after.is_err(), "Tampered binary MUST fail verification!");

        match res_after.unwrap_err() {
            VerificationError::TamperedBinary { expected, actual } => {
                assert_eq!(expected, bin_hash);
                assert_ne!(actual, bin_hash);
            }
            other => panic!("Expected TamperedBinary error, got {:?}", other),
        }

        let _ = fs::remove_file(bin_path);
    }

    // =========================================================================
    // TEST GROUP D: ADVERSARIAL ENVIRONMENT & DEPENDENCY TAMPER REJECTION
    // =========================================================================

    #[test]
    fn test_04_adversarial_env_and_dependency_tamper_rejection() {
        let bin_path = temp_test_file("service.bin", b"SERVICE_BINARY_DATA");
        let dep_path = temp_test_file("auth_mod.end", b"pub fn verify_token() {}");

        let bin_hash = measure_file_sha256(&bin_path).unwrap();
        let env_vars = vec![("AUTH_DOMAIN", "auth.end.org"), ("PORT", "8080")];
        let env_hash = measure_environment(Some(&env_vars));
        let dep_hashes = measure_dependencies(&[&dep_path]).unwrap();

        let signer = SoftwareAttestationSigner::generate();
        let quote = signer
            .sign_quote(&bin_hash, &env_hash, &dep_hashes, None)
            .unwrap();

        // Tamper 1: Environment modified
        let modified_env = vec![("AUTH_DOMAIN", "hacked.attacker.com"), ("PORT", "8080")];
        let env_tamper_res = AttestationVerifier::verify_target(
            &quote,
            &bin_path,
            Some(&modified_env),
            Some(&[&dep_path]),
        );
        assert!(env_tamper_res.is_err(), "Environment tampering must be rejected!");
        assert!(matches!(
            env_tamper_res.unwrap_err(),
            VerificationError::EnvHashMismatch { .. }
        ));

        // Tamper 2: Dependency file modified on disk
        fs::write(&dep_path, b"pub fn backdoor() { bypass_auth(); }").unwrap();
        let dep_tamper_res = AttestationVerifier::verify_target(
            &quote,
            &bin_path,
            Some(&env_vars),
            Some(&[&dep_path]),
        );
        assert!(dep_tamper_res.is_err(), "Dependency file tampering must be rejected!");
        assert!(matches!(
            dep_tamper_res.unwrap_err(),
            VerificationError::DependencyMismatch { .. }
        ));

        let _ = fs::remove_file(bin_path);
        let _ = fs::remove_file(dep_path);
    }

    // =========================================================================
    // TEST GROUP E: QUOTE MUTATION REJECTION
    // =========================================================================

    #[test]
    fn test_05_adversarial_quote_mutation_rejection() {
        let signer = SoftwareAttestationSigner::generate();
        let quote = signer
            .sign_quote(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                &BTreeMap::new(),
                Some("2026-08-26T20:00:00Z"),
            )
            .unwrap();

        // 1. Mutate binary_sha256 in quote
        let mut q_bin = quote.clone();
        q_bin.binary_sha256 = "1111111111111111111111111111111111111111111111111111111111111111".to_string();
        assert!(AttestationVerifier::verify_quote_offline(&q_bin).is_err());

        // 2. Mutate env_hash in quote
        let mut q_env = quote.clone();
        q_env.env_hash = "2222222222222222222222222222222222222222222222222222222222222222".to_string();
        assert!(AttestationVerifier::verify_quote_offline(&q_env).is_err());

        // 3. Mutate timestamp in quote
        let mut q_ts = quote.clone();
        q_ts.timestamp = "2026-08-27T00:00:00Z".to_string();
        assert!(AttestationVerifier::verify_quote_offline(&q_ts).is_err());

        // 4. Mutate signature bytes
        let mut q_sig = quote.clone();
        q_sig.signature = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
        assert!(AttestationVerifier::verify_quote_offline(&q_sig).is_err());

        // 5. Mutate public key
        let mut q_key = quote.clone();
        q_key.public_key = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert!(AttestationVerifier::verify_quote_offline(&q_key).is_err());
    }

    // =========================================================================
    // TEST GROUP F: FALSE TPM CLAIM REJECTION
    // =========================================================================

    #[test]
    fn test_06_false_tpm_claim_rejection() {
        let signer = SoftwareAttestationSigner::generate();
        let quote = signer
            .sign_quote(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                &BTreeMap::new(),
                None,
            )
            .unwrap();

        // 1. Changing kind from "software" to "tpm2" without evidence MUST be rejected
        let mut false_tpm_quote = quote.clone();
        false_tpm_quote.kind = AttestationKind::Tpm2;
        let res1 = AttestationVerifier::verify_quote_offline(&false_tpm_quote);
        assert!(res1.is_err(), "False TPM claim without evidence must be rejected");
        assert!(matches!(
            res1.unwrap_err(),
            VerificationError::MissingTpmEvidence(_)
        ));

        // 2. Attaching fake hardware evidence to software quote MUST be rejected
        let mut false_tpm_with_ev = quote.clone();
        false_tpm_with_ev.kind = AttestationKind::Tpm2;
        false_tpm_with_ev.tpm_evidence = Some(TpmEvidence {
            manufacturer: "Fake-TPM".to_string(),
            tpm_version: "2.0".to_string(),
            spec_version: "1.38".to_string(),
            pcr_algorithm: "SHA256".to_string(),
            pcr_digest: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            pcr_values: BTreeMap::new(),
            quote_signature: "fake_sig".to_string(),
            is_hardware: true,
        });
        let res2 = AttestationVerifier::verify_quote_offline(&false_tpm_with_ev);
        assert!(res2.is_err(), "Fake TPM signature must fail verification");

        // 3. Software quote with hardware flag set MUST be rejected
        let mut software_with_hw = quote.clone();
        software_with_hw.tpm_evidence = Some(TpmEvidence {
            manufacturer: "Fake".to_string(),
            tpm_version: "2.0".to_string(),
            spec_version: "1.0".to_string(),
            pcr_algorithm: "SHA256".to_string(),
            pcr_digest: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            pcr_values: BTreeMap::new(),
            quote_signature: "sig".to_string(),
            is_hardware: true,
        });
        let res3 = AttestationVerifier::verify_quote_offline(&software_with_hw);
        assert!(res3.is_err());
        assert!(matches!(
            res3.unwrap_err(),
            VerificationError::FalseTpmClaim(_)
        ));
    }

    // =========================================================================
    // TEST GROUP G: TPM 2.0 DETECTION & HARDWARE QUOTE EXECUTION
    // =========================================================================

    #[test]
    fn test_07_tpm_detection_and_hardware_quote() {
        let status = TpmDetector::detect();
        println!(
            "TPM Status: present={}, ready={}, version={}, manufacturer={}",
            status.is_present, status.is_ready, status.version, status.manufacturer
        );

        if status.is_present && status.is_ready {
            // Hardware TPM 2.0 is available on this host!
            let bin = temp_test_file("tpm_target.bin", b"TPM_MEASURED_TARGET_CODE");
            let bin_hash = measure_file_sha256(&bin).unwrap();
            let env_hash = measure_environment(None);
            let dep_hashes = BTreeMap::new();

            let tpm_quote = TpmAttestationEngine::sign_quote(&bin_hash, &env_hash, &dep_hashes, None)
                .expect("TPM quote generation failed on available TPM hardware");

            assert_eq!(tpm_quote.kind, AttestationKind::Tpm2);
            assert!(tpm_quote.tpm_evidence.is_some());
            let ev = tpm_quote.tpm_evidence.as_ref().unwrap();
            assert!(ev.is_hardware);
            assert!(ev.tpm_version.starts_with("2."));
            assert_eq!(ev.pcr_values.len(), 8);

            // Independent verification of TPM quote
            AttestationVerifier::verify_quote_offline(&tpm_quote)
                .expect("TPM quote verification failed");

            let res = AttestationVerifier::verify_target(&tpm_quote, &bin, None, None)
                .expect("TPM target verification failed");
            assert!(res.attested);
            assert_eq!(res.kind, AttestationKind::Tpm2);

            let _ = fs::remove_file(bin);
        } else {
            // TPM not available in this environment
            println!("TPM 2.0 hardware not available in this test environment");
        }
    }

    // =========================================================================
    // TEST GROUP H: UNIFIED ATTESTATION ENGINE FACADE
    // =========================================================================

    #[test]
    fn test_08_attestation_engine_facade() {
        let bin_path = temp_test_file("facade_app.bin", b"FACADE_APP_BINARY");

        // Software explicit mode
        let soft_quote = AttestationEngine::attest_target(
            &bin_path,
            Some(AttestationKind::Software),
            Some(&[("VAR", "1")]),
            None,
            None,
        )
        .expect("AttestationEngine software quote failed");

        assert_eq!(soft_quote.kind, AttestationKind::Software);

        let soft_res = AttestationEngine::verify_target(
            &soft_quote,
            &bin_path,
            Some(&[("VAR", "1")]),
            None,
        )
        .expect("AttestationEngine verification failed");

        assert!(soft_res.attested);

        let _ = fs::remove_file(bin_path);
    }

    // =========================================================================
    // TEST GROUP I: SERIALIZATION AND DESERIALIZATION
    // =========================================================================

    #[test]
    fn test_09_quote_json_serialization_roundtrip() {
        let signer = SoftwareAttestationSigner::generate();
        let quote = signer
            .sign_quote(
                "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                &BTreeMap::new(),
                Some("2026-08-26T20:15:30Z"),
            )
            .unwrap();

        let json = serde_json::to_string_pretty(&quote).unwrap();
        println!("Serialized Quote JSON:\n{}", json);

        let deserialized: AttestationQuote = serde_json::from_str(&json).unwrap();
        assert_eq!(quote, deserialized);

        // Verification must succeed on deserialized quote
        AttestationVerifier::verify_quote_offline(&deserialized).unwrap();
    }

    // =========================================================================
    // TEST GROUP J: LEGACY COMPILER VERIFIED BUILD COMPATIBILITY
    // =========================================================================

    #[test]
    fn test_10_legacy_verified_build_status() {
        let manifest_status = AttestationEngine::evaluate_verified_build(
            "pub fn main() void {}",
            "main.end",
            crate::security::types::SecurityLevel::Absolute,
            &[],
            &["memory_safety_proof".to_string()],
            &["CpuExecute".to_string()],
        );

        if let crate::security::VerifiedBuildStatus::Permitted { manifest, summary } = manifest_status {
            assert_eq!(manifest.security_level, crate::security::types::SecurityLevel::Absolute);
            assert_eq!(manifest.source_hash.len(), 64);
            assert_eq!(manifest.ast_semantic_hash.len(), 64);
            assert_eq!(manifest.attestation_digest.len(), 64);
            assert!(manifest.is_reproducible);
            assert!(summary.contains("Verified Build Succeeded"));
        } else {
            panic!("Expected Permitted build status");
        }
    }

    // =========================================================================
    // TEST GROUP K: END INTERPRETER ATTESTATION BUILTINS
    // =========================================================================

    #[test]
    fn test_11_end_interpreter_attestation_builtins() {
        let bin_path = temp_test_file("script_target.bin", b"INTERPRETER_TARGET_DATA");
        let bin_str = bin_path.to_string_lossy().replace('\\', "/");

        let src = format!(r#"
fn main() {{
    val target = "{}"
    val quote_json = end_attest_generate(target, "software")
    val kind = end_attest_quote_kind(quote_json)
    val digest = end_attest_quote_digest(quote_json)
    val verified = end_attest_verify(quote_json, target)
    ret verified
}}
        "#, bin_str);

        let mut lexer = crate::lexer::Lexer::new("test_attest.end", &src);
        let tokens = lexer.tokenize_all().unwrap();
        let mut parser = crate::parser::Parser::new("test_attest.end", tokens);
        let module = parser.parse_module("test_attest").unwrap();
        let mut interp = crate::codegen::interpreter::Interpreter::new();
        let res = interp.run(&module).unwrap();

        if let crate::codegen::interpreter::Value::Bool(b) = res {
            assert!(b, "Interpreter end_attest_verify should return true for untampered binary");
        } else {
            panic!("Expected bool return value from main, got {:?}", res);
        }

        let _ = fs::remove_file(bin_path);
    }
}
