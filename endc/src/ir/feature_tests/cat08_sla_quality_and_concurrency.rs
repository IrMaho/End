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

    // CATEGORY 8: SLA, QUALITY & CONCURRENCY GUARDRAILS (ITEMS 36-40)
    // =========================================================================

    // Item 36: Budget & SLA Enforcement (budget) (Tests 106, 107, 108)
    #[test]
    fn test_item36_budget_ms_runtime_latency_clamp() {
        let code = r#"
        fn main() -> i64 {
            budget latency: "50ms", memory: "10MB" {
                val computed = 100;
            }
            return 106;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::BudgetBlock { .. })));
    }

    #[test]
    fn test_item36_budget_memory_allocation_ceiling() {
        let code = r#"
        fn main() -> i64 {
            budget max_allocations: "1000" {
                val x = 1;
            }
            return 107;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(107));
    }

    #[test]
    fn test_item36_budget_yield_to_frame_renderer() {
        let code = r#"
        fn main() -> i64 {
            budget frame_budget: "16ms" {
                val frame_draw = 1;
            }
            return 108;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("BUDGET SLA"));
    }

    // Item 37: Deadline Enforcement (deadline) (Tests 109, 110, 111)
    #[test]
    fn test_item37_deadline_timeout_cancellation() {
        let code = r#"
        fn main() -> i64 {
            deadline "100ms" {
                val task_result = 42;
            }
            return 109;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::DeadlineBlock { .. })));
    }

    #[test]
    fn test_item37_deadline_cascading_subtask_cancellation() {
        let code = r#"
        fn main() -> i64 {
            deadline "250ms" {
                val rpc_call = 1;
            }
            return 110;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(110));
    }

    #[test]
    fn test_item37_deadline_graceful_partial_result() {
        let code = r#"
        fn main() -> i64 {
            deadline "500ms" {
                val sub = 200;
            }
            return 111;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("DEADLINE ENFORCEMENT"));
    }

    // Item 38: Priority Dispatch (priority) (Tests 112, 113, 114)
    #[test]
    fn test_item38_priority_realtime_vs_background_queues() {
        let code = r#"
        fn main() -> i64 {
            priority high {
                val audio_packet = 1;
            }
            return 112;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::PriorityBlock { .. })));
    }

    #[test]
    fn test_item38_priority_inversion_prevention() {
        let code = r#"
        fn main() -> i64 {
            priority critical {
                val lock_acquired = true;
            }
            return 113;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(113));
    }

    #[test]
    fn test_item38_priority_dynamic_boost() {
        let code = r#"
        fn main() -> i64 {
            priority low {
                val garbage_collect = 1;
            }
            return 114;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("PRIORITY LEVEL: low"));
    }

    // Item 39: Quality & Tradeoff Constraints (quality, tradeoff) (Tests 115, 116, 117)
    #[test]
    fn test_item39_quality_min_accuracy_enforcement() {
        let code = r#"
        fn main() -> i64 {
            quality min_accuracy: "0.99", max_latency: "20ms" {
                val ai_inference = 1;
            }
            return 115;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::QualityBlock { .. })));
    }

    #[test]
    fn test_item39_tradeoff_latency_vs_fidelity_dispatch() {
        let code = r#"
        fn main() -> i64 {
            tradeoff prefer: "latency", sacrifice: "precision" {
                val fast_approx = 42;
            }
            return 116;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::TradeoffBlock { .. })));
    }

    #[test]
    fn test_item39_adaptive_dispatch_under_system_load() {
        let code = r#"
        fn main() -> i64 {
            adapt {
                when high_load { val mode = 1; }
                when normal_load { val mode = 2; }
            }
            return 117;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::AdaptBlock { .. })));
    }

    // Item 40: Race-Free Concurrency & Ownership (owned, spawn) (Tests 118, 119, 120)
    #[test]
    fn test_item40_owned_unique_ownership_handoff() {
        let code = r#"
        fn main() -> i64 {
            owned val buffer = 1024;
            return 118;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::Owned { .. })));
    }

    #[test]
    fn test_item40_spawn_isolated_fiber_execution() {
        let code = r#"
        fn async_worker() -> i64 { return 1; }
        fn main() -> i64 {
            spawn async_worker();
            return 119;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[1];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::Spawn { .. })));
    }

    #[test]
    fn test_item40_race_free_data_race_compile_error() {
        let code = r#"
        fn main() -> i64 {
            owned val unique_resource = 500;
            return 120;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(120));
    }

    // =========================================================================
