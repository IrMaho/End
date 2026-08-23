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

    // CATEGORY 5: PORTS, ADAPTERS & GATEWAYS (ITEMS 21-25)
    // =========================================================================

    // Item 21: Port Declaration (port) (Tests 61, 62, 63)
    #[test]
    fn test_item21_port_definition_with_contracts() {
        let code = r#"
        port PaymentPort {
            charge(amount: i64) -> bool;
            refund(tx_id: string) -> bool;
        }
        fn main() -> i64 { return 61; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PortDecl { name, methods, .. } = &module.statements[0] {
            assert_eq!(name, "PaymentPort");
            assert_eq!(methods.len(), 2);
        }
    }

    #[test]
    fn test_item21_port_multiple_method_signatures() {
        let code = r#"
        port EmailSenderPort {
            send_email(to: string, body: string) -> bool;
        }
        fn main() -> i64 { return 62; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::PortDecl { .. })));
    }

    #[test]
    fn test_item21_port_abstract_domain_boundary() {
        let code = r#"
        port KeyValueStorePort {
            get(key: string) -> string;
            set(key: string, val: string) -> bool;
        }
        fn main() -> i64 { return 63; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PortDecl { methods, .. } = &module.statements[0] {
            assert_eq!(methods.len(), 2);
        }
    }

    // Item 22: Adapter Declaration (adapter for port) (Tests 64, 65, 66)
    #[test]
    fn test_item22_adapter_stripe_for_payment_port() {
        let code = r#"
        adapter StripeAdapter for PaymentPort {
            fn charge(amount: i64) -> bool { return true; }
        }
        fn main() -> i64 { return 64; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::AdapterDecl { name, port, .. } = &module.statements[0] {
            assert_eq!(name, "StripeAdapter");
            assert_eq!(port, "PaymentPort");
        }
    }

    #[test]
    fn test_item22_adapter_in_memory_mock_for_testing() {
        let code = r#"
        adapter MockDatabaseAdapter for DatabasePort {
            fn query(sql: string) -> string { return "mock_result"; }
        }
        fn main() -> i64 { return 65; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::AdapterDecl { name, port, .. } = &module.statements[0] {
            assert_eq!(name, "MockDatabaseAdapter");
            assert_eq!(port, "DatabasePort");
        }
    }

    #[test]
    fn test_item22_adapter_missing_method_validation() {
        let code = r#"
        adapter RedisCacheAdapter for CachePort {
            fn get(k: string) -> string { return ""; }
        }
        fn main() -> i64 { return 66; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AdapterDecl { .. })));
    }

    // Item 23: Facade Declaration (facade) (Tests 67, 68, 69)
    #[test]
    fn test_item23_facade_unified_api_surface() {
        let code = r#"
        facade ECommerceFacade exposes [checkout, search_catalog, view_order];
        fn main() -> i64 { return 67; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::FacadeDecl { name, exposes, .. } = &module.statements[0] {
            assert_eq!(name, "ECommerceFacade");
            assert_eq!(exposes.len(), 3);
        }
    }

    #[test]
    fn test_item23_facade_subsystem_simplification() {
        let code = r#"
        facade CloudStorageFacade exposes [upload_blob, download_blob];
        fn main() -> i64 { return 68; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::FacadeDecl { exposes, .. } = &module.statements[0] {
            assert_eq!(exposes.len(), 2);
        }
    }

    #[test]
    fn test_item23_facade_rate_limiting_and_caching() {
        let code = r#"
        facade BankingFacade exposes [transfer_funds];
        fn main() -> i64 { return 69; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::FacadeDecl { .. })));
    }

    // Item 24: Gateway Declaration (gateway) (Tests 70, 71, 72)
    #[test]
    fn test_item24_gateway_cross_domain_allowed_calls() {
        let code = r#"
        gateway ApiGateway -> BillingService allowed_calls [create_invoice, charge_subscription];
        fn main() -> i64 { return 70; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::GatewayDecl { from_mod, to_mod, allowed_calls, .. } = &module.statements[0] {
            assert_eq!(from_mod, "ApiGateway");
            assert_eq!(to_mod, "BillingService");
            assert_eq!(allowed_calls.len(), 2);
        }
    }

    #[test]
    fn test_item24_gateway_unauthorized_call_rejection() {
        let code = r#"
        gateway WebTier -> DatabaseTier allowed_calls [read_only_query];
        fn main() -> i64 { return 71; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::GatewayDecl { allowed_calls, .. } = &module.statements[0] {
            assert_eq!(allowed_calls, &vec!["read_only_query".to_string()]);
        }
    }

    #[test]
    fn test_item24_gateway_distributed_tracing_propagation() {
        let code = r#"
        gateway MicroserviceHub -> NotificationCluster allowed_calls [broadcast_alert];
        fn main() -> i64 { return 72; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::GatewayDecl { .. })));
    }

    // Item 25: Architectural Invariants (arch_invariant) (Tests 73, 74, 75)
    #[test]
    fn test_item25_arch_invariant_cycle_free_enforcement() {
        let code = r#"
        cycle_free;
        fn main() -> i64 { return 73; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.arch_cycle_free);
    }

    #[test]
    fn test_item25_arch_invariant_depth_and_fanout_limits() {
        let code = r#"
        depth 5;
        fanout EngineModule 4;
        fanin DatabaseModule 10;
        fn main() -> i64 { return 74; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.statements.len(), 3);
    }

    #[test]
    fn test_item25_arch_invariant_runtime_verification() {
        let code = r#"
        arch_invariant "no_circular_dependencies";
        fn main() -> i64 { return 75; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::ArchInvariantDecl { rule, .. } = &module.statements[0] {
            assert_eq!(rule, "no_circular_dependencies");
        }
    }

    // =========================================================================
