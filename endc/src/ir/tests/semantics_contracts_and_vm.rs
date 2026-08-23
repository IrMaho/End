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
    fn test_frozen_symbol_mutation_prevention() {
        let code = r#"
        pub fn test_frozen_violation() void {
            mut config = 100
            frozen config;
            config = 200
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0908" && e.message.contains("FrozenViolation")));
    }

    #[test]
    fn test_formal_verification_invariants_and_prove_semantics() {
        let code = r#"
        pub fn verify_contracts(x: i64) i64 {
            assume x > 0;
            invariant x >= 0;
            prove x + 1 > 0;
            guarantee x > 0;
            verify { x > 0, x + 1 > 1 }
            ret x
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_owned_variable_and_compute_handoff() {
        let code = r#"
        pub fn test_owned() void {
            owned data: i64 = 42;
            handoff data -> gpu;
            compute gpu {
                val temp = 10
            } fallback cpu {
                val temp = 5
            }
            return_to cpu data;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_budget_deadline_and_concurrency_features() {
        let code = r#"
        pub fn test_concurrency_and_sla() void {
            budget { cpu: "5ms", memory: "32mb" } {
                deadline "16ms" {
                    race_free {
                        val sum = 0
                    }
                }
            }
            parallel choose {
                fast => { val a = 1 },
                slow => { val b = 2 }
            }
            race {
                { val p1 = 1 },
                { val p2 = 2 }
            }
            hedge after 10 {
                val primary = 100
            } fallback {
                val fallback_res = 50
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_observability_and_reactive_stream_flow_statements() {
        let code = r#"
        pub fn test_reactive() void {
            observe cpu_usage, memory_rss;
            watch socket_buffer {
                on mutate => { val checked = 1 }
            }
            react to 10 > 5 {
                val reacted = 1
            }
            stream 42 {
                1 + 1,
                2 * 2
            }
            flow {
                10 + 20,
                30 + 40
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_agent_contracts_evolution_and_slicing_statements() {
        let code = r#"
        pub fn test_agent_evolution() void {
            agent CodeOptimizer {
                scope: "rendering",
                goal: "latency < 2ms",
                constraints: ["no_alloc", "preserve_invariants"]
            }
            task benchmark_pass {
                val measured = 1
            }
            accept { ["p99 < 2ms", "correctness == 100%"] }
            reject if { ["memory_regression", "api_breakage"] }
            baseline { latency: "1.2ms", allocations: "0" }
            regression { "latency > baseline.latency * 1.05" }
            explain { "hot_loop": "runs 60 times per second for UI redraw" }
            context rendering_pipeline {
                include: [Mesh, Shader, Camera],
                exclude: [Networking, Database]
            }
            slice render_subsystem {
                from: SceneGraph,
                include: [Transform, Material],
                exclude: [Audio]
            }
            patch Renderer {
                val patched = 1
            }
            evolve Renderer {
                intent: "support Vulkan 1.3",
                preserve: ["backward_compatibility", "zero_leak"],
                budget: "8ms",
                allow: ["add_method", "inline_assembly"],
                reject: ["breaking_change"],
                verify: ["invariants", "regression_tests"],
                accept: ["p99 < 16ms"]
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_interpreter_vm_evaluation_of_contracts_and_ownership() {
        let code = r#"
        pub fn main() i64 {
            owned a = 10;
            intent "compute sum" {
                goal: "fast add",
                preserve: ["safety"]
            }
            assume a > 0;
            prove a == 10;
            guarantee a + 5 == 15;
            verify { a > 0, a == 10 }
            adapt {
                if a == 10 => { a = a + 5 },
                if a == 0 => { a = 0 }
            }
            budget { cpu: "1ms" } {
                a = a * 2;
            }
            ret a
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(30));
    }

    #[test]
    fn test_race_free_data_race_detection_negative_and_positive() {
        // Negative test: mutating shared outer variable inside race_free block
        let bad_code = r#"
        pub fn test_bad_race() void {
            mut shared = 42
            race_free {
                shared = 100
            }
        }
        "#;
        let module = parse_str(bad_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", bad_code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0910" && e.message.contains("RaceConditionDetected")));

        // Positive test: mutating local variable inside race_free block
        let good_code = r#"
        pub fn test_good_race() void {
            race_free {
                mut local = 42
                local = 100
            }
        }
        "#;
        let module2 = parse_str(good_code).unwrap();
        let mut analyzer2 = SemanticAnalyzer::new("test.end", good_code);
        let res2 = analyzer2.analyze_module(&module2);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_handoff_and_return_to_domain_isolation() {
        // Negative test: accessing buffer after handing off to GPU before return_to
        let bad_code = r#"
        pub fn test_bad_handoff() void {
            val buffer = 500
            handoff buffer -> gpu;
            val read_buf = buffer
        }
        "#;
        let module = parse_str(bad_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", bad_code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0909" && e.message.contains("DomainBorrowConflict")));

        // Positive test: returning buffer back to CPU before reading
        let good_code = r#"
        pub fn test_good_handoff() void {
            val buffer = 500
            handoff buffer -> gpu;
            return_to cpu buffer;
            val read_buf = buffer
        }
        "#;
        let module2 = parse_str(good_code).unwrap();
        let mut analyzer2 = SemanticAnalyzer::new("test.end", good_code);
        let res2 = analyzer2.analyze_module(&module2);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_static_proof_compile_time_failure() {
        // Negative test: statically false proof obligation
        let bad_code = r#"
        pub fn test_bad_proof() void {
            prove 5 > 10;
        }
        "#;
        let module = parse_str(bad_code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", bad_code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0911" && e.message.contains("StaticProofFailed")));

        // Positive test: valid static proof obligation
        let good_code = r#"
        pub fn test_good_proof() void {
            prove 10 > 5;
            guarantee 100 == 100;
        }
        "#;
        let module2 = parse_str(good_code).unwrap();
        let mut analyzer2 = SemanticAnalyzer::new("test.end", good_code);
        let res2 = analyzer2.analyze_module(&module2);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_interpreter_checkpoint_and_rollback_behavior() {
        let code = r#"
        pub fn main() i64 {
            mut current_val = 100
            checkpoint save_point;
            current_val = 999
            rollback to save_point;
            ret current_val
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(100));
    }

    #[test]
    fn test_interpreter_transaction_rollback_on_failure() {
        let code = r#"
        pub fn main() i64 {
            mut balance = 100
            transaction {
                balance = 50
                invariant 1 == 2;
            }
            ret balance
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Transaction aborted and rolled back"));
    }

    #[test]
    fn test_interpreter_race_parallel_choose_and_hedge() {
        let code = r#"
        pub fn main() i64 {
            mut result = 0
            parallel choose {
                fast => { result = 10 },
                slow => { result = 20 }
            }
            race {
                { result = result + 5 },
                { result = result + 5 }
            }
            hedge after 10ms {
                result = result * 2
            } fallback {
                result = result * 2
            }
            ret result
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert!(matches!(res, Value::Int(_)));
    }

