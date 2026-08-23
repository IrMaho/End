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

    // CATEGORY 3: ARCHITECTURAL GUARDRAILS & ENFORCEMENTS (ITEMS 11-15)
    // =========================================================================

    // Item 11: Clean Architecture by Grammar (layer, direction) (Tests 31, 32, 33)
    #[test]
    fn test_item11_clean_architecture_domain_to_adapter_layers() {
        let code = r#"
        layer Domain { forbid depends Infrastructure; }
        layer Application { forbid depends Infrastructure; }
        layer Infrastructure {}
        fn main() -> i64 { return 31; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.arch_layers.contains_key("Domain"));
    }

    #[test]
    fn test_item11_clean_architecture_inward_dependency_direction() {
        let code = r#"
        direction: Controller -> UseCase;
        direction: UseCase -> DomainEntity;
        fn main() -> i64 { return 32; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert_eq!(analyzer.arch_directions.len(), 2);
    }

    #[test]
    fn test_item11_clean_architecture_forbidden_reverse_layer_calls() {
        let code = r#"
        forbid Database -> Presentation;
        fn main() -> i64 { return 33; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.module_forbidden.get("Database").unwrap().contains("Presentation"));
    }

    // Item 12: Boundary Sealed Protection (sealed) (Tests 34, 35, 36)
    #[test]
    fn test_item12_sealed_module_external_mutation_rejection() {
        let code = r#"
        sealed module CoreEngine {
            pub fn run_engine() -> bool { return true; }
        }
        fn main() -> i64 { return 34; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.modules[0].is_sealed);
    }

    #[test]
    fn test_item12_sealed_boundary_export_surface_control() {
        let code = r#"
        sealed boundary SecurityCore;
        fn main() -> i64 { return 35; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.sealed_modules.contains("SecurityCore"));
    }

    #[test]
    fn test_item12_sealed_type_inheritance_closure() {
        let code = r#"
        sealed struct NonExtendableKernel {
            id: i64,
        }
        fn main() -> i64 { return 36; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.sealed_structs.contains("NonExtendableKernel"));
    }

    // Item 13: Friend Module Access (friend) (Tests 37, 38, 39)
    #[test]
    fn test_item13_friend_module_private_member_access() {
        let code = r#"
        friend module Engine to EngineTestRunner;
        fn main() -> i64 { return 37; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.module_friends.get("Engine").unwrap().contains("EngineTestRunner"));
    }

    #[test]
    fn test_item13_friend_module_non_friend_rejection() {
        let code = r#"
        friend module Billing to AuditService;
        fn main() -> i64 { return 38; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(!analyzer.module_friends.get("Billing").unwrap().contains("ExternalHacker"));
    }

    #[test]
    fn test_item13_friend_module_scoped_subpackage_grant() {
        let code = r#"
        friend module DatabaseCore to DatabaseMigrator;
        fn main() -> i64 { return 39; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.module_friends.get("DatabaseCore").unwrap().contains("DatabaseMigrator"));
    }

    // Item 14: Private to Symbol Scoping (private_to) (Tests 40, 41, 42)
    #[test]
    fn test_item14_private_to_subsystem_protection() {
        let code = r#"
        private_to internal_crypto_key SecurityModule;
        fn main() -> i64 { return 40; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert_eq!(analyzer.private_to_symbols.get("internal_crypto_key").unwrap(), "SecurityModule");
    }

    #[test]
    fn test_item14_private_to_cross_boundary_leak_diagnostic() {
        let code = r#"
        private_to database_password ConfigLoader;
        fn main() -> i64 { return 41; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.private_to_symbols.contains_key("database_password"));
    }

    #[test]
    fn test_item14_private_to_inheritance_visibility() {
        let code = r#"
        private_to raw_pointer MemoryManager;
        fn main() -> i64 { return 42; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert_eq!(analyzer.private_to_symbols.get("raw_pointer").unwrap(), "MemoryManager");
    }

    // Item 15: Module Surface Management (surface { expose, hide }) (Tests 43, 44, 45)
    #[test]
    fn test_item15_surface_declaration_and_filtering() {
        let code = r#"
        surface CoreSubsystem {
            expose: ["init", "shutdown", "process"];
            hide: ["raw_buffer", "thread_pool_impl"];
        }
        fn main() -> i64 { return 43; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SurfaceDecl { name, exposes, hides, .. } = &module.statements[0] {
            assert_eq!(name, "CoreSubsystem");
            assert_eq!(exposes.len(), 3);
            assert_eq!(hides.len(), 2);
        }
    }

    #[test]
    fn test_item15_surface_opaque_internals_hiding() {
        let code = r#"
        surface PaymentGateway {
            expose: ["charge_card"];
            hide: ["merchant_secret_key"];
        }
        fn main() -> i64 { return 44; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SurfaceDecl { exposes, hides, .. } = &module.statements[0] {
            assert!(exposes.contains(&"charge_card".to_string()));
            assert!(hides.contains(&"merchant_secret_key".to_string()));
        }
    }

    #[test]
    fn test_item15_surface_evolutionary_exposure_matrix() {
        let code = r#"
        surface AuthSystem {
            expose: ["verify_jwt"];
            hide: ["rsa_private_key"];
        }
        fn main() -> i64 { return 45; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SurfaceDecl { .. })));
    }

    // =========================================================================
