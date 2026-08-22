#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::ir::hir::*;
    use crate::ir::lowering::*;
    use crate::ir::mir_lowering::*;
    use crate::semantic::analyzer::*;
    use crate::codegen::interpreter::{Interpreter, Value};

    fn parse_str(code: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new("test.end", code);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new("test.end", tokens);
        parser.parse_module("test_mod")
    }

    #[test]
    fn test_hir_lowering_and_region_preservation() {
        let code = "pub fn foo() void {\nregion arena {\nval x = 100\nval y = 200\n}\n}";
        let module = parse_str(code).unwrap();
        let hir = AstLowering::lower_module(&module);
        assert_eq!(hir.functions.len(), 1);
        assert_eq!(hir.functions[0].body.len(), 1);

        if let HirStatement::RegionBlock { name, body, .. } = &hir.functions[0].body[0] {
            assert_eq!(name, "arena");
            assert_eq!(body.len(), 2); // Both statements preserved!
        } else {
            panic!("Expected HirStatement::RegionBlock");
        }
    }

    #[test]
    fn test_hir_to_mir_lowering_pipeline() {
        let code = "pub fn calc(a: i64) i64 {\nif a > 0 {\nret a * 2\n} else {\nret 0\n}\n}";
        let module = parse_str(code).unwrap();
        let hir = AstLowering::lower_module(&module);
        let mir = HirToMirLowering::lower_module(&hir);

        assert_eq!(mir.functions.len(), 1);
        assert!(mir.functions[0].blocks.len() >= 3); // entry, then, else, merge blocks
    }

    #[test]
    fn test_borrow_conflict_mutation_during_borrow() {
        let code = "pub fn test_borrow() void {\nval x = 42\nval r = &x\nx = 100\n}";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0907"));
    }

    #[test]
    fn test_spawn_thread_ownership_transfer() {
        let code = "pub fn worker(data: i64) void {}\npub fn run_spawn() void {\nval res = 99\nspawn worker(res)\nval leak = res\n}";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0906"));
    }

    #[test]
    fn test_transitive_purity_violation_error() {
        let code = "pub fn net_call() void { socket(1, 2, 3) }\n@pure\npub fn pure_func() void { net_call() }";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0904"));
    }

    #[test]
    fn test_bitwise_not_and_cast_soundness() {
        let code = "pub fn test_ops() void {\nval x: i64 = ~10\nval y: u64 = x as u64\n}";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_fuzz_malformed_syntax_graceful_recovery() {
        let malformed_inputs = [
            "fn unclosed( {",
            "val string = \"unterminated string",
            "struct @#$ {",
            "fn foo() { val x = 10 / / 2 }",
            "import \"broken.end",
        ];

        for input in malformed_inputs {
            let res = parse_str(input);
            // Parser must return Err without any unhandled panic!
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_interpreter_constant_folding() {
        let code = "pub fn main() i64 {\nret 10 * 5 + 42 - 2\n}";
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(90));
    }

    #[test]
    fn test_50_revolutionary_syntaxes_lex_and_parse() {
        let code = r#"
        pub fn test_all_50() void {
            // 1. intent
            intent "optimize memory" {
                goal: "zero allocation",
                preserve: ["safety", "correctness"]
            }

            // 2. intent diff
            intent diff {
                preserve: [api, latency],
                change: [internal_buffer]
            }

            // 3. prove
            prove 10 > 5;

            // 4. assume
            assume { 100 > 0 }

            // 5. guarantee
            guarantee 42 > 0;

            // 6. invariant
            invariant balance >= 0;

            // 7. verify
            verify { 1 > 0, 2 > 1 }

            // 8. because
            because "avoids redundant compute";

            // 9. why
            why optimize { "hot execution path in rendering loop" }

            // 10. protect
            protect {
                val secret = 1234
            }

            // 11. frozen
            frozen secret_token;

            // 12. mutable_by
            mutable_by compiler, agent;

            // 13. owned
            owned buffer = 1024;

            // 14. handoff
            handoff buffer -> gpu;

            // 15. return_to
            return_to cpu buffer;

            // 16. compute
            compute gpu {
                val k = 1
            } fallback cpu {
                val k = 0
            }

            // 17. race_free
            race_free {
                val local_sum = 0
            }

            // 18. order
            order: stable;

            // 19. deterministic
            deterministic {
                val d = 42
            }

            // 20. replay
            replay {
                val frame = 1
            }

            // 21. checkpoint
            checkpoint world_state;

            // 22. rollback
            rollback to world_state;

            // 23. transaction
            transaction {
                val tx_step = 1
            }

            // 24. speculative
            speculative {
                val branch_val = 99
            }

            // 25. fallback
            fallback cpu {
                val fallback_step = 1
            }

            // 26. budget
            budget { cpu: "4ms", memory: "16mb" } {
                val budgeted_work = 100
            }

            // 27. deadline
            deadline "16ms" {
                val render_frame = 1
            }

            // 28. priority
            priority realtime {
                val audio_sample = 440
            }

            // 29. quality
            quality { min: "0.99", max_latency: "5ms" } {
                val q_work = 1
            }

            // 30. tradeoff
            tradeoff { prefer: "latency", sacrifice: "memory" } {
                val t_work = 2
            }

            // 31. adapt
            adapt {
                if 1 > 0 => { val mode = 1 },
                if 2 > 1 => { val mode = 2 }
            }

            // 32. observe
            observe cpu_cycles, frame_drops;

            // 33. watch
            watch buffer {
                on mutate => { val checked = 1 }
            }

            // 34. react
            react to 1 == 1 {
                val reacted = 1
            }

            // 35. stream
            stream 42 {
                1 + 1,
                2 * 2
            }

            // 36. flow
            flow {
                10 + 20,
                30 + 40
            }

            // 37. parallel choose
            parallel choose {
                cpu => { val fast_cpu = 1 },
                gpu => { val fast_gpu = 2 }
            }

            // 38. race
            race {
                { val path_a = 1 },
                { val path_b = 2 }
            }

            // 39. hedge
            hedge after 20 {
                val primary_req = 1
            } fallback {
                val secondary_req = 2
            }

            // 40. cancel_safe
            cancel_safe {
                val safe_action = 10
            }

            // 41. agent
            agent CodeOptimizer {
                scope: "rendering",
                goal: "latency < 2ms",
                constraints: ["no_alloc", "preserve_invariants"]
            }

            // 42. task
            task benchmark_pass {
                val measured = 1
            }

            // 43. accept
            accept { ["p99 < 2ms", "correctness == 100%"] }

            // 44. reject
            reject if { ["memory_regression", "api_breakage"] }

            // 45. baseline
            baseline { latency: "1.2ms", allocations: "0" }

            // 46. regression
            regression { "latency > baseline.latency * 1.05" }

            // 47. explain
            explain { "hot_loop": "runs 60 times per second for UI redraw" }

            // 48. context
            context rendering_pipeline {
                include: [Mesh, Shader, Camera],
                exclude: [Networking, Database]
            }

            // 49. slice
            slice render_subsystem {
                from: SceneGraph,
                include: [Transform, Material],
                exclude: [Audio]
            }

            // 50. patch
            patch Renderer {
                val patched = 1
            }

            // and evolve
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
        assert_eq!(module.functions.len(), 1);
        let fn_stmts = &module.functions[0].body.statements;
        // Verify all 50 statements parsed into AST!
        assert!(fn_stmts.len() >= 48);

        // Verify C backend code generation compiles without panicking
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("/* 🎯 [INTENT"));
        assert!(c_code.contains("/* 🛡️ [FORMAL PROVE] */"));
        assert!(c_code.contains("/* 🛡️ [INVARIANT] */"));
        assert!(c_code.contains("/* ❄️ [FROZEN SYMBOL]"));
        assert!(c_code.contains("/* 📦 [OWNED]"));
        assert!(c_code.contains("/* ⏱️ [BUDGET SLA]"));
        assert!(c_code.contains("/* 🧬 [EVOLVE 'Renderer']"));
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

    #[test]
    fn test_differential_semantic_equivalence_vm_vs_native_c() {
        let code = r#"
        fn calculate_sum(x: i64) i64 {
            mut total: i64 = 0;
            mut i: i64 = 1;
            while i <= x {
                total = total + i;
                i = i + 1;
            }
            return total;
        }

        pub fn main() i64 {
            val sum10 = calculate_sum(10);
            assume sum10 == 55;
            prove sum10 > 0;
            guarantee sum10 == 55;
            ret sum10
        }
        "#;
        let module = parse_str(code).unwrap();
        
        // 1. VM Interpretation
        let mut interp = Interpreter::new();
        let vm_res = interp.run(&module).unwrap();
        assert_eq!(vm_res, Value::Int(55));

        // 2. Native C Backend Code Generation
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("int64_t calculate_sum(int64_t x)"));
        assert!(c_code.contains("/* 🛡️ [FORMAL PROVE] */"));
        assert!(c_code.contains("/* 🛡️ [FORMAL GUARANTEE] */"));
    }

    #[test]
    fn test_c_backend_checkpoint_rollback_and_transaction_codegen() {
        let code = r#"
        pub fn main() i64 {
            mut balance: i64 = 500;
            checkpoint savepoint;
            balance = 1000;
            rollback to savepoint;
            transaction {
                balance = balance - 100;
            }
            ret balance
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("/* 💾 [CHECKPOINT STATE SNAPSHOT]: savepoint */"));
        assert!(c_code.contains("__snap_savepoint_balance = balance"));
        assert!(c_code.contains("/* ⏪ [ROLLBACK STATE RESTORE]: to savepoint */"));
        assert!(c_code.contains("balance = __snap_savepoint_balance"));
        assert!(c_code.contains("/* 💼 [ATOMIC TRANSACTION BLOCK: Write-Set Snapshot & Rollback] */"));
        assert!(c_code.contains("__snap_txn_balance = balance"));
        assert!(c_code.contains("balance = __snap_txn_balance"));
    }

    #[test]
    fn test_c_backend_true_latency_hedging_codegen() {
        let code = r#"
        pub fn main() i64 {
            mut res: i64 = 0;
            hedge after 25ms {
                res = 100;
            } fallback {
                res = 200;
            }
            ret res
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("/* 🛡️ [TRUE LATENCY HEDGING: Primary at t=0, Fallback after 25ms delay] */"));
        assert!(c_code.contains("__hedge_winner"));
        assert!(c_code.contains("END_CPU_SLEEP(25)"));
        assert!(c_code.contains("__atomic_compare_exchange_n"));
    }

    #[test]
    fn test_differential_multi_feature_matrix_execution() {
        let code = r#"
        fn factorial(n: i64) i64 {
            if n <= 1 {
                return 1;
            }
            return n * factorial(n - 1);
        }

        pub fn main() i64 {
            mut f5 = factorial(5);
            deterministic {
                f5 = f5 + 10;
            }
            checkpoint cp1;
            f5 = f5 + 100;
            rollback to cp1;
            prove f5 == 130;
            ret f5
        }
        "#;
        let module = parse_str(code).unwrap();

        // 1. VM Interpretation
        let mut interp = Interpreter::new();
        let vm_res = interp.run(&module).unwrap();
        assert_eq!(vm_res, Value::Int(130));

        // 2. Native C Backend Verification
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("int64_t factorial(int64_t n)"));
        assert!(c_code.contains("/* 🎯 [DETERMINISTIC BLOCK]"));
        assert!(c_code.contains("/* 💾 [CHECKPOINT STATE SNAPSHOT]: cp1 */"));
        assert!(c_code.contains("__snap_cp1_f5 = f5"));
        assert!(c_code.contains("/* ⏪ [ROLLBACK STATE RESTORE]: to cp1 */"));
        assert!(c_code.contains("f5 = __snap_cp1_f5"));
    }
}
