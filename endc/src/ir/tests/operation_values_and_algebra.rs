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

