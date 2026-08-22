#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::security::*;
    use crate::semantic::analyzer::*;

    fn parse_str(code: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new("security_test.end", code);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new("security_test.end", tokens);
        parser.parse_module("security_test_mod")
    }

    // =========================================================================
    // PILLAR 1: Type-Level Security Primitives (Features 1–10)
    // =========================================================================

    #[test]
    fn test_pillar1_taint_and_sanitizer_propagation() {
        // 1. Negative Test: Tainted user input passed directly to raw SQL sink without parameterization
        let bad_code = r#"
            pub fn handle_request(user_input: tainted<str>) void {
                query(user_input)
            }
        "#;
        let module = parse_str(bad_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", bad_code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0921"));

        // 2. Positive Test: Tainted input sanitized via SqlValue parameterization
        let good_code = r#"
            pub fn handle_request(user_input: tainted<str>) void {
                val safe_val = SqlValue.param(user_input)
                query("SELECT * FROM users WHERE id = ?", safe_val)
            }
        "#;
        let good_module = parse_str(good_code).unwrap();
        let mut good_analyzer = SemanticAnalyzer::new("test.end", good_code);
        let good_res = good_analyzer.analyze_module(&good_module);
        assert!(good_res.is_ok());
    }

    #[test]
    fn test_pillar1_secret_and_private_leak_prevention() {
        // 1. Negative Test: secret<ApiKey> logged to stdout / logger
        let leak_code = r#"
            pub fn auth() void {
                val api_key: secret<str> = "sk_live_9999"
                println(api_key)
            }
        "#;
        let module = parse_str(leak_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", leak_code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0926"));

        // 2. Positive Test: secret isolated within secure cryptographic hash boundary
        let clean_code = r#"
            pub fn auth() void {
                val api_key: secret<str> = "sk_live_9999"
                val token_hash = crypto.sha256(api_key)
            }
        "#;
        let clean_module = parse_str(clean_code).unwrap();
        let mut clean_analyzer = SemanticAnalyzer::new("test.end", clean_code);
        assert!(clean_analyzer.analyze_module(&clean_module).is_ok());
    }

    #[test]
    fn test_pillar1_capability_authority_sealed_verified_types() {
        let code = r#"
            pub fn process_order(req: verified<str>, uid: sealed<i64>) void {
                val read_auth: authority<str> = authority.acquire()
            }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    // =========================================================================
    // PILLAR 2: Vulnerability Class Elimination & Primitive Safety (Features 11–20)
    // =========================================================================

    #[test]
    fn test_pillar2_sql_html_shell_path_injection_elimination() {
        // 1. HTML/XSS Sink Violation
        let xss_code = r#"
            pub fn render(raw_content: tainted<str>) void {
                render_html(raw_content)
            }
        "#;
        let xss_module = parse_str(xss_code).unwrap();
        let mut xss_analyzer = SemanticAnalyzer::new("test.end", xss_code);
        let errs = xss_analyzer.analyze_module(&xss_module).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "E0922"));

        // 2. OS Command Injection Violation
        let shell_code = r#"
            pub fn backup(cmd_input: tainted<str>) void {
                system(cmd_input)
            }
        "#;
        let shell_module = parse_str(shell_code).unwrap();
        let mut shell_analyzer = SemanticAnalyzer::new("test.end", shell_code);
        let shell_errs = shell_analyzer.analyze_module(&shell_module).unwrap_err();
        assert!(shell_errs.iter().any(|e| e.code == "E0923"));

        // 3. Path Traversal Violation
        let path_code = r#"
            pub fn load_file(user_path: tainted<str>) void {
                read_file(user_path)
            }
        "#;
        let path_module = parse_str(path_code).unwrap();
        let mut path_analyzer = SemanticAnalyzer::new("test.end", path_code);
        let path_errs = path_analyzer.analyze_module(&path_module).unwrap_err();
        assert!(path_errs.iter().any(|e| e.code == "E0924"));
    }

    #[test]
    fn test_pillar2_linear_nonce_single_use_consumption() {
        // Linear Nonce reuse rejection test
        let mut taint_analyzer = TaintAndInformationFlowAnalyzer::new("test.end", SecurityLevel::Strict);
        taint_analyzer.declare_security_symbol("nonce_token", SecurityTypeKind::CryptoNonce(96), 1);

        // 1st use succeeds
        let first_use = taint_analyzer.mark_nonce_consumed("nonce_token", 5, 1);
        assert!(first_use.is_ok());

        // 2nd use fails with E0930 (linear resource already consumed)
        let second_use = taint_analyzer.mark_nonce_consumed("nonce_token", 8, 1);
        assert!(second_use.is_err());
        let err = second_use.unwrap_err();
        assert_eq!(err.code, "E0930");
    }

    #[test]
    fn test_pillar2_constant_time_zeroize_non_serializable() {
        // 1. Constant-Time Timing Side-Channel Detection
        let timing_code = r#"
            @constant_time
            pub fn verify_mac(secret_mac: str, provided: str) bool {
                if secret_mac == provided {
                    ret true
                }
                ret false
            }
        "#;
        let module = parse_str(timing_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", timing_code);
        let errs = analyzer.analyze_module(&module).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "E0929"));

        // 2. Non-Serializable Secret Serialization Rejection
        let serialize_code = r#"
            pub fn export_data(private_key: secret<str>) void {
                val json_payload = to_json(private_key)
            }
        "#;
        let s_module = parse_str(serialize_code).unwrap();
        let mut s_analyzer = SemanticAnalyzer::new("test.end", serialize_code);
        let s_errs = s_analyzer.analyze_module(&s_module).unwrap_err();
        assert!(s_errs.iter().any(|e| e.code == "E0927"));
    }

    // =========================================================================
    // PILLAR 3: Capability Security & Zero Ambient Authority (Features 21–30)
    // =========================================================================

    #[test]
    fn test_pillar3_ambient_authority_rejection_and_bound_functions() {
        // Ambient system access without capability declaration is rejected
        let ambient_code = r#"
            pub fn write_logs() void {
                std.fs.write("/var/log/app.log", "data")
            }
        "#;
        let module = parse_str(ambient_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", ambient_code);
        let errs = analyzer.analyze_module(&module).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "E0933"));

        // With explicit capability declaration, compilation succeeds
        let cap_code = r#"
            @capability("disk=true")
            pub fn write_logs() void {
                // Granted capability
            }
        "#;
        let cap_module = parse_str(cap_code).unwrap();
        let mut cap_analyzer = SemanticAnalyzer::new("test.end", cap_code);
        assert!(cap_analyzer.analyze_module(&cap_module).is_ok());
    }

    #[test]
    fn test_pillar3_capability_intersection_delegation_and_revocation() {
        let mut cap_engine = CapabilityAndDomainEngine::new("test.end", SecurityLevel::Strict);
        cap_engine.grant_capability("FileRead", AuthorityLevel::Read, Some("/tmp"), 1);

        // Delegation succeeds
        let del_res = cap_engine.delegate_capability("FileRead", "worker_thread", 3);
        assert!(del_res.is_ok());

        // Revocation invalidates capability
        cap_engine.revoke_capability("FileRead", 5, 1);
        let del_after_revoke = cap_engine.delegate_capability("FileRead", "worker_thread_2", 7);
        assert!(del_after_revoke.is_err());
        assert_eq!(del_after_revoke.unwrap_err().code, "E0932");
    }

    #[test]
    fn test_pillar3_security_domains_and_privilege_escalation() {
        let escalation_code = r#"
            pub fn reader_routine() void {
                val low_privilege_ctx = 1
                admin_escalate(low_privilege_ctx)
            }
        "#;
        let module = parse_str(escalation_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", escalation_code);
        let errs = analyzer.analyze_module(&module).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "E0934"));
    }

    // =========================================================================
    // PILLAR 4: Compiler Security Gate & Contract Verification (Features 31–40)
    // =========================================================================

    #[test]
    fn test_pillar4_security_contracts_and_threat_models() {
        let contract_code = r#"
            contract PaymentSecurity {
                accepts: [PaymentRequest],
                guarantees: [AuditTrail]
            }
        "#;
        let module = parse_str(contract_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", contract_code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_pillar4_security_levels_and_zero_warning_build_gate() {
        let code = r#"
            pub fn standard_op() void {
                val x = 100
            }
        "#;
        let module = parse_str(code).unwrap();
        let (report, status) = SecurityByConstructionEngine::audit_module_and_source(
            "test.end",
            code,
            &module,
            SecurityLevel::Absolute,
        );

        assert!(report.is_secure);
        assert!(report.verified_build_permitted);
        assert!(matches!(status, VerifiedBuildStatus::Permitted { .. }));
    }

    #[test]
    fn test_pillar4_dependency_verification_and_compiler_attestation() {
        // 1. Unverified dependency rejection in strict mode
        let mut contract_verifier = SecurityContractVerifier::new("test.end", SecurityLevel::Strict);
        let dep_source = "dependency untrusted_pkg { trust: untrusted }";
        let module = parse_str("pub fn test_pkg() void {}").unwrap();
        contract_verifier.analyze_contracts_and_dependencies(dep_source, &module);
        assert!(contract_verifier.violations.iter().any(|e| e.code == "E0937"));

        // 2. Cryptographic Attestation Generation for clean artifact
        let clean_code = "pub fn add(a: i64, b: i64) i64 { ret a + b }";
        let manifest_status = AttestationEngine::evaluate_verified_build(
            clean_code,
            "math.end",
            SecurityLevel::Absolute,
            &[],
            &["math_soundness_proof".to_string()],
            &["CpuExecute".to_string()],
        );

        if let VerifiedBuildStatus::Permitted { manifest, .. } = manifest_status {
            assert_eq!(manifest.security_level, SecurityLevel::Absolute);
            assert_eq!(manifest.compiler_version, "2.5.0-security-by-construction");
            assert!(!manifest.attestation_digest.is_empty());
            assert!(manifest.is_reproducible);
        } else {
            panic!("Expected VerifiedBuildStatus::Permitted");
        }
    }

    // =========================================================================
    // PILLAR 5: Information Flow, Adversarial & Verified Build (Features 41–50)
    // =========================================================================

    #[test]
    fn test_pillar5_security_borrowing_and_declassification_proofs() {
        let mut taint_analyzer = TaintAndInformationFlowAnalyzer::new("test.end", SecurityLevel::Strict);
        taint_analyzer.declare_security_symbol("user_email", SecurityTypeKind::Secret(Box::new(Type::Str)), 1);

        // Declassification without proof fails with E0928
        let fail_declass = taint_analyzer.declassify_symbol("user_email", "", 5, 1);
        assert!(fail_declass.is_err());
        assert_eq!(fail_declass.unwrap_err().code, "E0928");

        // Declassification with formal proof succeeds
        let ok_declass = taint_analyzer.declassify_symbol("user_email", "proof<EmailMaskingProof>", 7, 1);
        assert!(ok_declass.is_ok());
    }

    #[test]
    fn test_pillar5_security_state_machines_and_api_quarantine() {
        // Unquarantined dangerous API call fails with E0936
        let raw_api_code = r#"
            pub fn raw_access() void {
                unsafe.raw_mem_write(0x1000, 42)
            }
        "#;
        let module = parse_str(raw_api_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", raw_api_code);
        let errs = analyzer.analyze_module(&module).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "E0936"));
    }

    #[test]
    fn test_pillar5_adversarial_compilation_and_absolute_verified_build() {
        // Vulnerable code is attacked and exploited during adversarial compilation
        let vuln_source = r#"
            pub fn login(user_input: tainted<str>) void {
                query(user_input)
                println(user_input)
            }
        "#;
        let (adv_report, adv_violation) =
            AdversarialSecurityEngine::run_adversarial_simulation(vuln_source, SecurityLevel::Absolute);

        assert!(adv_report.attacks_successful > 0);
        assert!(adv_violation.is_some());
        assert_eq!(adv_violation.unwrap().code, "E0938");
    }
}
