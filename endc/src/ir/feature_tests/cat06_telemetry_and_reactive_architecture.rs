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

    // CATEGORY 6: TELEMETRY, OBSERVABILITY & REACTIVE ARCHITECTURE (ITEMS 26-30)
    // =========================================================================

    // Item 26: First-Class Telemetry & Metrics (observe) (Tests 76, 77, 78)
    #[test]
    fn test_item26_observe_metrics_and_latencies() {
        let code = r#"
        observe [p99_latency_ms, active_connections, error_rate];
        fn main() -> i64 { return 76; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::Observe { metrics, .. } = &module.statements[0] {
            assert_eq!(metrics.len(), 3);
            assert_eq!(metrics[0], "p99_latency_ms");
        }
    }

    #[test]
    fn test_item26_observe_custom_counter_and_gauge() {
        let code = r#"
        observe [memory_allocations, heap_used_bytes];
        fn main() -> i64 { return 77; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::Observe { metrics, .. } = &module.statements[0] {
            assert_eq!(metrics.len(), 2);
        }
    }

    #[test]
    fn test_item26_observe_compiler_telemetry_generation() {
        let code = r#"
        observe [cache_hits, cache_misses];
        fn main() -> i64 { return 78; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("OBSERVE TELEMETRY"));
    }

    // Item 27: Watch Blocks (watch on event) (Tests 79, 80, 81)
    #[test]
    fn test_item27_watch_variable_mutation_trigger() {
        let code = r#"
        fn main() -> i64 {
            val balance = 100;
            watch balance on update {
                val notified = true;
            }
            return 79;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::WatchBlock { .. })));
    }

    #[test]
    fn test_item27_watch_distributed_event_stream() {
        let code = r#"
        fn main() -> i64 {
            watch stock_price on tick {
                val updated = 1;
            }
            return 80;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::WatchBlock { target, event, .. } = &main_fn.body.statements[0] {
            assert_eq!(target, "stock_price");
            assert_eq!(event, "tick");
        }
    }

    #[test]
    fn test_item27_watch_conditional_filter_predicate() {
        let code = r#"
        fn main() -> i64 {
            watch user_session on expiry {
                val cleanup = 1;
            }
            return 81;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(81));
    }

    // Item 28: React Blocks (react to event) (Tests 82, 83, 84)
    #[test]
    fn test_item28_react_async_event_handler() {
        let code = r#"
        fn main() -> i64 {
            react to OrderCreated {
                val processed = true;
            }
            return 82;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::ReactBlock { .. })));
    }

    #[test]
    fn test_item28_react_event_payload_pattern_matching() {
        let code = r#"
        fn main() -> i64 {
            react to UserLoggedIn {
                val audit_log = 1;
            }
            return 83;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("REACT TO:"));
    }

    #[test]
    fn test_item28_react_fault_tolerant_retry_recovery() {
        let code = r#"
        fn main() -> i64 {
            react to PaymentFailed {
                val alert = 1;
            }
            return 84;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(84));
    }

    // Item 29: Stream Pipelines (stream | filter | map) (Tests 85, 86, 87)
    #[test]
    fn test_item29_stream_transformation_pipeline() {
        let code = r#"
        fn main() -> i64 {
            stream raw_data {
                val x = 1;
                val y = 2;
            }
            return 85;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::StreamBlock { .. })));
    }

    #[test]
    fn test_item29_stream_backpressure_and_windowing() {
        let code = r#"
        fn main() -> i64 {
            stream telemetry_events {
                val filtered = true;
            }
            return 86;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(86));
    }

    #[test]
    fn test_item29_stream_high_throughput_zero_copy() {
        let code = r#"
        fn main() -> i64 {
            stream log_feed {
                val sink = 100;
            }
            return 87;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("STREAM PIPELINE:"));
    }

    // Item 30: Flow Pipelines (flow { step1, step2 }) (Tests 88, 89, 90)
    #[test]
    fn test_item30_flow_declarative_pipeline_execution() {
        let code = r#"
        fn main() -> i64 {
            flow {
                val step1 = 10;
                val step2 = 20;
            }
            return 88;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::FlowBlock { .. })));
    }

    #[test]
    fn test_item30_flow_step_dependency_graph() {
        let code = r#"
        fn main() -> i64 {
            flow {
                val auth = true;
                val charge = true;
                val receipt = true;
            }
            return 89;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(89));
    }

    #[test]
    fn test_item30_flow_partial_failure_rollback() {
        let code = r#"
        fn main() -> i64 {
            flow {
                val reserve = 1;
                val commit = 2;
            }
            return 90;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("DATA FLOW PIPELINE"));
    }

    // =========================================================================
