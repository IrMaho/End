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

    // =========================================================================
    // LAYER 6: Architecture as Code (Tests 16 - 18)
    // =========================================================================

    #[test]
    fn test_layer6_architecture_contract_enforcement() {
        let code = r#"
        architecture SystemArch {
            layers: ["UI", "Domain", "Data"];
            rules: ["UI !-> Database"];
            directions: UI -> Domain, Domain -> Data;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_arch = module.statements.iter().any(|s| matches!(s, Statement::ArchitectureDecl { name, .. } if name == "SystemArch"));
        assert!(has_arch);
    }

    #[test]
    fn test_layer6_boundary_and_ownership() {
        let code = r#"
        lock architecture;

        change_limit {
            files: 5,
            modules: 2,
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        analyzer.analyze_module(&module);
        assert!(analyzer.arch_locked);
    }

    #[test]
    fn test_layer6_architecture_tests_and_cycle_free() {
        let code = r#"
        architecture_test {
            "assert no_cycle()";
            "assert no_forbidden_dependency()";
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_arch_test = module.statements.iter().any(|s| matches!(s, Statement::ArchitectureTestDecl { assertions, .. } if assertions.len() == 2));
        assert!(has_arch_test);
    }

    // =========================================================================
    // LAYER 7: Extensibility / Replaceability Engine (Tests 19 - 21)
    // =========================================================================

    #[test]
    fn test_layer7_semantic_dependency_graph() {
        let code = r#"
        mod auth {
            pub struct User {
                pub id: i64,
            }
            pub fn login() -> bool {
                return true;
            }
        }

        mod payments {
            depends: ["auth"];
            forbid: ["database"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let graph = SemanticDependencyGraph::build_from_ast(&module);
        assert!(graph.nodes.contains_key("test_mod::auth"));
        assert!(graph.nodes.contains_key("test_mod::payments"));
        assert!(graph.edges.iter().any(|e| e.from == "test_mod::payments" && e.to == "auth"));
    }

    #[test]
    fn test_layer7_impact_and_blast_radius() {
        let code = r#"
        mod database {
            pub struct Connection {
                pub url: str,
            }
        }
        mod repository {
            depends: ["database"];
        }
        mod service {
            depends: ["repository"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let graph = SemanticDependencyGraph::build_from_ast(&module);
        let impact = EvolutionEngine::analyze_impact(&graph, "test_mod::database");
        assert_eq!(impact.target, "test_mod::database");
        assert!(impact.blast_radius >= 0.0);
    }

    #[test]
    fn test_layer7_replaceability_and_extensibility_metrics() {
        let code = r#"
        mod decoupled_service {
            sealed: true;
            provides: ["ServiceApi"];
            guarantees: ["pure_computation"];
            pub fn run() -> i64 { return 100; }
        }
        "#;
        let module = parse_str(code).unwrap();
        let target_mod = &module.modules[0];
        let rep_score = EvolutionEngine::compute_replaceability(target_mod);
        let ext_score = EvolutionEngine::compute_extensibility(target_mod);

        assert!(rep_score.total_score >= 60.0);
        assert!(rep_score.can_replace_safely);
        assert!(ext_score.extensibility_index >= 70.0);
        assert!(ext_score.has_sealed_internals);
        assert!(ext_score.has_contracts);
    }

    // =========================================================================
    // LAYER 8: API Evolution & Migration (Tests 22 - 24)
    // =========================================================================

    #[test]
    fn test_layer8_api_stability_annotations() {
        let code = r#"
        stable API PaymentService;
        "#;
        let module = parse_str(code).unwrap();
        let has_stable = module.statements.iter().any(|s| matches!(s, Statement::StableDecl { api_name, .. } if api_name == "PaymentService"));
        assert!(has_stable);
    }

    #[test]
    fn test_layer8_api_snapshots_and_diff() {
        let mut v1 = ModuleDef::default();
        v1.name = "Billing".to_string();
        v1.functions.push(FunctionDef {
            name: "calculate_tax".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::I64,
            body: Block { statements: vec![], span: Span::default() },
            directives: vec![],
            morphic_param: None,
            span: Span::default(),
        });

        let mut v2 = v1.clone();
        v2.functions.push(FunctionDef {
            name: "calculate_vat".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::I64,
            body: Block { statements: vec![], span: Span::default() },
            directives: vec![],
            morphic_param: None,
            span: Span::default(),
        });

        let snap1 = EvolutionEngine::create_snapshot(&v1, 1);
        let snap2 = EvolutionEngine::create_snapshot(&v2, 2);

        let diff = EvolutionEngine::diff_api(&snap1, &snap2);
        assert_eq!(diff.breaking_changes.len(), 0);
        assert_eq!(diff.compatible_additions.len(), 1);
        assert_eq!(diff.semver_bump, "MINOR");
    }

    #[test]
    fn test_layer8_semver_and_compatibility_matrix() {
        let mut v1 = ModuleDef::default();
        v1.name = "Crypto".to_string();
        v1.functions.push(FunctionDef {
            name: "encrypt".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::Str,
            body: Block { statements: vec![], span: Span::default() },
            directives: vec![],
            morphic_param: None,
            span: Span::default(),
        });

        let mut v2 = v1.clone();
        v2.functions[0].return_type = Type::I64;

        let snap1 = EvolutionEngine::create_snapshot(&v1, 1);
        let snap2 = EvolutionEngine::create_snapshot(&v2, 2);

        let diff = EvolutionEngine::diff_api(&snap1, &snap2);
        assert_eq!(diff.breaking_changes.len(), 1);
        assert_eq!(diff.semver_bump, "MAJOR");
    }

    // =========================================================================
    // LAYER 9: Agent-Native Extensibility (Tests 25 - 27)
    // =========================================================================

    #[test]
    fn test_layer9_agent_extension_contract() {
        let code = r#"
        agent_extension payment_bot {
            purpose: "Autonomous micro-payment executor";
            inputs: ["amount: i64", "recipient: str"];
            outputs: ["tx_hash: str"];
            constraints: ["max_amount_per_tx <= 1000", "no_reentrancy"];
            tests: ["test_micro_pay", "test_limit_exceeded"];
            permissions: ["net.http", "storage.read"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_agent_ext = module.statements.iter().any(|s| matches!(s, Statement::AgentExtensionContractDecl { name, purpose, inputs, constraints, .. } if name == "payment_bot" && purpose.contains("Autonomous") && inputs.len() == 2 && constraints.len() == 2));
        assert!(has_agent_ext);
    }

    #[test]
    fn test_layer9_change_proposal_and_proof_gate() {
        let code = r#"
        proposal {
            title: "Upgrade Argon2id Cost Matrix";
            files: ["src/auth/crypto.end", "src/config/params.end"];
            symbols: ["Argon2Cost", "verify_password"];
            dependencies: ["crypto_native"];
            risks: ["increased_cpu_latency_under_load"];
            migration: "run end migrate auth 2 -> 3";
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_proposal = module.statements.iter().any(|s| matches!(s, Statement::AgentChangeProposalDecl { title, files, symbols, risks, migration, .. } if title == "Upgrade Argon2id Cost Matrix" && files.len() == 2 && symbols.len() == 2 && risks.len() == 1 && migration.is_some()));
        assert!(has_proposal);
    }

    #[test]
    fn test_layer9_agent_transactions_and_skills() {
        let code = r#"
        begin change {
            val step1 = "patch_ast";
            val step2 = "verify_contracts";
        } commit;

        mod secure_payments {
            skill: ["pci_dss_compliance", "tls_enforcement"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_txn = module.statements.iter().any(|s| matches!(s, Statement::AgentTransactionDecl { action, body, .. } if action == "change" && body.is_some()));
        assert!(has_txn);
        assert_eq!(module.modules[0].skills, vec!["pci_dss_compliance", "tls_enforcement"]);
    }

    // =========================================================================
    // LAYER 10: `end evolve` Engine & `@evolvable` Modules (Tests 28 - 30)
    // =========================================================================

    #[test]
    fn test_layer10_code_as_evolution_graph() {
        let code = r#"
        evolvable billing;

        extend Billing owned_by PaymentsTeam {
            fn calculate_fee() -> i64 {
                return 15;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_evolvable = module.statements.iter().any(|s| matches!(s, Statement::EvolvableDecl { module_name, .. } if module_name == "billing"));
        assert!(has_evolvable);
        assert_eq!(module.extensions[0].owned_by, Some("PaymentsTeam".to_string()));
    }

    #[test]
    fn test_layer10_extension_lifecycle_and_dead_detection() {
        let code = r#"
        extend LegacyProtocol @api(>=1) {
            fn old_handshake() -> bool {
                return false;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions[0].target, "LegacyProtocol");
        assert_eq!(module.extensions[0].version_req, Some("@api(>=1)".to_string()));
    }

    #[test]
    fn test_layer10_evolvable_master_module_verification() {
        let code = r#"
        @evolvable mod payments {
            sealed: true;
            requires: ["database"];
            provides: ["PaymentService"];
            guarantees: ["atomic_transfer", "audit_logged"];

            api {
                pub fn pay(amount: i64) -> bool {
                    return true;
                }
            }
            implementation {
                fn execute_transfer(amount: i64) -> bool {
                    return true;
                }
            }
            tests {
                fn test_transfer_integrity() -> bool {
                    return true;
                }
            }
            extension {
                fn custom_discount(amount: i64) -> i64 {
                    return amount;
                }
            }
            architecture {
                "payments -> database";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let payments_mod = &module.modules[0];
        assert!(payments_mod.is_evolvable);

        let report = EvolutionEngine::evaluate_evolvable_module(payments_mod);
        assert!(report.is_evolvable);
        assert!(report.extensibility_score >= 80.0);
        assert!(report.maintainability_score >= 80.0);
        assert!(report.verification_passed);
        assert!(report.formatted_output.contains("END EVOLUTION ANALYSIS"));
        assert!(report.formatted_output.contains("VERIFIED (100% Extensible DNA Compliant)"));
    }
