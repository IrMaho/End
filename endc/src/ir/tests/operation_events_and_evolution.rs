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

    #[test]
    fn test_operation_higher_order_returning_and_passing_closures() {
        let code = r#"
        operation base_calc(x: i64) -> i64 {
            return x * 10;
        }

        fn apply_op(op: Operation, val_in: i64) -> i64 {
            val res = op(val_in);
            return res.output;
        }

        fn build_multiplier() -> Operation {
            val anon = operation(n: i64) {
                return n * 5;
            };
            return anon;
        }

        fn main() -> i64 {
            val op1 = build_multiplier();
            val r1 = apply_op(op1, 8); // 8 * 5 = 40
            val r2 = apply_op(base_calc, 4); // 4 * 10 = 40
            return r1 + r2; // 40 + 40 = 80
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(80), "Higher-order operation passing and returning must produce 80");
    }

    #[test]
    fn test_operation_resilient_retry_fallback_and_conditional_when() {
        let code = r#"
        operation safe_step(x: i64) -> i64 {
            return x + 100;
        }

        operation fallback_step(x: i64) -> i64 {
            return 999;
        }

        fn main() -> i64 {
            // Test retry
            val retried = retry safe_step up to 5;
            val r_retry = retried(10);

            // Test fallback
            val alt_step = safe_step fallback fallback_step;
            val r_alt = alt_step(20);

            // Test conditional 'when'
            val cond_when = safe_step when (5 > 2);
            val r_when = cond_when(30);

            return r_retry.output + r_alt.output + r_when.output;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        // safe_step(10) = 110, safe_step(20) = 120, safe_step(30) = 130 -> 110 + 120 + 130 = 360
        assert_eq!(res, Value::Int(360), "Retry, fallback and conditional when must evaluate to 360");
    }

    #[test]
    fn test_operation_parallel_tuple_and_memoization_cache() {
        let code = r#"
        operation compute_a(x: i64) -> i64 {
            return x + 1;
        }

        operation compute_b(x: i64) -> i64 {
            return x * 10;
        }

        fn main() -> i64 {
            // Parallel composition &
            val par_flow = compute_a & compute_b;
            val par_res = par_flow(5); // array of [compute_a(5)=6, compute_b(5)=50]

            // Memoization
            val mem_op = memoize compute_b;
            val mem_res1 = mem_op(7);
            val mem_res2 = mem_op(7); // Must hit cache

            return mem_res1.output + mem_res2.output; // 70 + 70 = 140
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(140), "Memoized cached operation execution must yield 140");
        assert!(interp.memoized_cache.contains_key("compute_b"), "Cache must contain entry for compute_b");
    }

    #[test]
    fn test_operation_agent_refactoring_extract_inline_split_merge_evolve_decompose() {
        let code = r#"
        operation LegacyOrchestration {
            requires: "ctx.valid == true";
            guarantees: "ctx.committed == true";
            effects: ["db.write", "io.network"];
        }

        extract operation SliceAuth from LegacyOrchestration where responsibility == "auth_verification";
        inline operation FastAuth;
        split operation FullCheckout into ["ValidateCart", "ChargeCard", "SendInvoice"];
        merge { ValidateCart, ChargeCard } as FastCommerce;
        explain operation LegacyOrchestration;
        evolve operation LegacyOrchestration {
            preserve: ["pci_compliance", "idempotency"];
            optimize: ["latency", "throughput"];
            allow: ["async_dispatch"];
            reject: ["blocking_calls"];
        }

        decompose LegacyMonolith {
            target_modules: 25;
            optimize: ["cohesion", "coupling"];
            preserve: ["behavior", "api"];
            verify: ["compilation", "tests"];
        }

        fn main() -> i64 {
            return 25;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(25));

        assert!(interp.get_var("__extract_op_SliceAuth").is_some());
        assert!(interp.get_var("__inline_op_FastAuth").is_some());
        assert!(interp.get_var("__split_op_FullCheckout").is_some());
        assert!(interp.get_var("__merge_op_FastCommerce").is_some());
        assert!(interp.get_var("__explain_op_LegacyOrchestration").is_some());
        assert!(interp.get_var("__evolve_op_LegacyOrchestration").is_some());
        assert!(interp.get_var("__decompose_LegacyMonolith_target").is_some());
    }

