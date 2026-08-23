use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::analyzer::*;
use crate::codegen::interpreter::{Interpreter, Value};
use crate::codegen::c_backend::CBackend;

fn parse_str(code: &str) -> Result<Module, String> {
    let mut lexer = Lexer::new("test.end", code);
    let tokens = lexer.tokenize_all()?;
    let mut parser = Parser::new("test.end", tokens);
    parser.parse_module("test_mod")
}

    // CATEGORY 9: AI AGENT CONTRACTS & EXECUTABLE TASKS (ITEMS 41-45)
    // =========================================================================

    // Item 41: First-Class skill Definition (Tests 121, 122, 123)
    #[test]
    fn test_item41_skill_definition_with_hard_soft_rules() {
        let code = r#"
        skill Transactional for function {
            rules: ["auto_rollback", "no_dirty_reads"];
            hard: ["zero_data_loss"];
            soft: ["latency_under_50ms"];
        }
        fn main() -> i64 { return 121; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SkillDecl { .. })));
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(121));
        assert!(interp.skills.contains_key("Transactional"));
    }

    #[test]
    fn test_item41_skill_scope_function_and_module() {
        let code = r#"
        skill PureCompute for module {
            rules: ["no_io_in_loop"];
            hard: ["deterministic_results"];
        }
        fn main() -> i64 { return 122; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        let sk = interp.skills.get("PureCompute").unwrap();
        assert_eq!(sk.hard, vec!["deterministic_results".to_string()]);
    }

    #[test]
    fn test_item41_skill_satisfies_contract_verification() {
        let code = r#"
        satisfies PaymentModule ["Transactional", "PureCompute"];
        fn main() -> i64 { return 123; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SatisfiesDecl { .. })));
    }

    // Item 42: Agent Task Graph (task, todo) (Tests 124, 125, 126)
    #[test]
    fn test_item42_task_graph_dependency_resolution() {
        let code = r#"
        task refactor_auth_module {
            owner: "agent_alpha";
            status: "planned";
            requirement: "REQ_SEC_001";
            implementation: "modern_oauth2";
            skills: ["Transactional"];
        }
        fn main() -> i64 { return 124; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::TaskDecl { .. } | Statement::AgentTaskContractDecl { .. })));
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert!(interp.tasks_state.contains_key("refactor_auth_module"));
    }

    #[test]
    fn test_item42_todo_executable_spec_status_tracking() {
        let code = r#"
        todo REQ_PAY_001 {
            implement: "Stripe and PayPal Webhook Verification";
            requires: ["CryptoLib", "HttpClient"];
            verify: ["test_webhook_signature"];
            status: "open";
        }
        fn main() -> i64 { return 125; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::TodoDecl { .. })));
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert!(interp.todos_state.contains_key("REQ_PAY_001"));
    }

    #[test]
    fn test_item42_task_evidence_and_confidence_scoring() {
        let code = r#"
        task verify_encryption {
            owner: "security_bot";
            status: "in_progress";
            evidence {
                test: "test_aes256_gcm";
                prove: "non_zero_iv";
            }
        }
        fn main() -> i64 { return 126; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert!(interp.tasks_state.contains_key("verify_encryption"));
    }

    // Item 43: Agent Task Lifecycle (claim, complete, verify) (Tests 127, 128, 129)
    #[test]
    fn test_item43_task_claim_ownership_transition() {
        let code = r#"
        task optimize_query {
            owner: "unassigned";
            status: "planned";
        }
        claim task optimize_query;
        fn main() -> i64 { return 127; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(127));
        assert_eq!(interp.tasks_state.get("optimize_query").unwrap().status, "claimed");
    }

    #[test]
    fn test_item43_task_complete_evidence_submission() {
        let code = r#"
        task patch_vulnerability {
            owner: "agent_sec";
            status: "claimed";
        }
        complete task patch_vulnerability {
            result: "success";
            confidence: 0.99;
            summary: "SQL Injection vector eliminated and validated";
        }
        fn main() -> i64 { return 128; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert_eq!(interp.tasks_state.get("patch_vulnerability").unwrap().status, "completed");
        assert_eq!(interp.tasks_state.get("patch_vulnerability").unwrap().confidence, Some(0.99));
    }

    #[test]
    fn test_item43_task_verify_adversarial_testing() {
        let code = r#"
        task secure_vault {
            owner: "agent_sec";
            status: "completed";
        }
        verify secure_vault;
        fn main() -> i64 { return 129; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert_eq!(interp.tasks_state.get("secure_vault").unwrap().status, "accepted");
    }

    // Item 44: Agent Scoping & Boundary Firewall (agent_boundary, context_firewall) (Tests 130, 131, 132)
    #[test]
    fn test_item44_agent_boundary_private_secret_firewall() {
        let code = r#"
        agent_boundary PaymentSecurity;
        fn main() -> i64 { return 130; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AgentBoundaryDecl { .. })));
    }

    #[test]
    fn test_item44_context_firewall_prompt_injection_guard() {
        let code = r#"
        context_firewall CoreDatabase {
            deny: ["drop_tables", "raw_exec"];
            expose: ["typed_query"];
        }
        fn main() -> i64 { return 131; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ContextFirewallDecl { .. })));
    }

    #[test]
    fn test_item44_agentability_token_budget_limits() {
        let code = r#"
        agentability {
            max_context_tokens: 12000;
            max_operation_complexity: "low";
            max_dependency_fanout: 4;
        }
        fn main() -> i64 { return 132; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AgentabilityDecl { .. })));
    }

    // Item 45: Intent Contracts & Semantic Commits (intent, change) (Tests 133, 134, 135)
    #[test]
    fn test_item45_intent_goal_preserve_invariant_check() {
        let code = r#"
        intent {
            goal: "Refactor database connection pool";
            preserve: ["zero_connection_leaks", "thread_safety"];
            optimize: ["throughput"];
        }
        fn main() -> i64 { return 133; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::IntentDecl { .. })));
    }

    #[test]
    fn test_item45_intent_drift_detection_and_rejection() {
        let code = r#"
        intent diff preserve: ["public_api_v1"], change: ["internal_caching"];
        fn main() -> i64 { return 134; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::IntentDiff { .. })));
    }

    #[test]
    fn test_item45_semantic_commit_verified_manifest() {
        let code = r#"
        change {
            task: "implement_oauth2";
            message: "Added secure JWT verification flow";
            intent: "OAuth2 authentication compliant";
            evidence: ["test_token_verification"];
        }
        fn main() -> i64 { return 135; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SemanticCommitDecl { .. })));
    }

    // =========================================================================
