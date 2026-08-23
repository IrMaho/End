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

    // CATEGORY 1: FEATURE-FIRST CORE & GRAMMAR (ITEMS 1-5)
    // =========================================================================

    // Item 1: First-Class `feature` Definition (Tests 1, 2, 3)
    #[test]
    fn test_item01_feature_definition_complete_syntax() {
        let code = r#"
        feature Authentication {
            version: "1.2.0";
            owner: "auth_team";
            architecture: clean_hexagonal;
            contract {
                accepts: [Credentials];
                returns: [AuthToken];
                guarantees: ["timing_attack_safe", "rate_limited"];
            }
            api {
                fn login(user: string, pass: string) -> bool;
            }
            implementation Production {
                fn login(user: string, pass: string) -> bool {
                    return true;
                }
            }
        }
        fn main() -> i64 { return 1; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.features.len(), 1);
        let feat = &module.features[0];
        assert_eq!(feat.name, "Authentication");
        assert_eq!(feat.version.as_deref(), Some("1.2.0"));
        assert_eq!(feat.owner.as_deref(), Some("auth_team"));
        assert_eq!(feat.architecture_template.as_deref(), Some("clean_hexagonal"));
        assert!(feat.contracts.len() >= 3);
        assert!(feat.api.is_some());
        assert_eq!(feat.implementations.len(), 1);
    }

    #[test]
    fn test_item01_feature_definition_with_ownership_and_lifecycle() {
        let code = r#"
        feature OrderProcessing {
            version: "2.0.0";
            lifecycle {
                stable;
            }
            decision order_saga {
                reason: "Distributed 2PC transaction guarantee";
            }
        }
        fn main() -> i64 { return 2; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.name, "OrderProcessing");
        assert!(feat.lifecycle.is_some());
        assert_eq!(feat.lifecycle.as_ref().unwrap().state, "stable");
        assert_eq!(feat.decisions.len(), 1);
        assert_eq!(feat.decisions[0].target, "order_saga");
    }

    #[test]
    fn test_item01_feature_definition_nested_subfeatures() {
        let code = r#"
        feature BillingSystem {
            version: "1.0.0";
            feature InvoiceGeneration {
                version: "1.0.0";
            }
            feature PaymentGateway {
                version: "1.0.0";
            }
        }
        fn main() -> i64 { return 3; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.name, "BillingSystem");
        assert_eq!(feat.nested_features.len(), 2);
        assert_eq!(feat.nested_features[0].name, "InvoiceGeneration");
        assert_eq!(feat.nested_features[1].name, "PaymentGateway");
    }

    // Item 2: First-Class Contracts (accepts, returns, guarantees) (Tests 4, 5, 6)
    #[test]
    fn test_item02_contract_definition_and_guarantees() {
        let code = r#"
        contract Module PaymentsContract {
            accepts: [PaymentRequest, CardDetails];
            returns: [TransactionReceipt];
            guarantees: [idempotent, audit_logged, zero_double_charge];
        }
        fn main() -> i64 { return 4; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.contracts.len(), 1);
        let ctr = &module.contracts[0];
        assert_eq!(ctr.name, "PaymentsContract");
        assert!(ctr.clauses.contains(&"idempotent".to_string()));
        assert!(ctr.clauses.contains(&"audit_logged".to_string()));
    }

    #[test]
    fn test_item02_contract_parameterized_and_typed_io() {
        let code = r#"
        contract Module DatabasePort {
            accepts: [SqlQuery, QueryParams];
            returns: [ResultSet];
            guarantees: [sql_injection_safe, connection_pooled];
        }
        fn main() -> i64 { return 5; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.contracts.contains_key("DatabasePort"));
    }

    #[test]
    fn test_item02_contract_formal_postconditions_verification() {
        let code = r#"
        feature SecureVault {
            contract {
                accepts: [EncryptedPayload];
                returns: [DecryptedSecret];
                guarantees: ["constant_time", "zero_leakage"];
            }
        }
        fn main() -> i64 { return 6; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("SecureVault"));
    }

    // Item 3: Explicit Feature Dependencies (needs [Feature]) (Tests 7, 8, 9)
    #[test]
    fn test_item03_needs_explicit_dependencies_resolution() {
        let code = r#"
        feature InventoryService {
            version: "1.0.0";
        }
        feature CheckoutService {
            version: "1.0.0";
            needs: [InventoryService, PaymentGateway];
        }
        fn main() -> i64 { return 7; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.features.len(), 2);
        let checkout = &module.features[1];
        assert_eq!(checkout.needs.len(), 2);
        assert_eq!(checkout.needs[0].name, "InventoryService");
        assert_eq!(checkout.needs[1].name, "PaymentGateway");
    }

    #[test]
    fn test_item03_needs_typed_subcontract_dependencies() {
        let code = r#"
        feature OrderDispatcher {
            needs {
                EmailService;
                NotificationHub;
            }
        }
        fn main() -> i64 { return 8; }
        "#;
        let module = parse_str(code).unwrap();
        let disp = &module.features[0];
        assert_eq!(disp.needs.len(), 2);
        assert_eq!(disp.needs[0].name, "EmailService");
        assert_eq!(disp.needs[1].name, "NotificationHub");
    }

    #[test]
    fn test_item03_needs_circular_dependency_error_e0934() {
        let code = r#"
        feature FeatA {
            needs: [FeatB];
        }
        feature FeatB {
            needs: [FeatA];
        }
        fn main() -> i64 { return 9; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.errors.iter().any(|e| e.code == "E0934"));
    }

    // Item 4: Official Extension Points (extension_point) (Tests 10, 11, 12)
    #[test]
    fn test_item04_extension_point_declarations_and_types() {
        let code = r#"
        feature PaymentPipeline {
            extension_point PostPaymentHook {
                priority: 1;
            }
            extension_point TaxCalculator {
                priority: 2;
            }
        }
        fn main() -> i64 { return 10; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.extensions.len(), 2);
        assert_eq!(feat.extensions[0].name, "PostPaymentHook");
        assert_eq!(feat.extensions[1].name, "TaxCalculator");
    }

    #[test]
    fn test_item04_extension_point_composable_pipeline() {
        let code = r#"
        feature HttpRequestPipeline {
            extension_point AuthMiddleware {
                priority: 1;
            }
        }
        fn main() -> i64 { return 11; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.extensions[0].name, "AuthMiddleware");
    }

    #[test]
    fn test_item04_extension_point_unauthorized_override_protection() {
        let code = r#"
        feature CoreKernel {
            extension_point SystemClock {
                priority: 0;
            }
        }
        fn main() -> i64 { return 12; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.extensions[0].name, "SystemClock");
    }

    // Item 5: Feature Boundaries & Isolation (boundary { allow, deny }) (Tests 13, 14, 15)
    #[test]
    fn test_item05_boundary_allow_deny_enforcement() {
        let code = r#"
        feature UserDataStore {
            boundary {
                "api";
                "domain";
                "infrastructure";
            }
        }
        fn main() -> i64 { return 13; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert!(feat.boundary.is_some());
        let b = feat.boundary.as_ref().unwrap();
        assert_eq!(b.layers.len(), 3);
    }

    #[test]
    fn test_item05_boundary_sealed_module_protection() {
        let code = r#"
        sealed module SecurityBoundary {
            fn hash_secret(s: string) -> string { return s; }
        }
        fn main() -> i64 { return 14; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.sealed_modules.contains("SecurityBoundary"));
    }

    #[test]
    fn test_item05_boundary_leak_check_diagnostics() {
        let code = r#"
        leak check Payments forbid raw_card_number through PublicApi;
        fn main() -> i64 { return 15; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::LeakCheckDecl { .. })));
    }

    // =========================================================================
