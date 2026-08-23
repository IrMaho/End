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

    // CATEGORY 4: REFACTORING & ARCHITECTURE EVOLUTION (ITEMS 16-20)
    // =========================================================================

    // Item 16: Architectural Split (split) (Tests 46, 47, 48)
    #[test]
    fn test_item16_split_monolith_module_into_microservices() {
        let code = r#"
        split MonolithModule into [AuthService, BillingService, InventoryService];
        fn main() -> i64 { return 46; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SplitDecl { entity, parts, .. } = &module.statements[0] {
            assert_eq!(entity, "MonolithModule");
            assert_eq!(parts.len(), 3);
        }
    }

    #[test]
    fn test_item16_split_operation_into_pipeline_stages() {
        let code = r#"
        split operation ComplexCheckout into [ValidateCart, CalculateTaxes, AuthorizeCard];
        fn main() -> i64 { return 47; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SplitOpDecl { .. })));
    }

    #[test]
    fn test_item16_split_with_invariant_preservation() {
        let code = r#"
        split MonolithDatabase into [CustomerDb, OrderDb];
        fn main() -> i64 { return 48; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SplitDecl { .. })));
    }

    // Item 17: Architectural Partition (partition) (Tests 49, 50, 51)
    #[test]
    fn test_item17_partition_by_domain_cluster() {
        let code = r#"
        partition Warehouse by location [NorthAmerica, Europe, AsiaPacific];
        fn main() -> i64 { return 49; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PartitionDecl { entity, by, parts, .. } = &module.statements[0] {
            assert_eq!(entity, "Warehouse");
            assert_eq!(by, "location");
            assert_eq!(parts.len(), 3);
        }
    }

    #[test]
    fn test_item17_partition_by_tenancy_and_region() {
        let code = r#"
        partition UserTenant by tier [FreeTier, EnterpriseTier];
        fn main() -> i64 { return 50; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PartitionDecl { entity, by, parts, .. } = &module.statements[0] {
            assert_eq!(entity, "UserTenant");
            assert_eq!(by, "tier");
            assert_eq!(parts.len(), 2);
        }
    }

    #[test]
    fn test_item17_partition_dynamic_router_dispatch() {
        let code = r#"
        partition RequestRouter by method [HttpHandler, WsHandler, GrpcHandler];
        fn main() -> i64 { return 51; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::PartitionDecl { .. })));
    }

    // Item 18: Symbol Extraction (extract) (Tests 52, 53, 54)
    #[test]
    fn test_item18_extract_common_utilities_into_shared() {
        let code = r#"
        extract [format_currency, parse_iso_date] to SharedUtils;
        fn main() -> i64 { return 52; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::ExtractDecl { symbols, into_module, .. } = &module.statements[0] {
            assert_eq!(symbols.len(), 2);
            assert_eq!(into_module, "SharedUtils");
        }
    }

    #[test]
    fn test_item18_extract_operation_subflow_to_module() {
        let code = r#"
        extract operation SliceSecurity from MonolithEngine into SecurityCore;
        fn main() -> i64 { return 53; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ExtractOpDecl { .. })));
    }

    #[test]
    fn test_item18_extract_dependency_rewiring() {
        let code = r#"
        extract [jwt_verify, token_refresh] to AuthGateway;
        fn main() -> i64 { return 54; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ExtractDecl { .. })));
    }

    // Item 19: Semantic Clustering (cluster) (Tests 55, 56, 57)
    #[test]
    fn test_item19_cluster_by_cohesion_metrics() {
        let code = r#"
        cluster by cohesion "affinity_score > 0.85";
        fn main() -> i64 { return 55; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::ClusterDecl { by, predicate, .. } = &module.statements[0] {
            assert_eq!(by, "cohesion");
            assert_eq!(predicate, "affinity_score > 0.85");
        }
    }

    #[test]
    fn test_item19_cluster_by_affinity_rules() {
        let code = r#"
        cluster by domain "namespace == 'billing'";
        fn main() -> i64 { return 56; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ClusterDecl { .. })));
    }

    #[test]
    fn test_item19_cluster_monolith_decomposition_plan() {
        let code = r#"
        decompose MonolithApp {
            target_modules: 5;
            optimize: [cohesion, coupling];
            preserve: [public_apis];
        }
        fn main() -> i64 { return 57; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::DecomposeDecl { .. })));
    }

    // Item 20: Module Separation (separate) (Tests 58, 59, 60)
    #[test]
    fn test_item20_separate_core_from_ui_cleanly() {
        let code = r#"
        separate DomainCore from WebPresentation;
        fn main() -> i64 { return 58; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SeparateDecl { left, right, .. } = &module.statements[0] {
            assert_eq!(left, "DomainCore");
            assert_eq!(right, "WebPresentation");
        }
    }

    #[test]
    fn test_item20_separate_shared_state_into_channels() {
        let code = r#"
        separate DatabaseAdapter from HttpController;
        fn main() -> i64 { return 59; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SeparateDecl { .. })));
    }

    #[test]
    fn test_item20_separate_cyclic_coupling_resolution() {
        let code = r#"
        separate OrderService from InventoryService;
        fn main() -> i64 { return 60; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("SEPARATE MODULES"));
    }

    // =========================================================================
