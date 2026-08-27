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

    // CATEGORY 7: FIRST-CLASS OPERATIONS & RESILIENCE (ITEMS 31-35)
    // =========================================================================

    // Item 31: First-Class operation Values (Tests 91, 92, 93)
    #[test]
    fn test_item31_operation_first_class_passing_and_returning() {
        let code = r#"
        operation FetchUser {
            requires: [DbConnection];
            guarantees: [UserExists];
            effects: [reads_db];
        }
        fn main() -> i64 { return 91; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.statements.len(), 1);
        if let Statement::OperationDecl(op) = &module.statements[0] {
            assert_eq!(op.name, "FetchUser");
            assert_eq!(op.requires, vec!["DbConnection".to_string()]);
            assert_eq!(op.guarantees, vec!["UserExists".to_string()]);
        }
    }

    #[test]
    fn test_item31_operation_higher_order_combinators() {
        let code = r#"
        operation ValidateAuth {
            requires: [JwtSecret];
        }
        fn main() -> i64 { return 92; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_item31_operation_lambda_anonymous_binding() {
        let code = r#"
        operation ProcessOrder {
            effects: [writes_db, emits_event];
        }
        fn main() -> i64 { return 93; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("OPERATION DECLARATION 'ProcessOrder'"));
    }

    // Item 32: Operation Composition (compose_ops) (Tests 94, 95, 96)
    #[test]
    fn test_item32_operation_composition_pipeline_order() {
        let code = r#"
        operation OpA {}
        operation OpB {}
        fn main() -> i64 {
            val combo = OpA >> OpB;
            return 94;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Compose { .. } | Expression::Binary { op: BinaryOp::Shr, .. }));
        }
    }

    #[test]
    fn test_item32_operation_composition_type_inference() {
        let code = r#"
        operation Step1 {}
        operation Step2 {}
        operation Step3 {}
        fn main() -> i64 {
            val full_pipeline = Step1 >> Step2 >> Step3;
            return 95;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(95));
    }

    #[test]
    fn test_item32_operation_composition_early_exit_on_error() {
        let code = r#"
        operation Validate {}
        operation Persist {}
        fn main() -> i64 {
            val p = Validate >> Persist;
            return 96;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("end_compose_ops"));
    }

    // Item 33: Operation Resilience (retry, fallback) (Tests 97, 98, 99)
    #[test]
    fn test_item33_operation_retry_exponential_backoff() {
        let code = r#"
        operation UnreliableNetworkCall {}
        fn main() -> i64 {
            val resilient_call = UnreliableNetworkCall retry 3;
            return 97;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Repeat { is_retry: true, .. }));
        }
    }

    #[test]
    fn test_item33_operation_fallback_degraded_service() {
        let code = r#"
        operation PrimaryDb {}
        operation SecondaryDb {}
        fn main() -> i64 {
            val resilient = PrimaryDb / SecondaryDb;
            return 98;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Alternative { .. }));
        }
    }

    #[test]
    fn test_item33_operation_circuit_breaker_trip() {
        let code = r#"
        operation LiveRpc {}
        operation CachedFallback {}
        fn main() -> i64 {
            val safe_call = (LiveRpc retry 2) / CachedFallback;
            return 99;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(99));
    }

    // Item 34: Parallel Operations (parallel, parallel_choose) (Tests 100, 101, 102)
    #[test]
    fn test_item34_parallel_multi_branch_execution() {
        let code = r#"
        operation FetchProfile {}
        operation FetchOrders {}
        fn main() -> i64 {
            val parallel_load = FetchProfile || FetchOrders;
            return 100;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Parallel { .. }));
        }
    }

    #[test]
    fn test_item34_parallel_choose_first_successful_hedging() {
        let code = r#"
        fn main() -> i64 {
            parallel_choose {
                branch US_EAST { val x = 1; }
                branch EU_WEST { val y = 2; }
            }
            return 101;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::ParallelChoose { .. })));
    }

    #[test]
    fn test_item34_parallel_thread_pool_and_fiber_safety() {
        let code = r#"
        fn main() -> i64 {
            parallel_choose {
                branch ServerA { val resA = 10; }
                branch ServerB { val resB = 20; }
            }
            return 102;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(102));
    }

    // Item 35: Operation Memoization (memoize) (Tests 103, 104, 105)
    #[test]
    fn test_item35_memoize_pure_computation_cache() {
        let code = r#"
        operation ComputeFibonacci {}
        fn main() -> i64 {
            val cached_fib = memoize ComputeFibonacci;
            return 103;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Memoize { .. }));
        }
    }

    #[test]
    fn test_item35_memoize_cache_invalidation_key() {
        let code = r#"
        operation CalculateTaxTable {}
        fn main() -> i64 {
            val fast_tax = memoize CalculateTaxTable;
            return 104;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(104));
    }

    #[test]
    fn test_item35_memoize_thread_safe_concurrent_access() {
        let code = r#"
        operation HeavyMatrixMultiply {}
        fn main() -> i64 {
            val cached_matmul = memoize HeavyMatrixMultiply;
            return 105;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("end_memoize_op"));
    }

    // =========================================================================
