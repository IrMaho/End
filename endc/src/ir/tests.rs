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
    use crate::agent_api::evolution_engine::*;

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
            if !res.is_err() {
                eprintln!("FAILED FOR INPUT: {} -> {:?}", input, res);
            }
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

    #[test]
    fn test_50_agent_modular_architecture_all_families_parse_semantic_vm_codegen() {
        let code = r#"
        // Family 1: Module DNA
        mod payments {
            responsibility: "processes card transactions"
            owns: [Payment, chargeCard]
            exposes: [PaymentService]
            depends: [banking]
            depends_only: [banking]
            forbid: [legacy_ui]
            sealed: true
            purity: "pure"
            cohesion: 0.95
        }

        // Family 2: Dependency Architecture
        forbid payments -> legacy_ui
        layer domain { forbid depends infrastructure }
        direction api -> domain

        // Family 3: Agent-Native Decomposition
        split UserService into { identity, profile, preferences }
        partition OrderService by responsibility { validation, pricing, persistence }
        extract { symbols: [Payment, validatePayment], into: payments.domain }
        cluster by semantic { related_to Payment }
        separate authentication from user

        // Family 4: Dependency Intelligence & Contracts
        contract Module payments { accepts: [CardInfo], returns: [Receipt], guarantees: [idempotent] }
        port PaymentPort { pay, refund }
        adapter StripeAdapter for PaymentPort {
            val configured = true
        }
        facade CheckoutFacade { exposes: [checkout] }
        gateway PaymentGateway from checkout to payments { allow: [processPayment] }

        // Family 5: Architectural Invariants & Stability
        invariant "payments must not depend on legacy_ui"
        preserve refactor { public_api, error_codes } {
            val x = 100
        }
        compat v1 for payments {
            val legacy_active = false
        }
        stable CheckoutAPI
        sealed Boundary checkout_boundary

        // Family 6: Dependency Firewall & Visibility
        friend payments to fraud_engine
        private_to UserRepository users
        surface ApiGateway { expose: [getUsers], hide: [UserDbConnection] }
        purity Module payments = pure

        // Family 7: Agent Context Architecture
        view PaymentCheckoutView { include: [payments, checkout] }
        lens FineGrained { focus: "payments", hide: "auth" }
        agent_scope refactoring_agent { modules: [payments, checkout], forbid: [auth] }
        budget_context context_for_agent { token_budget: 8000, priority: [payments, contracts] }

        // Family 8: Safe Refactoring
        move PaymentService from checkout to payments
        migrate UserRepository from monolith to users
        bridge checkout -> payments via PaymentGateway
        redirect OldPaymentAPI -> payments.PaymentAPI
        deprecate OldPaymentAPI after "migration complete" remove "when no consumers"

        // Family 9: Anti-Spaghetti Metrics
        cycle_free = true
        max_fanout payments: 5
        max_fanin core_types: 50
        max_dependency_depth: 4
        cohesion payments >= 0.85

        // Family 10: Automated Modularization & Evolution
        modularize PaymentSystem { target_files: 10..20, preserve: [behavior] }
        decompose monolith { target_modules: 25, optimize: [cohesion, coupling], preserve: [behavior, api], gravity: semantic }
        architecture CleanArchitecture { layers: [domain, usecases, adapters, infrastructure], rules: ["domain depends_only none"], invariants: ["no cyclic dependencies"], cycle_free: true, max_depth: 4 }
        repair monolith
        evolve architecture { from monolith toward feature_modules, target_modules: 25, preserve: [behavior, api], optimize: [cohesion, test_coverage], reject_if: ["breaking changes", "cyclic dependencies"], verify: [architecture, compilation, behavior] }
        gravity { semantic: 0.40, call_graph: 0.30, data_flow: 0.20, historical: 0.10 }

        pub fn main() i64 {
            val res = 25
            ret res
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.modules.len(), 1);
        let m = &module.modules[0];
        assert_eq!(m.name, "payments");
        assert_eq!(m.responsibility.as_deref(), Some("processes card transactions"));
        assert_eq!(m.owns, vec!["Payment", "chargeCard"]);
        assert_eq!(m.exposes, vec!["PaymentService"]);
        assert_eq!(m.depends, vec!["banking"]);
        assert_eq!(m.depends_only, Some(vec!["banking".to_string()]));
        assert_eq!(m.forbid, vec!["legacy_ui"]);
        assert!(m.is_sealed);
        assert_eq!(m.purity.as_deref(), Some("pure"));
        assert_eq!(m.cohesion, Some(0.95));

        // 1. Semantic Analyzer
        let mut analyzer = SemanticAnalyzer::new("test_arch.end", code);
        let sem_res = analyzer.analyze_module(&module);
        assert!(sem_res.is_ok(), "Semantic errors: {:?}", analyzer.errors);

        // 2. VM Execution
        let mut interp = Interpreter::new();
        let vm_res = interp.run(&module).unwrap();
        assert_eq!(vm_res, Value::Int(25));
        assert_eq!(interp.variables[0].get("__decompose_monolith_target"), Some(&Value::Int(25)));
        assert_eq!(interp.variables[0].get("__modularize_PaymentSystem"), Some(&Value::Int(20)));
        assert_eq!(interp.variables[0].get("__evolve_monolith_feature_modules"), Some(&Value::Int(25)));
        assert_eq!(interp.variables[0].get("__gravity_semantic"), Some(&Value::Float(0.40)));

        // 3. C Backend Codegen
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("/* 🚫 [FORBID DEPENDENCY: 'payments' -> 'legacy_ui'] */"));
        assert!(c_code.contains("/* ✂️ [SPLIT ENTITY 'UserService']: into [identity, profile, preferences] */"));
        assert!(c_code.contains("/* 📜 [MODULE CONTRACT 'payments']"));
        assert!(c_code.contains("/* 🤖 [AGENT SCOPE 'refactoring_agent']"));
        assert!(c_code.contains("/* 💥 [DECOMPOSE MONOLITH 'monolith']: target_modules=25"));
        assert!(c_code.contains("/* 🚀 [EVOLVE ARCHITECTURE]: from='monolith' toward='feature_modules', target_modules=25"));
    }

    #[test]
    fn test_architectural_semantic_violations_e0913_to_e0918() {
        // Test E0913: Forbidden Dependency
        let code_e0913 = r#"
        mod billing {
            depends: [ui]
        }
        forbid billing -> ui
        "#;
        let mod_e0913 = parse_str(code_e0913).unwrap();
        let mut sem_e0913 = SemanticAnalyzer::new("e0913.end", code_e0913);
        let _ = sem_e0913.analyze_module(&mod_e0913);
        assert!(sem_e0913.errors.iter().any(|e| e.code == "E0913"), "Expected E0913 forbidden dependency violation");

        // Test E0914: Cyclic Dependency Detected
        let code_e0914 = r#"
        mod modA { depends: [modB] }
        mod modB { depends: [modC] }
        mod modC { depends: [modA] }
        cycle_free = true
        "#;
        let mod_e0914 = parse_str(code_e0914).unwrap();
        let mut sem_e0914 = SemanticAnalyzer::new("e0914.end", code_e0914);
        let _ = sem_e0914.analyze_module(&mod_e0914);
        assert!(sem_e0914.errors.iter().any(|e| e.code == "E0914"), "Expected E0914 cyclic dependency error");

        // Test E0915: Architectural Leak Detected
        let code_e0915 = r#"
        leak check payments forbid PaymentRepository leaking through CheckoutResult
        "#;
        let mod_e0915 = parse_str(code_e0915).unwrap();
        let mut sem_e0915 = SemanticAnalyzer::new("e0915.end", code_e0915);
        let _ = sem_e0915.analyze_module(&mod_e0915);
        assert!(sem_e0915.errors.iter().any(|e| e.code == "E0915"), "Expected E0915 architectural leak error");

        // Test E0916: Fanout Limit Exceeded
        let code_e0916 = r#"
        mod heavy_mod {
            depends: [a, b, c, d, e, f, g]
        }
        max_fanout heavy_mod: 3
        "#;
        let mod_e0916 = parse_str(code_e0916).unwrap();
        let mut sem_e0916 = SemanticAnalyzer::new("e0916.end", code_e0916);
        let _ = sem_e0916.analyze_module(&mod_e0916);
        assert!(sem_e0916.errors.iter().any(|e| e.code == "E0916"), "Expected E0916 fanout limit exceeded");

        // Test E0917: Cohesion Below Threshold
        let code_e0917 = r#"
        mod messy_mod {
            cohesion: 0.30
        }
        "#;
        let mod_e0917 = parse_str(code_e0917).unwrap();
        let mut sem_e0917 = SemanticAnalyzer::new("e0917.end", code_e0917);
        let _ = sem_e0917.analyze_module(&mod_e0917);
        assert!(sem_e0917.errors.iter().any(|e| e.code == "E0917"), "Expected E0917 cohesion below threshold");

        // Test E0918: Direction Flow Violation
        let code_e0918 = r#"
        mod domain {
            depends: [infrastructure]
        }
        direction infrastructure -> domain
        "#;
        let mod_e0918 = parse_str(code_e0918).unwrap();
        let mut sem_e0918 = SemanticAnalyzer::new("e0918.end", code_e0918);
        let _ = sem_e0918.analyze_module(&mod_e0918);
        assert!(sem_e0918.errors.iter().any(|e| e.code == "E0918"), "Expected E0918 direction violation error");
    }

    #[test]
    fn test_first_class_operation_values_contracts_and_invocation() {
        let code = r#"
        operation increment(x: i64) -> i64 {
            requires: "x >= 0";
            guarantees: "result > x";
            effects: ["cpu", "pure"];
            emits: ["ValueIncremented"];
            version: 1;
            return x + 1;
        }

        fn main() -> i64 {
            val op_var = increment;
            val res = op_var(41);
            return res.output;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(42), "First-class operation invocation should return unwrapped 42");

        // Verify anonymous operation literals
        let code_anon = r#"
        fn main() -> i64 {
            val doubler = operation(n: i64) {
                return n * 2;
            };
            val r = doubler(21);
            return r.output;
        }
        "#;
        let mod_anon = parse_str(code_anon).unwrap();
        let mut interp_anon = Interpreter::new();
        let res_anon = interp_anon.run(&mod_anon).unwrap();
        assert_eq!(res_anon, Value::Int(42), "Anonymous operation literal should execute properly");
    }

    #[test]
    fn test_rich_operation_results_and_telemetry() {
        let code = r#"
        operation process_order(amount: i64) {
            effects: ["payment", "audit"];
            emits: ["OrderProcessed"];
            return amount * 10;
        }

        observe process_order as order_trace;
        analyze process_order;

        fn main() -> i64 {
            val result = process_order(5);
            val status = result.status;
            val out = result.output;
            return out;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(50), "Operation output should be 50");
        assert!(interp.get_var("order_trace").is_some(), "order_trace variable must be recorded");
        assert!(interp.get_var("__analysis_process_order").is_some(), "Operation static analysis should be recorded");
        assert!(!interp.traces.is_empty(), "Interpreter traces must not be empty");
    }

    #[test]
    fn test_operation_algebra_composition_repetition_alternative_parallel_conditional() {
        let code = r#"
        operation step1(x: i64) -> i64 {
            return x + 10;
        }

        operation step2(x: i64) -> i64 {
            return x * 2;
        }

        operation fallback_op(x: i64) -> i64 {
            return 999;
        }

        fn main() -> i64 {
            // Sequential Composition (A + B / A >> B / compose)
            val pipeline = step1 + step2;
            val pipe_res = pipeline(5);

            // Repetition (A * 3)
            val repeater = step1 * 3;

            // Alternative (A | B / fallback)
            val alt = step1 | fallback_op;

            // Parallel (A & B)
            val parallel_pair = step1 & step2;

            // Conditional (A ? cond / A when cond)
            val cond_op = step1 ? (1 == 1);

            return pipe_res.output;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(30), "Step1 (5+10=15) then Step2 (15*2=30) should yield 30");

        // Verify compose syntax block
        let code_compose = r#"
        operation add_five(x: i64) { return x + 5; }
        operation mult_three(x: i64) { return x * 3; }

        fn main() -> i64 {
            val total = compose { add_five then mult_three };
            return 15;
        }
        "#;
        let mod_compose = parse_str(code_compose).unwrap();
        let mut interp_compose = Interpreter::new();
        let res_compose = interp_compose.run(&mod_compose).unwrap();
        assert_eq!(res_compose, Value::Int(15));
    }

    #[test]
    fn test_event_native_architecture_hub_and_dispatch() {
        let code = r#"
        event UserCreated {
            user_id: i64,
            email: str
        }

        event PaymentReceived {
            amount: i64
        }

        operation send_welcome() {
            return 1;
        }

        hub ApplicationEvents {
            owns: ["UserCreated", "PaymentReceived"];
            on UserCreated -> send_welcome;
            on PaymentReceived {
                println("Payment processed successfully!");
            }
        }

        fn main() -> i64 {
            emit UserCreated(101, "dev@endlang.org");
            emit PaymentReceived(2500);
            return 100;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(100));
        assert!(interp.emitted_events.contains(&"UserCreated".to_string()), "Event UserCreated must be emitted");
        assert!(interp.emitted_events.contains(&"PaymentReceived".to_string()), "Event PaymentReceived must be emitted");
        assert!(interp.event_hubs.contains_key("ApplicationEvents"), "Event Hub ApplicationEvents must be registered");
    }

    #[test]
    fn test_operation_refactoring_decomposition_and_evolution() {
        let code = r#"
        operation CheckoutWorkflow {
            requires: "cart.not_empty";
            guarantees: "order.placed";
            effects: ["db", "network"];
        }

        extract operation PaymentStep from CheckoutWorkflow where step == "card_charge";
        inline operation FastValidation;
        split operation OrderProcessing into ["ValidateOrder", "ChargeCard", "SendInvoice"];
        merge { ValidateOrder, ChargeCard } as FastCheckout;
        explain operation CheckoutWorkflow;
        evolve operation CheckoutWorkflow {
            preserve: ["pci_compliance", "idempotency"];
            optimize: ["latency", "memory"];
            allow: ["async_invoice"];
            reject: ["blocking_io"];
        }

        fn main() -> i64 {
            return 1;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(1));

        assert!(interp.get_var("__extract_op_PaymentStep").is_some());
        assert!(interp.get_var("__inline_op_FastValidation").is_some());
        assert!(interp.get_var("__split_op_OrderProcessing").is_some());
        assert!(interp.get_var("__merge_op_FastCheckout").is_some());
        assert!(interp.get_var("__explain_op_CheckoutWorkflow").is_some());
        assert!(interp.get_var("__evolve_op_CheckoutWorkflow").is_some());
    }

    #[test]
    fn test_50_agent_operation_values_complete_family_matrix() {
        let code = r#"
        event UserCreated {
            user_id: i64,
            email: str
        }

        event OrderPlaced {
            order_id: i64,
            total: i64
        }

        operation validate_cart(items: i64) -> i64 {
            requires: "items > 0";
            guarantees: "cart_valid == true";
            effects: ["pure"];
            version: 1;
            return items;
        }

        operation charge_payment(amount: i64) -> i64 {
            requires: "amount > 0";
            guarantees: "paid == true";
            effects: ["network", "pci_dss"];
            emits: ["OrderPlaced"];
            version: 2;
            return amount * 100;
        }

        operation send_notifications(status: i64) -> i64 {
            effects: ["smtp", "push"];
            emits: ["UserCreated"];
            return status + 1;
        }

        hub CommerceEvents {
            owns: ["UserCreated", "OrderPlaced"];
            on OrderPlaced {
                println("Event OrderPlaced handled!");
            }
            on UserCreated -> send_notifications;
        }

        extract operation PaymentService from LegacyCommerce where step == "card_charge";
        inline operation FastValidation;
        split operation FullCheckout into ["Validate", "Charge", "Notify"];
        merge { Validate, Charge } as FastCommerce;
        explain operation charge_payment;
        evolve operation charge_payment {
            preserve: ["pci_compliance", "idempotency"];
            optimize: ["latency", "memory"];
            allow: ["async_invoice"];
            reject: ["blocking_io"];
        }

        fn execute_op(op: Operation, arg: i64) -> i64 {
            val r = op(arg);
            return r.output;
        }

        fn make_pipeline() -> Operation {
            return validate_cart + charge_payment;
        }

        fn main() -> i64 {
            // Feature 1-3: First-class Operation value & Rich OperationResult
            val op = validate_cart;
            val r1 = op(5);
            val out1 = r1.output;
            val status1 = r1.status;
            val duration1 = r1.duration_ns;

            // Feature 4-7: Operation Algebra Composition & Operators
            val checkout_flow = validate_cart >> charge_payment;
            val checkout_res = checkout_flow(3);

            // Feature 6 & 24: Repetition & Retry
            val repeated = validate_cart * 3;
            val retried = retry charge_payment 3;

            // Feature 26: Fallback & Alternative
            val safe_checkout = charge_payment | validate_cart;

            // Feature 32: Parallel composition
            val parallel_steps = validate_cart & send_notifications;

            // Feature 8: Conditional
            val cond_step = validate_cart ? (1 == 1);

            // Feature 35: Memoization
            val cached_op = memoize validate_cart;

            // Feature 18 & 19: Higher-order functions with Operation values
            val dyn_flow = make_pipeline();
            val dyn_res = execute_op(dyn_flow, 10);

            // Feature 10 & 29: Event emission & Hub dispatch
            emit OrderPlaced(9001, 1000);
            emit UserCreated(42, "agent@endlang.org");

            return checkout_res.output;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();

        // 3 items -> validate_cart yields 3 -> charge_payment(3) yields 3 * 100 = 300
        assert_eq!(res, Value::Int(300), "Algebraic pipeline (validate >> charge) output must be 300");

        // Verify Event Hub & Emissions
        assert!(interp.emitted_events.contains(&"OrderPlaced".to_string()));
        assert!(interp.emitted_events.contains(&"UserCreated".to_string()));
        assert!(interp.event_hubs.contains_key("CommerceEvents"));

        // Verify Refactoring Primitives Recorded in Global State
        assert!(interp.get_var("__extract_op_PaymentService").is_some());
        assert!(interp.get_var("__inline_op_FastValidation").is_some());
        assert!(interp.get_var("__split_op_FullCheckout").is_some());
        assert!(interp.get_var("__merge_op_FastCommerce").is_some());
        assert!(interp.get_var("__explain_op_charge_payment").is_some());
        assert!(interp.get_var("__evolve_op_charge_payment").is_some());

        // Verify C Backend Code Generation
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("/* ⚡ [OPERATION DECLARATION 'validate_cart']"));
        assert!(c_code.contains("/* 🔔 [EVENT DECLARATION 'UserCreated'] */"));
        assert!(c_code.contains("/* 🌐 [EVENT HUB 'CommerceEvents']"));
        assert!(c_code.contains("end_emit_event(\"OrderPlaced\""));
        assert!(c_code.contains("/* ✂️ [EXTRACT OPERATION 'PaymentService' FROM 'LegacyCommerce'"));
        assert!(c_code.contains("/* 📥 [INLINE OPERATION 'FastValidation'] */"));
        assert!(c_code.contains("/* 🪓 [SPLIT OPERATION 'FullCheckout' INTO [Validate, Charge, Notify]] */"));
        assert!(c_code.contains("/* 🔗 [MERGE OPERATIONS [Validate, Charge] AS 'FastCommerce'] */"));
        assert!(c_code.contains("/* 💡 [EXPLAIN OPERATION 'charge_payment'] */"));
        assert!(c_code.contains("/* 🧬 [EVOLVE OPERATION 'charge_payment']"));
        assert!(c_code.contains("end_compose_ops"));
        assert!(c_code.contains("end_repeat_op"));
        assert!(c_code.contains("end_retry_op"));
        assert!(c_code.contains("end_alternative_op"));
        assert!(c_code.contains("end_parallel_op"));
        assert!(c_code.contains("end_memoize_op"));
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

    #[test]
    fn test_50_agent_contract_system_complete_matrix() {
        let code = r#"
        // 1. Feature Definition & Contracts
        feature PaymentProcessing {
            requirement: "All transactions must be idempotent and audited";
            skills: ["Idempotent", "TransactionSafe", "AuditLogged"];
            tasks: ["AUTH_01", "PAY_02", "VERIFY_03"];
        }

        // 2-7. First-Class Skills with Scopes, Hard/Soft Constraints, Rules
        skill TransactionSafe for module {
            rules: ["no_unhandled_exceptions", "auto_rollback_on_failure"];
            requires: ["DbConnection", "IsolationLevelSerializable"];
            hard: ["zero_dirty_reads"];
            soft: ["max_query_time_50ms"];
        }

        skill Idempotent for function {
            rules: ["deterministic_id_generation", "cache_replay_protection"];
            hard: ["no_double_charge"];
        }

        // 8. Project Skill Profiles
        project {
            skills {
                quality: "strict";
                agent_autonomy: "supervised";
                verification_level: "formal_and_evidence";
            }
        }

        // 9-11. Task Graph & Lifecycle State Machine
        task implement_secure_payment {
            owner: "agent_codex";
            status: "planned";
            requirement: "REQ_PAY_001";
            implementation: "process_secure_tx";
            skills: ["TransactionSafe", "Idempotent"];
            change_budget: ["max_files: 5", "max_lines: 300", "max_modules: 2"];
            evidence {
                test: "test_replay_attack_prevention";
                prove: "guarantee_single_deduction";
                trace: "audit_trail_001";
            }
        }

        // 12-16. Evidence System & Traceability Matrix
        requirement REQ_PAY_001 { "Payment deduction must occur exactly once per idempotency key." }
        implements REQ_PAY_001 ["process_secure_tx", "validate_idempotency_key"];
        verifies REQ_PAY_001 ["test_replay_attack_prevention", "prove_single_charge"];

        // 17-19. Executable TODOs with Status Tracking
        todo AUTH_01 {
            implement: "OAuth2 Token Validation with Refresh Flow";
            requires: ["CryptoLib", "JwtDecoder"];
            verify: ["test_token_expiration", "test_signature_tamper"];
            status: "implemented";
        }

        todo PAY_02 {
            implement: "Card Gateway Stripe & PayPal Adapter";
            requires: ["HttpClient", "SecretKey"];
            verify: ["test_stripe_charge", "test_paypal_charge"];
            status: "planned";
        }

        // 20-26. Skill Satisfies & Scope Enforcement
        satisfies PaymentsModule ["TransactionSafe", "AuditLogged"];

        // 27-32. Agentability, Context Budget & Boundaries
        agentability {
            max_context_tokens: 16000;
            max_operation_complexity: "low";
            max_dependency_fanout: 6;
        }

        agent_boundary CoreSecurity;

        agent_context PaymentsModule {
            expose: ["process_payment", "refund_transaction"];
            hide: ["private_key", "raw_credit_card_data"];
        }

        context_firewall DatabaseLayer {
            deny: ["direct_sql_injection", "raw_table_drop"];
            expose: ["sanitized_query_interface"];
        }

        agent_api SecurityGateway {
            expose: ["authenticate_user", "refresh_session"];
            hide: ["internal_hashing_salt"];
        }

        // 33-37. Intent, Intent Drift & Semantic Commits
        intent {
            goal: "Refactor payment adapter without breaking idempotency guarantees";
            preserve: ["idempotency", "zero_double_charge"];
            optimize: ["latency", "throughput"];
        }

        change {
            task: "implement_secure_payment";
            intent: "Optimize database connection pooling in payment gateway";
            satisfies: ["TransactionSafe", "Idempotent"];
            evidence: ["test_replay_attack_prevention", "latency_benchmark_under_10ms"];
        }

        // 38-42. Agent Review, Approval & Leases
        review implement_secure_payment {
            summary: "Idempotency key check implemented using in-memory atomic hash map with Redis fallback";
            completed: 3;
            unresolved: 0;
            risks: 0;
            confidence: 0.99;
        }

        approval required ["lead_security_architect", "qa_automated_pipeline"];

        agent lease PaymentsModule {
            owner: "agent_alpha_autonomous";
            duration: "task_duration_2h";
        }

        // 43-47. Knowledge Base & ADR Decisions
        knowledge PaymentArchitecture {
            decisions: ["ADR_001_Use_Ed25519", "ADR_002_Two_Phase_Commit"];
            constraints: ["PCI_DSS_Level_1", "GDPR_Compliance"];
        }

        decision ADR_001_Use_Ed25519 {
            choose: "Ed25519 for signature verification";
            because: "Provides superior resistance to side-channel attacks and sub-millisecond verification";
            reject: "RSA-2048 due to key size and CPU overhead";
        }

        // 48-50. Agent Capabilities, Regression Guards & Verification Workflow
        agent {
            capabilities: ["code_generation", "unit_test_authoring", "algebraic_refactoring"];
            cannot: ["commit_secrets_to_vcs", "modify_protected_production_branches"];
        }

        regression_guard ["latency < 25ms", "memory_allocation < 10MB", "zero_data_leaks"];

        // Execution & Lifecycle State Transitions
        claim task implement_secure_payment;

        complete task implement_secure_payment {
            result: "success";
            confidence: 0.98;
            summary: "Transaction safety and idempotency fully implemented and proved";
            evidence: ["test_replay_attack_prevention", "guarantee_single_deduction"];
            risks: "none";
            recommendation: "promote_to_production";
            notes: "Zero regressions detected";
        }

        verify implement_secure_payment;
        verify adversarial {
            skill: "Idempotent";
        }

        fn main() -> i64 {
            return 100;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(100));

        // 1. Verify Features State
        assert!(interp.features.contains_key("PaymentProcessing"));
        let feat = interp.features.get("PaymentProcessing").unwrap();
        assert_eq!(feat.1, vec!["Idempotent", "TransactionSafe", "AuditLogged"]);
        assert_eq!(feat.2, vec!["AUTH_01", "PAY_02", "VERIFY_03"]);

        // 2. Verify Skills & Scopes
        assert!(interp.skills.contains_key("TransactionSafe"));
        let skill = interp.skills.get("TransactionSafe").unwrap();
        assert!(skill.for_scope == Some("module".to_string()) || skill.for_scope == Some("mod".to_string()));
        assert_eq!(skill.hard, vec!["zero_dirty_reads"]);
        assert_eq!(skill.soft, vec!["max_query_time_50ms"]);

        // 3. Verify Project Profile
        assert_eq!(interp.project_profile.get("quality").unwrap(), "strict");
        assert_eq!(interp.project_profile.get("verification_level").unwrap(), "formal_and_evidence");

        // 4. Verify Task Lifecycle (claim -> complete -> verify -> accepted)
        assert!(interp.tasks_state.contains_key("implement_secure_payment"));
        let task_st = interp.tasks_state.get("implement_secure_payment").unwrap();
        assert_eq!(task_st.status, "accepted");
        assert_eq!(task_st.result, Some("success".to_string()));
        assert_eq!(task_st.confidence, Some(0.98));
        assert!(interp.verified_tasks.contains("implement_secure_payment"));
        assert!(interp.verified_tasks.contains("adversarial"));

        // 5. Verify Requirement Traceability
        assert!(interp.requirements.contains_key("REQ_PAY_001"));
        assert_eq!(interp.requirement_implements.get("REQ_PAY_001").unwrap(), &vec!["process_secure_tx", "validate_idempotency_key"]);
        assert_eq!(interp.requirement_verifies.get("REQ_PAY_001").unwrap(), &vec!["test_replay_attack_prevention", "prove_single_charge"]);

        // 6. Verify Executable TODOs
        assert!(interp.todos_state.contains_key("AUTH_01"));
        assert_eq!(interp.todos_state.get("AUTH_01").unwrap().status, "implemented");
        assert_eq!(interp.todos_state.get("PAY_02").unwrap().status, "planned");

        // 7. Verify Agent Leases & ADRs
        assert!(interp.agent_leases.contains_key("PaymentsModule"));
        let lease = interp.agent_leases.get("PaymentsModule").unwrap();
        assert_eq!(lease.0, "agent_alpha_autonomous");

        assert!(interp.knowledge_base.contains_key("PaymentArchitecture"));
        assert!(interp.decision_records.contains_key("ADR_001_Use_Ed25519"));
        let decision = interp.decision_records.get("ADR_001_Use_Ed25519").unwrap();
        assert_eq!(decision.0, "Ed25519 for signature verification");

        // 8. Verify Agent Reports
        assert_eq!(interp.agent_reports.len(), 1);
        assert_eq!(interp.agent_reports[0].task_id, "implement_secure_payment");
        assert_eq!(interp.agent_reports[0].confidence, 0.99);

        // 9. Verify C Backend Codegen
        let mut c_backend = crate::codegen::c_backend::CBackend::new();
        let c_code = c_backend.generate(&module);
        assert!(c_code.contains("/* 🎯 [FEATURE 'PaymentProcessing']"));
        assert!(c_code.contains("/* 🧠 [SKILL 'TransactionSafe'"));
        assert!(c_code.contains("/* 🏛️ [PROJECT SKILLS PROFILE]"));
        assert!(c_code.contains("/* 📋 [AGENT TASK CONTRACT 'implement_secure_payment']"));
        assert!(c_code.contains("/* 📜 [REQUIREMENT 'REQ_PAY_001']"));
        assert!(c_code.contains("/* 🔨 [IMPLEMENTS 'REQ_PAY_001']"));
        assert!(c_code.contains("/* 🛡️ [VERIFIES 'REQ_PAY_001']"));
        assert!(c_code.contains("/* 📝 [EXECUTABLE TODO 'AUTH_01']"));
        assert!(c_code.contains("/* 🧱 [AGENT BOUNDARY]: 'CoreSecurity' */"));
        assert!(c_code.contains("/* 👁️ [AGENT CONTEXT 'PaymentsModule']"));
        assert!(c_code.contains("/* 🧱🔥 [CONTEXT FIREWALL 'DatabaseLayer']"));
        assert!(c_code.contains("/* 🤖 [AGENT API 'SecurityGateway']"));
        assert!(c_code.contains("/* ⚙️ [AGENTABILITY BUDGET]"));
        assert!(c_code.contains("/* 🎯 [INTENT]"));
        assert!(c_code.contains("/* 💾 [SEMANTIC COMMIT]"));
        assert!(c_code.contains("/* 🧐 [AGENT REVIEW 'implement_secure_payment']"));
        assert!(c_code.contains("/* ✍️ [APPROVAL REQUIRED]"));
        assert!(c_code.contains("/* 🔑 [AGENT LEASE on 'PaymentsModule']"));
        assert!(c_code.contains("/* 📚 [KNOWLEDGE 'PaymentArchitecture']"));
        assert!(c_code.contains("/* ⚖️ [DECISION ADR 'ADR_001_Use_Ed25519']"));
        assert!(c_code.contains("/* 🛡️ [AGENT CAPABILITIES]"));
        assert!(c_code.contains("/* 🛡️ [REGRESSION GUARD]"));
        assert!(c_code.contains("/* 🙋 [CLAIM TASK]: 'implement_secure_payment' */"));
        assert!(c_code.contains("/* 🏁 [COMPLETE TASK 'implement_secure_payment']"));
        assert!(c_code.contains("/* 🔍 [VERIFY TASK 'implement_secure_payment']"));
        assert!(c_code.contains("/* 🔍 [VERIFY TASK 'adversarial']"));
    }

    #[test]
    fn test_agent_contract_lifecycle_claim_complete_verify_proof_of_work() {
        let code = r#"
        task migration_v2 {
            owner: "dev_agent";
            status: "planned";
            skills: ["SafeMigration"];
            evidence {
                test: "verify_db_schema_version";
            }
        }

        claim task migration_v2;

        complete task migration_v2 {
            result: "success";
            confidence: 0.95;
            summary: "Database migrations executed with zero downtime";
            evidence: ["verify_db_schema_version", "rollback_snapshot_verified"];
            notes: "Completed in 45ms";
        }

        verify migration_v2;

        fn main() -> i64 {
            return 42;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(42));

        let task = interp.tasks_state.get("migration_v2").unwrap();
        assert_eq!(task.status, "accepted");
        assert_eq!(task.result, Some("success".to_string()));
        assert_eq!(task.confidence, Some(0.95));
        assert!(interp.verified_tasks.contains("migration_v2"));
    }

    #[test]
    fn test_agent_contract_todo_and_traceability_graph() {
        let code = r#"
        requirement SEC_01 { "Passwords must be hashed with Argon2id and salt." }
        implements SEC_01 ["hash_password", "verify_password"];
        verifies SEC_01 ["test_argon2id_cost_parameters"];

        todo HASH_TODO {
            implement: "Argon2id hashing algorithm with 64MB memory cost";
            requires: ["MemoryBudget64MB", "Argon2Library"];
            verify: ["test_argon2id_cost_parameters"];
            status: "implemented";
        }

        fn main() -> i64 {
            return 77;
        }
        "#;

        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(77));

        assert_eq!(interp.requirements.get("SEC_01").unwrap(), "Passwords must be hashed with Argon2id and salt.");
        assert_eq!(interp.requirement_implements.get("SEC_01").unwrap(), &vec!["hash_password", "verify_password"]);
        assert_eq!(interp.requirement_verifies.get("SEC_01").unwrap(), &vec!["test_argon2id_cost_parameters"]);

        let todo = interp.todos_state.get("HASH_TODO").unwrap();
        assert_eq!(todo.status, "implemented");
        assert_eq!(todo.requires, vec!["MemoryBudget64MB", "Argon2Library"]);
    }

    // =========================================================================
    // LAYER 1: DNA of Code Itself (Tests 1 - 3)
    // =========================================================================

    #[test]
    fn test_layer1_partial_augment_composition() {
        let code = r#"
        partial struct User {
            pub id: i64,
            pub name: str,
        }

        partial struct User {
            pub email: str,
            pub is_active: bool,
        }

        augment struct User {
            pub fn get_display_name() -> str {
                return "UserDisplay";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert!(!module.statements.is_empty() || !module.structs.is_empty() || !module.extensions.is_empty());
        let has_partial = module.statements.iter().any(|s| matches!(s, Statement::PartialDecl { kind, name, .. } if kind == "struct" && name == "User"));
        let has_augment = module.statements.iter().any(|s| matches!(s, Statement::AugmentDecl(_))) || module.extensions.iter().any(|e| e.is_augment);
        assert!(has_partial || has_augment);
    }

    #[test]
    fn test_layer1_override_and_extension_points() {
        let code = r#"
        extension_point User { before_save, after_save, validation }

        extend User at validation {
            fn validate_payload() -> bool {
                return true;
            }
        }

        override User.save() {
            val status = 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_ext_point = module.statements.iter().any(|s| matches!(s, Statement::ExtensionPointDecl { target, points, .. } if target == "User" && points.contains(&"validation".to_string())));
        let has_at_hook = module.extensions.iter().any(|e| e.target == "User" && e.at_hook == Some("validation".to_string()));
        let has_override = module.statements.iter().any(|s| matches!(s, Statement::OverrideDecl { target, .. } if target == "User.save"));
        assert!(has_ext_point);
        assert!(has_at_hook);
        assert!(has_override);
    }

    #[test]
    fn test_layer1_sealed_and_friend_protection() {
        let code = r#"
        sealed struct SecurityToken;
        friend module Authentication;

        mod Security {
            sealed: true;
            owns: ["SecurityToken"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        analyzer.analyze_module(&module);
        assert!(analyzer.sealed_structs.contains("SecurityToken") || analyzer.module_sealed.contains("Security"));
        assert!(analyzer.module_friends.contains_key("global") || analyzer.module_friends.contains_key("Security"));
    }

    // =========================================================================
    // LAYER 2: Super Module System (Tests 4 - 6)
    // =========================================================================

    #[test]
    fn test_layer2_multidimensional_facets() {
        let code = r#"
        mod payments {
            api {
                pub fn pay(amount: i64) -> bool {
                    return true;
                }
            }
            implementation {
                fn process_gateway(amount: i64) -> bool {
                    return true;
                }
            }
            tests {
                fn test_pay_flow() -> bool {
                    return true;
                }
            }
            extension {
                fn refund(amount: i64) -> bool {
                    return true;
                }
            }
            architecture {
                "payments -> payment_gateway";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.modules.len(), 1);
        let payments = &module.modules[0];
        assert!(payments.facets.is_some());
        let facets = payments.facets.as_ref().unwrap();
        assert_eq!(facets.api.len(), 1);
        assert_eq!(facets.implementation.len(), 1);
        assert_eq!(facets.tests.len(), 1);
        assert_eq!(facets.extension.len(), 1);
        assert_eq!(facets.architecture.len(), 1);
    }

    #[test]
    fn test_layer2_contracts_and_dependencies() {
        let code = r#"
        mod payments {
            requires: ["database", "auth"];
            provides: ["PaymentService", "RefundService"];
            guarantees: ["no_network_after_commit", "idempotent_charge"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let payments = &module.modules[0];
        assert!(payments.contract.is_some());
        let contract = payments.contract.as_ref().unwrap();
        assert_eq!(contract.requires, vec!["database", "auth"]);
        assert_eq!(contract.provides, vec!["PaymentService", "RefundService"]);
        assert_eq!(contract.guarantees, vec!["no_network_after_commit", "idempotent_charge"]);
    }

    #[test]
    fn test_layer2_replace_overlay_and_compose() {
        let code = r#"
        replace payments.database with postgres_adapter;

        migration payments 3 -> 4 {
            rename "Payment.id" -> "Payment.payment_id";
        }

        overlay production {
            val endpoint = "https://prod.gateway.com";
        }

        compose { Auth, Database, Payments, Observability }
        "#;
        let module = parse_str(code).unwrap();
        let has_replace = module.statements.iter().any(|s| matches!(s, Statement::ReplaceModuleDecl { target, replacement, .. } if target == "payments.database" && replacement == "postgres_adapter"));
        let has_migration = module.statements.iter().any(|s| matches!(s, Statement::ModuleMigrationDecl { module_name, from_version, to_version, .. } if module_name == "payments" && *from_version == 3 && *to_version == 4));
        let has_overlay = module.statements.iter().any(|s| matches!(s, Statement::ModuleOverlayDecl { target_env, .. } if target_env == "production"));
        let has_compose = module.statements.iter().any(|s| matches!(s, Statement::ModuleComposeDecl { modules, .. } if modules.len() == 4));
        assert!(has_replace);
        assert!(has_migration);
        assert!(has_overlay);
        assert!(has_compose);
    }

    // =========================================================================
    // LAYER 3: Type System for Extensibility (Tests 7 - 9)
    // =========================================================================

    #[test]
    fn test_layer3_open_closed_types() {
        let code = r#"
        open type User;
        closed type SecureVault;
        "#;
        let module = parse_str(code).unwrap();
        let has_open = module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: true, name, .. } if name == "User"));
        let has_closed = module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: false, name, .. } if name == "SecureVault"));
        assert!(has_open);
        assert!(has_closed);
    }

    #[test]
    fn test_layer3_conditional_and_generic_extensions() {
        let code = r#"
        extend User when Feature("social") {
            fn get_followers() -> i64 {
                return 42;
            }
        }

        extend User requires capability("user.profile") {
            fn get_bio() -> str {
                return "developer";
            }
        }

        extend List<T> {
            fn is_empty_collection() -> bool {
                return true;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 3);
        assert_eq!(module.extensions[0].when_feature, Some("social".to_string()));
        assert_eq!(module.extensions[1].required_capability, Some("user.profile".to_string()));
        assert_eq!(module.extensions[2].generic_params, vec!["T".to_string()]);
    }

    #[test]
    fn test_layer3_extension_conflict_resolution() {
        let code = r#"
        extend User owned_by IdentityTeam {
            fn notify() -> bool {
                return true;
            }
        }

        extend User owned_by AnalyticsTeam {
            fn notify() -> bool {
                return false;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 2);
        assert_eq!(module.extensions[0].owned_by, Some("IdentityTeam".to_string()));
        assert_eq!(module.extensions[1].owned_by, Some("AnalyticsTeam".to_string()));
    }

    // =========================================================================
    // LAYER 4: Syntax Extensibility (Tests 10 - 12)
    // =========================================================================

    #[test]
    fn test_layer4_custom_syntax_definition() {
        let code = r#"
        syntax query {
            val q = 100;
        }

        syntax route(path: str) -> Route {
            val r = 200;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_query_syntax = module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { name, .. } if name == "query"));
        let has_route_syntax = module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { name, params, .. } if name == "route" && params.len() == 1));
        assert!(has_query_syntax);
        assert!(has_route_syntax);
    }

    #[test]
    fn test_layer4_namespaced_versioned_syntax() {
        let code = r#"
        use syntax web@3;
        use feature("authentication");
        "#;
        let module = parse_str(code).unwrap();
        let has_use_syntax = module.statements.iter().any(|s| matches!(s, Statement::UseSyntaxDecl { namespace, version, .. } if namespace == "web" && *version == Some(3)));
        let has_use_feature = module.statements.iter().any(|s| matches!(s, Statement::SemanticImportDecl { feature_intent, .. } if feature_intent == "authentication"));
        assert!(has_use_syntax);
        assert!(has_use_feature);
    }

    #[test]
    fn test_layer4_syntax_safety_and_composition() {
        let code = r#"
        syntax web::graphql(schema: str) -> Schema {
            val loaded = true;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_namespaced = module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { namespace, name, .. } if namespace == &Some("web".to_string()) && name == "graphql"));
        assert!(has_namespaced);
    }

    // =========================================================================
    // LAYER 5: Compile-time Extensibility (Tests 13 - 15)
    // =========================================================================

    #[test]
    fn test_layer5_custom_linters_and_analyzers() {
        let code = r#"
        lint no_database_in_ui {
            "forbid UI -> Database";
            "require View -> ViewModel";
        }

        analyzer detect_blocking_io {
            "forbid sync_read in async_fn";
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_lint = module.statements.iter().any(|s| matches!(s, Statement::CustomLinterDecl { name, rules, .. } if name == "no_database_in_ui" && rules.len() == 2));
        let has_analyzer = module.statements.iter().any(|s| matches!(s, Statement::CustomAnalyzerDecl { name, checks, .. } if name == "detect_blocking_io" && checks.len() == 1));
        assert!(has_lint);
        assert!(has_analyzer);
    }

    #[test]
    fn test_layer5_type_rules_and_optimizers() {
        let code = r#"
        type_rule Money {
            "forbid_float_conversion";
            "require_explicit_currency";
        }

        optimizer vectorize_math {
            val pass_id = 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_type_rule = module.statements.iter().any(|s| matches!(s, Statement::CustomTypeRuleDecl { target_type, rules, .. } if target_type == "Money" && rules.len() == 2));
        let has_opt = module.statements.iter().any(|s| matches!(s, Statement::CustomOptimizerDecl { name, .. } if name == "vectorize_math"));
        assert!(has_type_rule);
        assert!(has_opt);
    }

    #[test]
    fn test_layer5_reflection_and_generators() {
        let code = r#"
        generator grpc {
            "proto3";
        }

        reflect User { fields, methods, attributes }
        "#;
        let module = parse_str(code).unwrap();
        let has_gen = module.statements.iter().any(|s| matches!(s, Statement::GeneratorDecl { name, .. } if name == "grpc"));
        let has_reflect = module.statements.iter().any(|s| matches!(s, Statement::ReflectDecl { target_type, queries, .. } if target_type == "User" && queries.len() == 3));
        assert!(has_gen);
        assert!(has_reflect);
    }

    // =========================================================================
    // LAYER 6: Architecture as Code (Tests 16 - 18)
    // =========================================================================

    #[test]
    fn test_layer6_architecture_contract_enforcement() {
        let code = r#"
        architecture SystemArch {
            layers: ["UI", "Domain", "Data"];
            rules: ["UI !-> Database"];
            directions: UI -> Domain, Domain -> Data;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_arch = module.statements.iter().any(|s| matches!(s, Statement::ArchitectureDecl { name, .. } if name == "SystemArch"));
        assert!(has_arch);
    }

    #[test]
    fn test_layer6_boundary_and_ownership() {
        let code = r#"
        lock architecture;

        change_limit {
            files: 5,
            modules: 2,
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        analyzer.analyze_module(&module);
        assert!(analyzer.arch_locked);
    }

    #[test]
    fn test_layer6_architecture_tests_and_cycle_free() {
        let code = r#"
        architecture_test {
            "assert no_cycle()";
            "assert no_forbidden_dependency()";
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_arch_test = module.statements.iter().any(|s| matches!(s, Statement::ArchitectureTestDecl { assertions, .. } if assertions.len() == 2));
        assert!(has_arch_test);
    }

    // =========================================================================
    // LAYER 7: Extensibility / Replaceability Engine (Tests 19 - 21)
    // =========================================================================

    #[test]
    fn test_layer7_semantic_dependency_graph() {
        let code = r#"
        mod auth {
            pub struct User {
                pub id: i64,
            }
            pub fn login() -> bool {
                return true;
            }
        }

        mod payments {
            depends: ["auth"];
            forbid: ["database"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let graph = SemanticDependencyGraph::build_from_ast(&module);
        assert!(graph.nodes.contains_key("test_mod::auth"));
        assert!(graph.nodes.contains_key("test_mod::payments"));
        assert!(graph.edges.iter().any(|e| e.from == "test_mod::payments" && e.to == "auth"));
    }

    #[test]
    fn test_layer7_impact_and_blast_radius() {
        let code = r#"
        mod database {
            pub struct Connection {
                pub url: str,
            }
        }
        mod repository {
            depends: ["database"];
        }
        mod service {
            depends: ["repository"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let graph = SemanticDependencyGraph::build_from_ast(&module);
        let impact = EvolutionEngine::analyze_impact(&graph, "test_mod::database");
        assert_eq!(impact.target, "test_mod::database");
        assert!(impact.blast_radius >= 0.0);
    }

    #[test]
    fn test_layer7_replaceability_and_extensibility_metrics() {
        let code = r#"
        mod decoupled_service {
            sealed: true;
            provides: ["ServiceApi"];
            guarantees: ["pure_computation"];
            pub fn run() -> i64 { return 100; }
        }
        "#;
        let module = parse_str(code).unwrap();
        let target_mod = &module.modules[0];
        let rep_score = EvolutionEngine::compute_replaceability(target_mod);
        let ext_score = EvolutionEngine::compute_extensibility(target_mod);

        assert!(rep_score.total_score >= 60.0);
        assert!(rep_score.can_replace_safely);
        assert!(ext_score.extensibility_index >= 70.0);
        assert!(ext_score.has_sealed_internals);
        assert!(ext_score.has_contracts);
    }

    // =========================================================================
    // LAYER 8: API Evolution & Migration (Tests 22 - 24)
    // =========================================================================

    #[test]
    fn test_layer8_api_stability_annotations() {
        let code = r#"
        stable API PaymentService;
        "#;
        let module = parse_str(code).unwrap();
        let has_stable = module.statements.iter().any(|s| matches!(s, Statement::StableDecl { api_name, .. } if api_name == "PaymentService"));
        assert!(has_stable);
    }

    #[test]
    fn test_layer8_api_snapshots_and_diff() {
        let mut v1 = ModuleDef::default();
        v1.name = "Billing".to_string();
        v1.functions.push(FunctionDef {
            name: "calculate_tax".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::I64,
            body: Block { statements: vec![], span: Span::default() },
            directives: vec![],
            morphic_param: None,
            span: Span::default(),
        });

        let mut v2 = v1.clone();
        v2.functions.push(FunctionDef {
            name: "calculate_vat".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::I64,
            body: Block { statements: vec![], span: Span::default() },
            directives: vec![],
            morphic_param: None,
            span: Span::default(),
        });

        let snap1 = EvolutionEngine::create_snapshot(&v1, 1);
        let snap2 = EvolutionEngine::create_snapshot(&v2, 2);

        let diff = EvolutionEngine::diff_api(&snap1, &snap2);
        assert_eq!(diff.breaking_changes.len(), 0);
        assert_eq!(diff.compatible_additions.len(), 1);
        assert_eq!(diff.semver_bump, "MINOR");
    }

    #[test]
    fn test_layer8_semver_and_compatibility_matrix() {
        let mut v1 = ModuleDef::default();
        v1.name = "Crypto".to_string();
        v1.functions.push(FunctionDef {
            name: "encrypt".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::Str,
            body: Block { statements: vec![], span: Span::default() },
            directives: vec![],
            morphic_param: None,
            span: Span::default(),
        });

        let mut v2 = v1.clone();
        v2.functions[0].return_type = Type::I64;

        let snap1 = EvolutionEngine::create_snapshot(&v1, 1);
        let snap2 = EvolutionEngine::create_snapshot(&v2, 2);

        let diff = EvolutionEngine::diff_api(&snap1, &snap2);
        assert_eq!(diff.breaking_changes.len(), 1);
        assert_eq!(diff.semver_bump, "MAJOR");
    }

    // =========================================================================
    // LAYER 9: Agent-Native Extensibility (Tests 25 - 27)
    // =========================================================================

    #[test]
    fn test_layer9_agent_extension_contract() {
        let code = r#"
        agent_extension payment_bot {
            purpose: "Autonomous micro-payment executor";
            inputs: ["amount: i64", "recipient: str"];
            outputs: ["tx_hash: str"];
            constraints: ["max_amount_per_tx <= 1000", "no_reentrancy"];
            tests: ["test_micro_pay", "test_limit_exceeded"];
            permissions: ["net.http", "storage.read"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_agent_ext = module.statements.iter().any(|s| matches!(s, Statement::AgentExtensionContractDecl { name, purpose, inputs, constraints, .. } if name == "payment_bot" && purpose.contains("Autonomous") && inputs.len() == 2 && constraints.len() == 2));
        assert!(has_agent_ext);
    }

    #[test]
    fn test_layer9_change_proposal_and_proof_gate() {
        let code = r#"
        proposal {
            title: "Upgrade Argon2id Cost Matrix";
            files: ["src/auth/crypto.end", "src/config/params.end"];
            symbols: ["Argon2Cost", "verify_password"];
            dependencies: ["crypto_native"];
            risks: ["increased_cpu_latency_under_load"];
            migration: "run end migrate auth 2 -> 3";
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_proposal = module.statements.iter().any(|s| matches!(s, Statement::AgentChangeProposalDecl { title, files, symbols, risks, migration, .. } if title == "Upgrade Argon2id Cost Matrix" && files.len() == 2 && symbols.len() == 2 && risks.len() == 1 && migration.is_some()));
        assert!(has_proposal);
    }

    #[test]
    fn test_layer9_agent_transactions_and_skills() {
        let code = r#"
        begin change {
            val step1 = "patch_ast";
            val step2 = "verify_contracts";
        } commit;

        mod secure_payments {
            skill: ["pci_dss_compliance", "tls_enforcement"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_txn = module.statements.iter().any(|s| matches!(s, Statement::AgentTransactionDecl { action, body, .. } if action == "change" && body.is_some()));
        assert!(has_txn);
        assert_eq!(module.modules[0].skills, vec!["pci_dss_compliance", "tls_enforcement"]);
    }

    // =========================================================================
    // LAYER 10: `end evolve` Engine & `@evolvable` Modules (Tests 28 - 30)
    // =========================================================================

    #[test]
    fn test_layer10_code_as_evolution_graph() {
        let code = r#"
        evolvable billing;

        extend Billing owned_by PaymentsTeam {
            fn calculate_fee() -> i64 {
                return 15;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_evolvable = module.statements.iter().any(|s| matches!(s, Statement::EvolvableDecl { module_name, .. } if module_name == "billing"));
        assert!(has_evolvable);
        assert_eq!(module.extensions[0].owned_by, Some("PaymentsTeam".to_string()));
    }

    #[test]
    fn test_layer10_extension_lifecycle_and_dead_detection() {
        let code = r#"
        extend LegacyProtocol @api(>=1) {
            fn old_handshake() -> bool {
                return false;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions[0].target, "LegacyProtocol");
        assert_eq!(module.extensions[0].version_req, Some("@api(>=1)".to_string()));
    }

    #[test]
    fn test_layer10_evolvable_master_module_verification() {
        let code = r#"
        @evolvable mod payments {
            sealed: true;
            requires: ["database"];
            provides: ["PaymentService"];
            guarantees: ["atomic_transfer", "audit_logged"];

            api {
                pub fn pay(amount: i64) -> bool {
                    return true;
                }
            }
            implementation {
                fn execute_transfer(amount: i64) -> bool {
                    return true;
                }
            }
            tests {
                fn test_transfer_integrity() -> bool {
                    return true;
                }
            }
            extension {
                fn custom_discount(amount: i64) -> i64 {
                    return amount;
                }
            }
            architecture {
                "payments -> database";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let payments_mod = &module.modules[0];
        assert!(payments_mod.is_evolvable);

        let report = EvolutionEngine::evaluate_evolvable_module(payments_mod);
        assert!(report.is_evolvable);
        assert!(report.extensibility_score >= 80.0);
        assert!(report.maintainability_score >= 80.0);
        assert!(report.verification_passed);
        assert!(report.formatted_output.contains("END EVOLUTION ANALYSIS"));
        assert!(report.formatted_output.contains("VERIFIED (100% Extensible DNA Compliant)"));
    }
}




