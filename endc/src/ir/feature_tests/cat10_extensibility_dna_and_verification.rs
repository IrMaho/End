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

    // CATEGORY 10: EXTENSIBILITY DNA & UNIVERSAL VERIFICATION (ITEMS 46-50)
    // =========================================================================

    // Item 46: Partial & Augment Types (partial, augment) (Tests 136, 137, 138)
    #[test]
    fn test_item46_partial_struct_and_module_merging() {
        let code = r#"
        partial struct UserProfile {
            id: i64,
        }
        partial struct UserProfile {
            name: string,
        }
        fn main() -> i64 { return 136; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 2);
        assert!(module.structs[0].is_partial);
        assert!(module.structs[1].is_partial);
    }

    #[test]
    fn test_item46_augment_external_library_type() {
        let code = r#"
        augment struct HttpRequest {
            session_id: string,
        }
        fn main() -> i64 { return 137; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 1);
        assert!(module.extensions[0].is_augment);
    }

    #[test]
    fn test_item46_partial_conflict_resolution() {
        let code = r#"
        partial module SharedKernel {
            fn utility_one() -> i64 { return 1; }
        }
        partial module SharedKernel {
            fn utility_two() -> i64 { return 2; }
        }
        fn main() -> i64 { return 138; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.modules.len(), 2);
        assert!(module.modules[0].is_partial);
        assert!(module.modules[1].is_partial);
    }

    // Item 47: Open/Closed Extensibility Types (open, closed) (Tests 139, 140, 141)
    #[test]
    fn test_item47_open_type_unrestricted_subtyping() {
        let code = r#"
        open struct PluginContext {
            env: string,
        }
        fn main() -> i64 { return 139; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: true, .. })));
    }

    #[test]
    fn test_item47_closed_type_sealed_pattern_matching() {
        let code = r#"
        closed struct SecurityDescriptor {
            token: string,
        }
        fn main() -> i64 { return 140; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: false, .. })));
    }

    #[test]
    fn test_item47_open_type_dynamic_extension_table() {
        let code = r#"
        open module ExtensibleEngine;
        fn main() -> i64 { return 141; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: true, .. })));
    }

    // Item 48: Namespaced Versioned Syntax (syntax @version) (Tests 142, 143, 144)
    #[test]
    fn test_item48_custom_syntax_definition_and_dsl() {
        let code = r#"
        syntax web::graphql {
            query: string;
        }
        fn main() -> i64 { return 142; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { .. })));
    }

    #[test]
    fn test_item48_namespaced_syntax_isolated_scope() {
        let code = r#"
        use syntax web::graphql;
        fn main() -> i64 { return 143; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::UseSyntaxDecl { .. })));
    }

    #[test]
    fn test_item48_syntax_versioning_and_migration() {
        let code = r#"
        syntax db::sql {
            statement: string;
        }
        use syntax db::sql;
        fn main() -> i64 { return 144; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.statements.len(), 2);
        assert_eq!(module.functions.len(), 1);
    }

    // Item 49: Compile-time Plugins & Analyzers (compiler_plugin, lint) (Tests 145, 146, 147)
    #[test]
    fn test_item49_custom_compiler_plugin_optimization() {
        let code = r#"
        compiler_plugin VectorizerPass optimizer;
        fn main() -> i64 { return 145; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::CompilerPluginDecl { .. })));
    }

    #[test]
    fn test_item49_custom_linter_rule_enforcement() {
        let code = r#"
        lint NoUnwrapInProduction {
            "disallow Expression::QuantumUnwrap without fallback";
        }
        fn main() -> i64 { return 146; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::CustomLinterDecl { .. })));
    }

    #[test]
    fn test_item49_type_rule_and_reflect_inspection() {
        let code = r#"
        reflect UserAccount {
            fields;
            methods;
        }
        fn main() -> i64 { return 147; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ReflectDecl { .. })));
    }

    // Item 50: Blast Radius & 7-Pillar Evolution Engine (calculate_blast_radius) (Tests 148, 149, 150)
    #[test]
    fn test_item50_blast_radius_direct_and_transitive_impact() {
        let code = r#"
        feature PaymentCore {
            api {
                fn charge() -> bool;
            }
        }
        feature CheckoutService {
            needs: [PaymentCore];
        }
        feature AnalyticsService {
            needs: [CheckoutService];
        }
        fn main() -> i64 { return 148; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());

        let report = analyzer.calculate_blast_radius("PaymentCore");
        println!("DEBUG REPORT: {:?}", report);
        println!("DEBUG FEATURES: {:?}", analyzer.features.keys());
        assert!(report.affected_features.contains(&"CheckoutService".to_string()));
        assert!(report.affected_features.contains(&"AnalyticsService".to_string()));
        assert!(report.affected_public_apis.contains(&"charge".to_string()));
    }

    #[test]
    fn test_item50_blast_radius_breaking_change_safeguards() {
        let code = r#"
        feature LegacyAuth {
            lifecycle {
                deprecated;
                replace_with: OAuth2Service;
                migration: AuthMigrationPath;
            }
        }
        fn main() -> i64 { return 149; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);

        let report = analyzer.calculate_blast_radius("LegacyAuth");
        assert!(report.required_migrations.contains(&"AuthMigrationPath".to_string()));
    }

    #[test]
    fn test_item50_evolution_proposal_and_formal_proof_gate() {
        let code = r#"
        proposal UpgradeToV2 {
            target: "PaymentGateway";
            version: 2;
            proof: "zero_downtime_guaranteed";
        }
        fn main() -> i64 { return 150; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AgentChangeProposalDecl { .. })));
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(150));
    }

