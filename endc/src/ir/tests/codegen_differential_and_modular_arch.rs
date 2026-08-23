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

