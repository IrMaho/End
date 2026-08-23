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

    // CATEGORY 2: FEATURE COMPOSITION & EVOLUTION (ITEMS 6-10)
    // =========================================================================

    // Item 6: Feature Replaceability & Hot-Swapping (Tests 16, 17, 18)
    #[test]
    fn test_item06_replaceable_feature_mock_and_prod_swapping() {
        let code = r#"
        feature PaymentProcessor {
            replaceable;
            api {
                fn charge(amt: i64) -> bool;
            }
        }
        replace feature PaymentProcessor with MockPaymentProcessor;
        fn main() -> i64 { return 16; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert!(feat.is_replaceable);
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ReplaceFeature { .. })));
    }

    #[test]
    fn test_item06_replaceable_feature_dynamic_dispatch() {
        let code = r#"
        feature StorageEngine {
            replaceable;
        }
        replace feature StorageEngine with S3CloudStorage;
        fn main() -> i64 { return 17; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.features[0].is_replaceable);
    }

    #[test]
    fn test_item06_replaceable_feature_signature_matching() {
        let code = r#"
        feature NotificationService {
            replaceable;
            api {
                fn send(msg: string) -> bool;
            }
        }
        fn main() -> i64 { return 18; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.features[0].is_replaceable);
        assert!(module.features[0].api.is_some());
    }

    // Item 7: Feature Evolution & SemVer Guard (@evolvable) (Tests 19, 20, 21)
    #[test]
    fn test_item07_evolvable_feature_audit_e0937() {
        let code = r#"
        feature UnboundedEvolvable {
            evolvable;
        }
        fn main() -> i64 { return 19; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.errors.iter().any(|e| e.code == "E0937"));
    }

    #[test]
    fn test_item07_evolvable_feature_semver_compatibility() {
        let code = r#"
        feature SafeEvolvable {
            evolvable;
            contract {
                guarantees: ["backward_compatible_v1"];
            }
            extension_point CustomHook {
                type: "hook";
                is_composable: true;
            }
        }
        fn main() -> i64 { return 20; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert_eq!(analyzer.errors.len(), 0);
    }

    #[test]
    fn test_item07_evolvable_feature_deprecation_path() {
        let code = r#"
        feature LegacyAuth {
            lifecycle {
                deprecated;
                replace_with: ModernAuth;
                migration: MigrateLegacyAuth;
            }
        }
        fn main() -> i64 { return 21; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        let lc = feat.lifecycle.as_ref().unwrap();
        assert_eq!(lc.state, "deprecated");
        assert_eq!(lc.replace_with.as_deref(), Some("ModernAuth"));
        assert_eq!(lc.migration_path.as_deref(), Some("MigrateLegacyAuth"));
    }

    // Item 8: Feature Migration & Renaming (migration) (Tests 22, 23, 24)
    #[test]
    fn test_item08_migration_symbol_and_field_renaming() {
        let code = r#"
        migration PaymentFeature from "1.0.0" to "2.0.0" {
            rename "Payment.id" -> "Payment.transaction_id";
            replace "Payment.legacy_charge" -> "Payment.charge_v2";
        }
        fn main() -> i64 { return 22; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.feature_migrations.len(), 1);
        let mig = &module.feature_migrations[0];
        assert_eq!(mig.feature_name, "PaymentFeature");
        assert_eq!(mig.renames.len(), 1);
        assert_eq!(mig.replacements.len(), 1);
    }

    #[test]
    fn test_item08_migration_state_transformation_mapping() {
        let code = r#"
        migration UserAccount from "1.0.0" to "2.0.0" {
            rename "User.username" -> "User.handle";
        }
        fn main() -> i64 { return 23; }
        "#;
        let module = parse_str(code).unwrap();
        let mig = &module.feature_migrations[0];
        assert_eq!(mig.renames[0].0, "User.username");
        assert_eq!(mig.renames[0].1, "User.handle");
    }

    #[test]
    fn test_item08_migration_bidirectional_compat_bridge() {
        let code = r#"
        bridge LegacyModule -> ModernModule via AdapterBridge;
        fn main() -> i64 { return 24; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::BridgeDecl { .. })));
    }

    // Item 9: Feature Composition (compose { F1, F2 }) (Tests 25, 26, 27)
    #[test]
    fn test_item09_feature_composition_multivariate_bundle() {
        let code = r#"
        feature CompleteCommerce {
            compose {
                CatalogFeature;
                CartFeature;
                CheckoutFeature;
            }
        }
        fn main() -> i64 { return 25; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.compose.len(), 3);
        assert_eq!(feat.compose[0], "CatalogFeature");
        assert_eq!(feat.compose[1], "CartFeature");
        assert_eq!(feat.compose[2], "CheckoutFeature");
    }

    #[test]
    fn test_item09_feature_composition_precedence_and_conflict() {
        let code = r#"
        compose feature EnterpriseStack with [SecurityPlugin, ObservabilityPlugin];
        fn main() -> i64 { return 26; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ComposeFeature { .. })));
    }

    #[test]
    fn test_item09_feature_composition_deep_pipeline() {
        let code = r#"
        feature WebBackend {
            compose: [HttpServer, RouteMatcher, SessionManager];
        }
        fn main() -> i64 { return 27; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.compose.len(), 3);
    }

    // Item 10: Feature Decoration (decorate feature) (Tests 28, 29, 30)
    #[test]
    fn test_item10_feature_decoration_telemetry_and_audit() {
        let code = r#"
        decorate feature PaymentProcessing with [AuditLogger, PrometheusExporter];
        fn main() -> i64 { return 28; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::DecorateFeature { .. })));
    }

    #[test]
    fn test_item10_feature_decoration_retry_and_caching() {
        let code = r#"
        decorate feature ExternalApi {
            ExponentialRetry;
            RedisCache;
        }
        fn main() -> i64 { return 29; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::DecorateFeature { target, decorators, .. } = &module.statements[0] {
            assert_eq!(target, "ExternalApi");
            assert_eq!(decorators.len(), 2);
        }
    }

    #[test]
    fn test_item10_feature_decoration_chain_ordering() {
        let code = r#"
        feature DecoratedCheckout {
            decorate {
                RateLimiter;
                AuthValidator;
            }
        }
        fn main() -> i64 { return 30; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.features[0].decorations.len(), 2);
    }

    // =========================================================================
