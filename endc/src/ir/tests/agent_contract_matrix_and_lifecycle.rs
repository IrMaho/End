use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ir::hir::*;
use crate::ir::lowering::*;
use crate::ir::mir_lowering::*;
use crate::semantic::analyzer::*;
use crate::codegen::interpreter::{Interpreter, Value};
use crate::agent_api::evolution_engine::*;

fn parse_str(code: &str) -> Result<Module, String> {
    let mut lexer = Lexer::new("test.end", code);
    let tokens = lexer.tokenize_all()?;
    let mut parser = Parser::new("test.end", tokens);
    parser.parse_module("test_mod")
}

    #[test]
    fn test_50_agent_contract_system_complete_matrix() {
        let code = r#"
        // 1. Feature Definition & Contracts
        feature PaymentProcessing {
            requirement: "All transactions must be idempotent and audited";
            skills: ["Idempotent", "TransactionSafe", "AuditLogged"];
            tasks: ["AUTH_01", "PAY_02", "VERIFY_03"];
        }

        // 2-7. First-Class Skills with Scopes, Hard/Soft Constraints, Rules
        skill TransactionSafe for module {
            rules: ["no_unhandled_exceptions", "auto_rollback_on_failure"];
            requires: ["DbConnection", "IsolationLevelSerializable"];
            hard: ["zero_dirty_reads"];
            soft: ["max_query_time_50ms"];
        }

        skill Idempotent for function {
            rules: ["deterministic_id_generation", "cache_replay_protection"];
            hard: ["no_double_charge"];
        }

        // 8. Project Skill Profiles
        project {
            skills {
                quality: "strict";
                agent_autonomy: "supervised";
                verification_level: "formal_and_evidence";
            }
        }

        // 9-11. Task Graph & Lifecycle State Machine
        task implement_secure_payment {
            owner: "agent_codex";
            status: "planned";
            requirement: "REQ_PAY_001";
            implementation: "process_secure_tx";
            skills: ["TransactionSafe", "Idempotent"];
            change_budget: ["max_files: 5", "max_lines: 300", "max_modules: 2"];
            evidence {
                test: "test_replay_attack_prevention";
                prove: "guarantee_single_deduction";
                trace: "audit_trail_001";
            }
        }

        // 12-16. Evidence System & Traceability Matrix
        requirement REQ_PAY_001 { "Payment deduction must occur exactly once per idempotency key." }
        implements REQ_PAY_001 ["process_secure_tx", "validate_idempotency_key"];
        verifies REQ_PAY_001 ["test_replay_attack_prevention", "prove_single_charge"];

        // 17-19. Executable TODOs with Status Tracking
        todo AUTH_01 {
            implement: "OAuth2 Token Validation with Refresh Flow";
            requires: ["CryptoLib", "JwtDecoder"];
            verify: ["test_token_expiration", "test_signature_tamper"];
            status: "implemented";
        }

        todo PAY_02 {
            implement: "Card Gateway Stripe & PayPal Adapter";
            requires: ["HttpClient", "SecretKey"];
            verify: ["test_stripe_charge", "test_paypal_charge"];
            status: "planned";
        }

        // 20-26. Skill Satisfies & Scope Enforcement
        satisfies PaymentsModule ["TransactionSafe", "AuditLogged"];

        // 27-32. Agentability, Context Budget & Boundaries
        agentability {
            max_context_tokens: 16000;
            max_operation_complexity: "low";
            max_dependency_fanout: 6;
        }

        agent_boundary CoreSecurity;

        agent_context PaymentsModule {
            expose: ["process_payment", "refund_transaction"];
            hide: ["private_key", "raw_credit_card_data"];
        }

        context_firewall DatabaseLayer {
            deny: ["direct_sql_injection", "raw_table_drop"];
            expose: ["sanitized_query_interface"];
        }

        agent_api SecurityGateway {
            expose: ["authenticate_user", "refresh_session"];
            hide: ["internal_hashing_salt"];
        }

        // 33-37. Intent, Intent Drift & Semantic Commits
        intent {
            goal: "Refactor payment adapter without breaking idempotency guarantees";
            preserve: ["idempotency", "zero_double_charge"];
            optimize: ["latency", "throughput"];
        }

        change {
            task: "implement_secure_payment";
            intent: "Optimize database connection pooling in payment gateway";
            satisfies: ["TransactionSafe", "Idempotent"];
            evidence: ["test_replay_attack_prevention", "latency_benchmark_under_10ms"];
        }

        // 38-42. Agent Review, Approval & Leases
        review implement_secure_payment {
            summary: "Idempotency key check implemented using in-memory atomic hash map with Redis fallback";
            completed: 3;
            unresolved: 0;
            risks: 0;
            confidence: 0.99;
        }

        approval required ["lead_security_architect", "qa_automated_pipeline"];

        agent lease PaymentsModule {
            owner: "agent_alpha_autonomous";
            duration: "task_duration_2h";
        }

        // 43-47. Knowledge Base & ADR Decisions
        knowledge PaymentArchitecture {
            decisions: ["ADR_001_Use_Ed25519", "ADR_002_Two_Phase_Commit"];
            constraints: ["PCI_DSS_Level_1", "GDPR_Compliance"];
        }

        decision ADR_001_Use_Ed25519 {
            choose: "Ed25519 for signature verification";
            because: "Provides superior resistance to side-channel attacks and sub-millisecond verification";
            reject: "RSA-2048 due to key size and CPU overhead";
        }

        // 48-50. Agent Capabilities, Regression Guards & Verification Workflow
        agent {
            capabilities: ["code_generation", "unit_test_authoring", "algebraic_refactoring"];
            cannot: ["commit_secrets_to_vcs", "modify_protected_production_branches"];
        }

        regression_guard ["latency < 25ms", "memory_allocation < 10MB", "zero_data_leaks"];

        // Execution & Lifecycle State Transitions
        claim task implement_secure_payment;

        complete task implement_secure_payment {
            result: "success";
            confidence: 0.98;
            summary: "Transaction safety and idempotency fully implemented and proved";
            evidence: ["test_replay_attack_prevention", "guarantee_single_deduction"];
            risks: "none";
            recommendation: "promote_to_production";
            notes: "Zero regressions detected";
        }

        verify implement_secure_payment;
        verify adversarial {
            skill: "Idempotent";
        }

        fn main() -> i64 {
            return 100;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(100));

        // 1. Verify Features State
        assert!(interp.features.contains_key("PaymentProcessing"));
        let feat = interp.features.get("PaymentProcessing").unwrap();
        assert_eq!(feat.1, vec!["Idempotent", "TransactionSafe", "AuditLogged"]);
        assert_eq!(feat.2, vec!["AUTH_01", "PAY_02", "VERIFY_03"]);

        // 2. Verify Skills & Scopes
        assert!(interp.skills.contains_key("TransactionSafe"));
        let skill = interp.skills.get("TransactionSafe").unwrap();
        assert!(skill.for_scope == Some("module".to_string()) || skill.for_scope == Some("mod".to_string()));
        assert_eq!(skill.hard, vec!["zero_dirty_reads"]);
        assert_eq!(skill.soft, vec!["max_query_time_50ms"]);

        // 3. Verify Project Profile
        assert_eq!(interp.project_profile.get("quality").unwrap(), "strict");
        assert_eq!(interp.project_profile.get("verification_level").unwrap(), "formal_and_evidence");

        // 4. Verify Task Lifecycle (claim -> complete -> verify -> accepted)
        assert!(interp.tasks_state.contains_key("implement_secure_payment"));
        let task_st = interp.tasks_state.get("implement_secure_payment").unwrap();
        assert_eq!(task_st.status, "accepted");
        assert_eq!(task_st.result, Some("success".to_string()));
        assert_eq!(task_st.confidence, Some(0.98));
        assert!(interp.verified_tasks.contains("implement_secure_payment"));
        assert!(interp.verified_tasks.contains("adversarial"));

        // 5. Verify Requirement Traceability
        assert!(interp.requirements.contains_key("REQ_PAY_001"));
        assert_eq!(interp.requirement_implements.get("REQ_PAY_001").unwrap(), &vec!["process_secure_tx", "validate_idempotency_key"]);
        assert_eq!(interp.requirement_verifies.get("REQ_PAY_001").unwrap(), &vec!["test_replay_attack_prevention", "prove_single_charge"]);

        // 6. Verify Executable TODOs
        assert!(interp.todos_state.contains_key("AUTH_01"));
        assert_eq!(interp.todos_state.get("AUTH_01").unwrap().status, "implemented");
        assert_eq!(interp.todos_state.get("PAY_02").unwrap().status, "planned");

        // 7. Verify Agent Leases & ADRs
        assert!(interp.agent_leases.contains_key("PaymentsModule"));
        let lease = interp.agent_leases.get("PaymentsModule").unwrap();
        assert_eq!(lease.0, "agent_alpha_autonomous");

        assert!(interp.knowledge_base.contains_key("PaymentArchitecture"));
        assert!(interp.decision_records.contains_key("ADR_001_Use_Ed25519"));
        let decision = interp.decision_records.get("ADR_001_Use_Ed25519").unwrap();
        assert_eq!(decision.0, "Ed25519 for signature verification");

        // 8. Verify Agent Reports
        assert_eq!(interp.agent_reports.len(), 1);
        assert_eq!(interp.agent_reports[0].task_id, "implement_secure_payment");
        assert_eq!(interp.agent_reports[0].confidence, 0.99);

        // 9. Verify C Backend Codegen
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("/* 🎯 [FEATURE 'PaymentProcessing']"));
        assert!(c_code.contains("/* 🧠 [SKILL 'TransactionSafe'"));
        assert!(c_code.contains("/* 🏛️ [PROJECT SKILLS PROFILE]"));
        assert!(c_code.contains("/* 📋 [AGENT TASK CONTRACT 'implement_secure_payment']"));
        assert!(c_code.contains("/* 📜 [REQUIREMENT 'REQ_PAY_001']"));
        assert!(c_code.contains("/* 🔨 [IMPLEMENTS 'REQ_PAY_001']"));
        assert!(c_code.contains("/* 🛡️ [VERIFIES 'REQ_PAY_001']"));
        assert!(c_code.contains("/* 📝 [EXECUTABLE TODO 'AUTH_01']"));
        assert!(c_code.contains("/* 🧱 [AGENT BOUNDARY]: 'CoreSecurity' */"));
        assert!(c_code.contains("/* 👁️ [AGENT CONTEXT 'PaymentsModule']"));
        assert!(c_code.contains("/* 🧱🔥 [CONTEXT FIREWALL 'DatabaseLayer']"));
        assert!(c_code.contains("/* 🤖 [AGENT API 'SecurityGateway']"));
        assert!(c_code.contains("/* ⚙️ [AGENTABILITY BUDGET]"));
        assert!(c_code.contains("/* 🎯 [INTENT]"));
        assert!(c_code.contains("/* 💾 [SEMANTIC COMMIT]"));
        assert!(c_code.contains("/* 🧐 [AGENT REVIEW 'implement_secure_payment']"));
        assert!(c_code.contains("/* ✍️ [APPROVAL REQUIRED]"));
        assert!(c_code.contains("/* 🔑 [AGENT LEASE on 'PaymentsModule']"));
        assert!(c_code.contains("/* 📚 [KNOWLEDGE 'PaymentArchitecture']"));
        assert!(c_code.contains("/* ⚖️ [DECISION ADR 'ADR_001_Use_Ed25519']"));
        assert!(c_code.contains("/* 🛡️ [AGENT CAPABILITIES]"));
        assert!(c_code.contains("/* 🛡️ [REGRESSION GUARD]"));
        assert!(c_code.contains("/* 🙋 [CLAIM TASK]: 'implement_secure_payment' */"));
        assert!(c_code.contains("/* 🏁 [COMPLETE TASK 'implement_secure_payment']"));
        assert!(c_code.contains("/* 🔍 [VERIFY TASK 'implement_secure_payment']"));
        assert!(c_code.contains("/* 🔍 [VERIFY TASK 'adversarial']"));
    }

    #[test]
    fn test_agent_contract_lifecycle_claim_complete_verify_proof_of_work() {
        let code = r#"
        task migration_v2 {
            owner: "dev_agent";
            status: "planned";
            skills: ["SafeMigration"];
            evidence {
                test: "verify_db_schema_version";
            }
        }

        claim task migration_v2;

        complete task migration_v2 {
            result: "success";
            confidence: 0.95;
            summary: "Database migrations executed with zero downtime";
            evidence: ["verify_db_schema_version", "rollback_snapshot_verified"];
            notes: "Completed in 45ms";
        }

        verify migration_v2;

        fn main() -> i64 {
            return 42;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(42));

        let task = interp.tasks_state.get("migration_v2").unwrap();
        assert_eq!(task.status, "accepted");
        assert_eq!(task.result, Some("success".to_string()));
        assert_eq!(task.confidence, Some(0.95));
        assert!(interp.verified_tasks.contains("migration_v2"));
    }

    #[test]
    fn test_agent_contract_todo_and_traceability_graph() {
        let code = r#"
        requirement SEC_01 { "Passwords must be hashed with Argon2id and salt." }
        implements SEC_01 ["hash_password", "verify_password"];
        verifies SEC_01 ["test_argon2id_cost_parameters"];

        todo HASH_TODO {
            implement: "Argon2id hashing algorithm with 64MB memory cost";
            requires: ["MemoryBudget64MB", "Argon2Library"];
            verify: ["test_argon2id_cost_parameters"];
            status: "implemented";
        }

        fn main() -> i64 {
            return 77;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(77));

        assert_eq!(interp.requirements.get("SEC_01").unwrap(), "Passwords must be hashed with Argon2id and salt.");
        assert_eq!(interp.requirement_implements.get("SEC_01").unwrap(), &vec!["hash_password", "verify_password"]);
        assert_eq!(interp.requirement_verifies.get("SEC_01").unwrap(), &vec!["test_argon2id_cost_parameters"]);

        let todo = interp.todos_state.get("HASH_TODO").unwrap();
        assert_eq!(todo.status, "implemented");
        assert_eq!(todo.requires, vec!["MemoryBudget64MB", "Argon2Library"]);
    }

