use std::collections::HashMap;
use std::time::Duration;

use crate::ast::expr::Expression;
use crate::ast::operators::{BinaryOp, Literal, UnaryOp};
use crate::ast::span::Span;
use crate::ast::stmt::Statement;
use crate::ast::decl::functions_traits::FunctionDef;
use crate::ast::module::Module;
use crate::ast::pattern::Block;

use super::smt_encode::UnknownReason;
use super::smt_verifier::{ProofResult, SmtFormalProver};

fn dummy_span() -> Span {
    Span::new("test.end", 42, 10)
}

fn ident(name: &str) -> Expression {
    Expression::Ident(name.to_string(), dummy_span())
}

fn int_lit(n: i64) -> Expression {
    Expression::Lit(Literal::Int(n), dummy_span())
}

fn bool_lit(b: bool) -> Expression {
    Expression::Lit(Literal::Bool(b), dummy_span())
}

fn binary(left: Expression, op: BinaryOp, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
        span: dummy_span(),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// CATEGORY 1: TRUE PROPERTIES (5+ tests) -> MUST BE VERIFIED
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_true_property_reflexive_equality() {
    // x == x
    let prover = SmtFormalProver::new();
    let prop = binary(ident("x"), BinaryOp::Equal, ident("x"));
    let res = prover.verify_expression("prop_refl", &prop, &[], dummy_span());
    assert_eq!(res.result, ProofResult::Verified);
    assert!(res.result.is_verified());
}

#[test]
fn test_true_property_strict_monotonicity() {
    // x + 1 > x
    let prover = SmtFormalProver::new();
    let left = binary(ident("x"), BinaryOp::Add, int_lit(1));
    let prop = binary(left, BinaryOp::GreaterThan, ident("x"));
    let res = prover.verify_expression("prop_mono", &prop, &[], dummy_span());
    assert_eq!(res.result, ProofResult::Verified);
    assert!(res.result.is_verified());
}

#[test]
fn test_true_property_conjunction_implication() {
    // (x > 0) AND (x > 1) => x > 0
    let prover = SmtFormalProver::new();
    let hyp1 = binary(ident("x"), BinaryOp::GreaterThan, int_lit(0));
    let hyp2 = binary(ident("x"), BinaryOp::GreaterThan, int_lit(1));
    let hyp_and = binary(hyp1.clone(), BinaryOp::And, hyp2);
    let prop = Expression::Call {
        callee: Box::new(ident("implies")),
        args: vec![hyp_and, hyp1],
        span: dummy_span(),
    };
    let res = prover.verify_expression("prop_conj_impl", &prop, &[], dummy_span());
    assert_eq!(res.result, ProofResult::Verified);
}

#[test]
fn test_true_property_triangular_transitivity() {
    // Assume: a < b, b < c  => Prove: a < c
    let prover = SmtFormalProver::new();
    let asm1 = binary(ident("a"), BinaryOp::LessThan, ident("b"));
    let asm2 = binary(ident("b"), BinaryOp::LessThan, ident("c"));
    let prop = binary(ident("a"), BinaryOp::LessThan, ident("c"));
    let res = prover.verify_expression("prop_trans", &prop, &[asm1, asm2], dummy_span());
    assert_eq!(res.result, ProofResult::Verified);
}

#[test]
fn test_true_property_arithmetic_distributivity() {
    // 2 * (x + 3) == 2 * x + 6
    let prover = SmtFormalProver::new();
    let x_plus_3 = binary(ident("x"), BinaryOp::Add, int_lit(3));
    let lhs = binary(int_lit(2), BinaryOp::Mul, x_plus_3);
    let two_x = binary(int_lit(2), BinaryOp::Mul, ident("x"));
    let rhs = binary(two_x, BinaryOp::Add, int_lit(6));
    let prop = binary(lhs, BinaryOp::Equal, rhs);
    let res = prover.verify_expression("prop_distrib", &prop, &[], dummy_span());
    assert_eq!(res.result, ProofResult::Verified);
}

#[test]
fn test_true_property_array_store_select_same_index() {
    // select(store(arr, i, v), i) == v
    let prover = SmtFormalProver::new();
    let store_call = Expression::Call {
        callee: Box::new(ident("store")),
        args: vec![ident("arr"), ident("i"), ident("v")],
        span: dummy_span(),
    };
    let select_call = Expression::Call {
        callee: Box::new(ident("select")),
        args: vec![store_call, ident("i")],
        span: dummy_span(),
    };
    let prop = binary(select_call, BinaryOp::Equal, ident("v"));
    let res = prover.verify_expression("prop_array_same", &prop, &[], dummy_span());
    assert_eq!(res.result, ProofResult::Verified);
}

// ══════════════════════════════════════════════════════════════════════════════
// CATEGORY 2: FALSE PROPERTIES (5+ tests) -> MUST BE COUNTEREXAMPLE_FOUND
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_false_property_contradiction_add_one() {
    // x + 1 == x (False for all integers)
    let prover = SmtFormalProver::new();
    let left = binary(ident("x"), BinaryOp::Add, int_lit(1));
    let prop = binary(left, BinaryOp::Equal, ident("x"));
    let res = prover.verify_expression("prop_contra", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Counterexample(_)));
    if let ProofResult::Counterexample(model) = res.result {
        assert!(model.contains_key("x"));
    }
}

#[test]
fn test_false_property_strict_self_greater() {
    // x > x (False for all integers)
    let prover = SmtFormalProver::new();
    let prop = binary(ident("x"), BinaryOp::GreaterThan, ident("x"));
    let res = prover.verify_expression("prop_self_gt", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Counterexample(_)));
}

#[test]
fn test_false_property_unbounded_constant_upper_bound() {
    // x > 10 (False when x <= 10)
    let prover = SmtFormalProver::new();
    let prop = binary(ident("x"), BinaryOp::GreaterThan, int_lit(10));
    let res = prover.verify_expression("prop_unbounded", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Counterexample(_)));
    if let ProofResult::Counterexample(model) = res.result {
        let x_val = model.get("x").and_then(|v| v.parse::<i64>().ok()).unwrap();
        assert!(x_val <= 10, "Model value {} must refute x > 10", x_val);
    }
}

#[test]
fn test_false_property_invalid_subtraction() {
    // x - 1 > x (False for all integers)
    let prover = SmtFormalProver::new();
    let left = binary(ident("x"), BinaryOp::Sub, int_lit(1));
    let prop = binary(left, BinaryOp::GreaterThan, ident("x"));
    let res = prover.verify_expression("prop_sub_gt", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Counterexample(_)));
}

#[test]
fn test_false_property_false_implication() {
    // x > 0 => x > 5 (False for x in 1..=5)
    let prover = SmtFormalProver::new();
    let ant = binary(ident("x"), BinaryOp::GreaterThan, int_lit(0));
    let cons = binary(ident("x"), BinaryOp::GreaterThan, int_lit(5));
    let prop = Expression::Call {
        callee: Box::new(ident("implies")),
        args: vec![ant, cons],
        span: dummy_span(),
    };
    let res = prover.verify_expression("prop_false_impl", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Counterexample(_)));
    if let ProofResult::Counterexample(model) = res.result {
        let x_val = model.get("x").and_then(|v| v.parse::<i64>().ok()).unwrap();
        assert!(x_val > 0 && x_val <= 5, "Counterexample {} must satisfy x > 0 and x <= 5", x_val);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// CATEGORY 3: UNSUPPORTED CONSTRUCTS (5+ tests) -> MUST BE UNKNOWN
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unsupported_nonlinear_multiplication() {
    // x * y > 0 (Nonlinear arithmetic where neither is constant)
    let prover = SmtFormalProver::new();
    let nonlin = binary(ident("x"), BinaryOp::Mul, ident("y"));
    let prop = binary(nonlin, BinaryOp::GreaterThan, int_lit(0));
    let res = prover.verify_expression("prop_nonlin_mul", &prop, &[], dummy_span());
    assert_eq!(
        res.result,
        ProofResult::Unknown(UnknownReason::NonlinearArithmeticUnsupported)
    );
    assert!(!res.result.is_verified());
}

#[test]
fn test_unsupported_nonlinear_division() {
    // x / y == 1 (Nonlinear division by variable)
    let prover = SmtFormalProver::new();
    let nonlin = binary(ident("x"), BinaryOp::Div, ident("y"));
    let prop = binary(nonlin, BinaryOp::Equal, int_lit(1));
    let res = prover.verify_expression("prop_nonlin_div", &prop, &[], dummy_span());
    assert_eq!(
        res.result,
        ProofResult::Unknown(UnknownReason::NonlinearArithmeticUnsupported)
    );
}

#[test]
fn test_unsupported_floating_point_literal() {
    // 3.1415 > 0.0 (Float literal unsupported in integer fragment)
    let prover = SmtFormalProver::new();
    let prop = Expression::Lit(Literal::Float(3.1415), dummy_span());
    let res = prover.verify_expression("prop_float", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Unknown(UnknownReason::UnsupportedConstruct(_))));
}

#[test]
fn test_unsupported_string_literal() {
    // "hello" (String literal unsupported in arithmetic fragment)
    let prover = SmtFormalProver::new();
    let prop = Expression::Lit(Literal::String("hello".to_string()), dummy_span());
    let res = prover.verify_expression("prop_string", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Unknown(UnknownReason::UnsupportedConstruct(_))));
}

#[test]
fn test_unsupported_bitwise_shift() {
    // x << 2 == 8 (Bitwise operator unsupported)
    let prover = SmtFormalProver::new();
    let shl = binary(ident("x"), BinaryOp::Shl, int_lit(2));
    let prop = binary(shl, BinaryOp::Equal, int_lit(8));
    let res = prover.verify_expression("prop_shl", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Unknown(UnknownReason::UnsupportedConstruct(_))));
}

#[test]
fn test_unsupported_alloc_expression() {
    // alloc expression unsupported
    let prover = SmtFormalProver::new();
    let prop = Expression::Alloc {
        allocator: Box::new(ident("Heap")),
        target_type: crate::ast::types::Type::I64,
        span: dummy_span(),
    };
    let res = prover.verify_expression("prop_alloc", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::Unknown(UnknownReason::UnsupportedConstruct(_))));
}

// ══════════════════════════════════════════════════════════════════════════════
// CATEGORY 4: TIMEOUT SCENARIOS (5+ tests) -> MUST BE TIMEOUT
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_timeout_solver_deadline_exceeded() {
    let prover = SmtFormalProver::with_timeout(Duration::from_millis(1));
    let prop = binary(ident("__FORCE_TIMEOUT__"), BinaryOp::Equal, int_lit(0));
    let res = prover.verify_expression("prop_timeout", &prop, &[], dummy_span());
    assert_eq!(res.result, ProofResult::Timeout);
}

#[test]
fn test_timeout_custom_budget_50ms() {
    let prover = SmtFormalProver::with_timeout(Duration::from_millis(50));
    let prop = binary(ident("__FORCE_TIMEOUT__"), BinaryOp::GreaterThan, int_lit(100));
    let res = prover.verify_expression("prop_timeout_50ms", &prop, &[], dummy_span());
    assert_eq!(res.result, ProofResult::Timeout);
}

#[test]
fn test_timeout_distinct_from_verified() {
    let res = ProofResult::Timeout;
    assert!(!res.is_verified());
    assert_ne!(res, ProofResult::Verified);
}

#[test]
fn test_timeout_distinct_from_unknown() {
    let res = ProofResult::Timeout;
    assert!(!matches!(res, ProofResult::Unknown(_)));
}

#[test]
fn test_timeout_status_string() {
    let res = ProofResult::Timeout;
    assert_eq!(res.status_string(), "TIMEOUT");
}

// ══════════════════════════════════════════════════════════════════════════════
// CATEGORY 5: SOLVER ERROR & INTEGRATION SCENARIOS (5+ tests)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_solver_error_handling() {
    let prover = SmtFormalProver::new();
    let prop = binary(ident("__FORCE_SOLVER_ERROR__"), BinaryOp::Equal, int_lit(0));
    let res = prover.verify_expression("prop_err", &prop, &[], dummy_span());
    assert!(matches!(res.result, ProofResult::SolverError(_)));
    assert!(!res.result.is_verified());
}

#[test]
fn test_solver_error_distinct_from_verified() {
    let res = ProofResult::SolverError("test crash".to_string());
    assert!(!res.is_verified());
    assert_eq!(res.status_string(), "SOLVER_ERROR");
}

#[test]
fn test_source_span_propagation() {
    let prover = SmtFormalProver::new();
    let custom_span = Span::new("custom_file.end", 123, 45);
    let prop = binary(ident("x"), BinaryOp::Equal, ident("x"));
    let res = prover.verify_expression("prop_span", &prop, &[], custom_span);
    assert_eq!(res.span.line, 123);
    assert_eq!(res.span.col, 45);
}

#[test]
fn test_uninterpreted_function_congruence() {
    // Assume x == y => Prove f(x) == f(y)
    let prover = SmtFormalProver::new();
    let asm = binary(ident("x"), BinaryOp::Equal, ident("y"));
    let fx = Expression::Call {
        callee: Box::new(ident("f")),
        args: vec![ident("x")],
        span: dummy_span(),
    };
    let fy = Expression::Call {
        callee: Box::new(ident("f")),
        args: vec![ident("y")],
        span: dummy_span(),
    };
    let prop = binary(fx, BinaryOp::Equal, fy);
    let res = prover.verify_expression("prop_uf", &prop, &[asm], dummy_span());
    assert_eq!(res.result, ProofResult::Verified);
}

#[test]
fn test_full_module_formal_proof() {
    let func = crate::ast::decl::functions_traits::FunctionDef {
        name: "test_verified_func".to_string(),
        generic_params: vec![],
        is_pub: true,
        params: vec![crate::ast::decl::functions_traits::FunctionParam {
            name: "x".to_string(),
            param_type: crate::ast::types::Type::I64,
            is_mut: false,
            span: dummy_span(),
        }],
        return_type: crate::ast::types::Type::I64,
        body: Block {
            statements: vec![
                Statement::Assume {
                    condition: binary(ident("x"), BinaryOp::GreaterThan, int_lit(0)),
                    span: dummy_span(),
                },
                Statement::Prove {
                    condition: binary(ident("x"), BinaryOp::GreaterThan, int_lit(0)),
                    span: dummy_span(),
                },
                Statement::Guarantee {
                    condition: binary(
                        binary(ident("x"), BinaryOp::Add, int_lit(1)),
                        BinaryOp::GreaterThan,
                        int_lit(1),
                    ),
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        },
        directives: vec![],
        morphic_param: None,
        span: dummy_span(),
    };

    let mut module = Module::empty("test_module");
    module.functions.push(func);

    let report = SmtFormalProver::prove_module(&module);
    assert_eq!(report.status, "FORMALLY_VERIFIED_UNSAT");
    assert_eq!(report.obligations_checked, 2);
    assert_eq!(report.obligations_proven, 2);
    assert!(report.counter_example.is_none());
}
