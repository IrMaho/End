use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::ast::decl::functions_traits::FunctionDef;
use crate::ast::expr::Expression;
use crate::ast::module::Module;
use crate::ast::span::Span;
use crate::ast::stmt::Statement;

use super::smt_encode::{sanitize_smt_ident, SmtEncoder, SmtExpr, SmtType, UnknownReason};
use super::smt_solver::{RawSolverResult, SmtSolverEngine};

/// Result of formal verification for an individual obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofResult {
    /// Formally proven: negation of property is UNSAT.
    Verified,
    /// Property is false: counterexample model found.
    Counterexample(HashMap<String, String>),
    /// Property contains unsupported constructs or solver returned unknown.
    Unknown(UnknownReason),
    /// Solver exceeded time budget.
    Timeout,
    /// Solver encountered an internal crash or process failure.
    SolverError(String),
}

impl ProofResult {
    pub fn is_verified(&self) -> bool {
        matches!(self, ProofResult::Verified)
    }

    pub fn status_string(&self) -> &'static str {
        match self {
            ProofResult::Verified => "VERIFIED",
            ProofResult::Counterexample(_) => "COUNTEREXAMPLE_FOUND",
            ProofResult::Unknown(_) => "UNKNOWN",
            ProofResult::Timeout => "TIMEOUT",
            ProofResult::SolverError(_) => "SOLVER_ERROR",
        }
    }
}

/// Kind of verification obligation in End language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObligationKind {
    Prove,
    Invariant,
    Guarantee,
    VerifyBlock,
}

/// A formal verification obligation with source tracking and context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    pub name: String,
    pub source_span: Span,
    pub kind: ObligationKind,
    pub property: Expression,
    pub assumptions: Vec<Expression>,
    pub local_context: HashMap<String, SmtType>,
}

/// Individual verification outcome for an obligation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationResult {
    pub obligation_name: String,
    pub span: Span,
    pub result: ProofResult,
    pub smt_query: String,
    pub duration_ms: u64,
}

/// Comprehensive formal verification report for a module or test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtProofReport {
    pub status: String,
    pub solver_target: String,
    pub obligations_checked: usize,
    pub obligations_proven: usize,
    pub results: Vec<ObligationResult>,
    pub smt_lib_script: String,
    pub counter_example: Option<String>,
    pub execution_evidence: Option<String>,
}

/// Formal SMT Prover pipeline for End language.
pub struct SmtFormalProver {
    solver: SmtSolverEngine,
}

impl Default for SmtFormalProver {
    fn default() -> Self {
        Self::new()
    }
}

impl SmtFormalProver {
    pub fn new() -> Self {
        Self {
            solver: SmtSolverEngine::new(),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            solver: SmtSolverEngine::new().with_timeout(timeout),
        }
    }

    /// Prove all formal verification obligations across a module.
    pub fn prove_module(module: &Module) -> SmtProofReport {
        let prover = SmtFormalProver::new();
        prover.verify_module(module)
    }

    /// Instance method to verify all obligations in a module.
    pub fn verify_module(&self, module: &Module) -> SmtProofReport {
        let mut obligations = Vec::new();

        for func in &module.functions {
            let mut current_assumptions = Vec::new();
            let mut func_context = HashMap::new();

            for p in &func.params {
                func_context.insert(p.name.clone(), SmtType::Int);
            }

            for (idx, stmt) in func.body.statements.iter().enumerate() {
                match stmt {
                    Statement::Assume { condition, .. } => {
                        current_assumptions.push(condition.clone());
                    }
                    Statement::Prove { condition, span } => {
                        obligations.push(Obligation {
                            name: format!("{}_prove_stmt_{}", func.name, idx),
                            source_span: span.clone(),
                            kind: ObligationKind::Prove,
                            property: condition.clone(),
                            assumptions: current_assumptions.clone(),
                            local_context: func_context.clone(),
                        });
                    }
                    Statement::Invariant { condition, span } => {
                        obligations.push(Obligation {
                            name: format!("{}_invariant_stmt_{}", func.name, idx),
                            source_span: span.clone(),
                            kind: ObligationKind::Invariant,
                            property: condition.clone(),
                            assumptions: current_assumptions.clone(),
                            local_context: func_context.clone(),
                        });
                    }
                    Statement::Guarantee { condition, span } => {
                        obligations.push(Obligation {
                            name: format!("{}_guarantee_stmt_{}", func.name, idx),
                            source_span: span.clone(),
                            kind: ObligationKind::Guarantee,
                            property: condition.clone(),
                            assumptions: current_assumptions.clone(),
                            local_context: func_context.clone(),
                        });
                    }
                    Statement::VarDecl {
                        name,
                        initializer,
                        span,
                        ..
                    } => {
                        func_context.insert(name.clone(), SmtType::Int);
                        if let Some(init) = initializer {
                            // Synthesize assignment assumption: name == init
                            let assign_expr = Expression::Binary {
                                left: Box::new(Expression::Ident(name.clone(), span.clone())),
                                op: crate::ast::operators::BinaryOp::Equal,
                                right: Box::new(init.clone()),
                                span: span.clone(),
                            };
                            current_assumptions.push(assign_expr);
                        }
                    }
                    _ => {}
                }
            }
        }

        if obligations.is_empty() {
            return SmtProofReport {
                status: "FORMALLY_VERIFIED_UNSAT".to_string(),
                solver_target: "Z3 / SMT-LIB2 Standard QF_LIA".to_string(),
                obligations_checked: 0,
                obligations_proven: 0,
                results: Vec::new(),
                smt_lib_script: ";; No formal obligations in module\n".to_string(),
                counter_example: None,
                execution_evidence: Some("No obligations checked".to_string()),
            };
        }

        let mut verified_count = 0;
        let mut first_counterexample = None;
        let mut aggregate_script = String::new();
        let mut results = Vec::new();

        aggregate_script.push_str(";; End Formal SMT-LIB2 Verification Suite\n(set-logic QF_LIA)\n\n");

        for obl in &obligations {
            let res = self.prove_obligation(obl);
            aggregate_script.push_str(&res.smt_query);
            aggregate_script.push_str("\n\n");

            if res.result.is_verified() {
                verified_count += 1;
            } else if let ProofResult::Counterexample(ref model) = res.result {
                if first_counterexample.is_none() {
                    first_counterexample = Some(format!("{:?}", model));
                }
            }

            results.push(res);
        }

        let total = obligations.len();
        let status = if verified_count == total {
            "FORMALLY_VERIFIED_UNSAT".to_string()
        } else if first_counterexample.is_some() {
            "COUNTER_EXAMPLE_FOUND".to_string()
        } else if results.iter().any(|r| matches!(r.result, ProofResult::Timeout)) {
            "TIMEOUT".to_string()
        } else if results.iter().any(|r| matches!(r.result, ProofResult::SolverError(_))) {
            "SOLVER_ERROR".to_string()
        } else {
            "UNKNOWN".to_string()
        };

        let evidence_summary = format!(
            "Executed {} formal SMT obligations: {} VERIFIED, {} FAILED",
            total,
            verified_count,
            total - verified_count
        );

        SmtProofReport {
            status,
            solver_target: "Z3 / SMT-LIB2 Standard QF_LIA".to_string(),
            obligations_checked: total,
            obligations_proven: verified_count,
            results,
            smt_lib_script: aggregate_script,
            counter_example: first_counterexample,
            execution_evidence: Some(evidence_summary),
        }
    }

    /// Prove an individual formal obligation by encoding into SMT-LIB2 and invoking real solver.
    pub fn prove_obligation(&self, obligation: &Obligation) -> ObligationResult {
        let start_time = Instant::now();
        let mut encoder = SmtEncoder::with_context(&obligation.local_context);

        // 1. Encode Property
        let encoded_prop = match encoder.encode(&obligation.property) {
            Ok(p) => p,
            Err(reason) => {
                return ObligationResult {
                    obligation_name: obligation.name.clone(),
                    span: obligation.source_span.clone(),
                    result: ProofResult::Unknown(reason),
                    smt_query: format!(";; Encoding failed for obligation {}\n", obligation.name),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        // 2. Encode Assumptions
        let mut encoded_assumptions = Vec::new();
        for a in &obligation.assumptions {
            match encoder.encode(a) {
                Ok(ea) => encoded_assumptions.push(ea),
                Err(reason) => {
                    return ObligationResult {
                        obligation_name: obligation.name.clone(),
                        span: obligation.source_span.clone(),
                        result: ProofResult::Unknown(reason),
                        smt_query: format!(";; Encoding failed in assumptions for {}\n", obligation.name),
                        duration_ms: start_time.elapsed().as_millis() as u64,
                    };
                }
            }
        }

        // 3. Build SMT-LIB2 Script
        let mut script = String::new();
        script.push_str(&format!(";; Obligation: {}\n", obligation.name));
        script.push_str(&format!(";; Span: Line {}, Col {}\n", obligation.source_span.line, obligation.source_span.col));

        // Collect all referenced variables and their sorts
        let mut referenced_vars = HashMap::new();
        encoded_prop.collect_var_types(&mut referenced_vars);
        for a in &encoded_assumptions {
            a.collect_var_types(&mut referenced_vars);
        }

        // Apply any local context overrides
        for (k, v) in &obligation.local_context {
            referenced_vars.insert(k.clone(), v.clone());
        }

        // Collect uninterpreted functions
        let mut referenced_funcs = HashMap::new();
        encoded_prop.collect_functions(&mut referenced_funcs);
        for a in &encoded_assumptions {
            a.collect_functions(&mut referenced_funcs);
        }

        // Declare uninterpreted functions
        for (fname, arity) in &referenced_funcs {
            let int_args = vec!["Int"; *arity].join(" ");
            script.push_str(&format!(
                "(declare-fun {} ({}) Int)\n",
                sanitize_smt_ident(fname),
                int_args
            ));
        }

        // Declare variables
        for (var, smt_type) in &referenced_vars {
            script.push_str(&format!(
                "(declare-const {} {})\n",
                sanitize_smt_ident(var),
                smt_type
            ));
        }

        // Assert all assumptions
        for a in &encoded_assumptions {
            script.push_str(&format!("(assert {})\n", a.to_smt2()));
        }

        // Assert NOT(property) for UNSAT refutation proof
        let negated_prop = SmtExpr::Not(Box::new(encoded_prop));
        script.push_str(&format!("(assert {})\n", negated_prop.to_smt2()));
        script.push_str("(check-sat)\n");

        let vars_vec: Vec<String> = referenced_vars.keys().cloned().collect();

        // 4. Invoke Real SMT Solver
        let raw_result = self.solver.solve(&script, &vars_vec);
        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        // 5. Interpret Result
        let proof_result = match raw_result {
            RawSolverResult::Unsat => ProofResult::Verified,
            RawSolverResult::Sat(model) => ProofResult::Counterexample(model),
            RawSolverResult::Unknown(r) => ProofResult::Unknown(UnknownReason::SolverReturnedUnknown(r)),
            RawSolverResult::Timeout => ProofResult::Timeout,
            RawSolverResult::Error(e) => ProofResult::SolverError(e),
        };

        ObligationResult {
            obligation_name: obligation.name.clone(),
            span: obligation.source_span.clone(),
            result: proof_result,
            smt_query: script,
            duration_ms: elapsed_ms,
        }
    }

    /// Convenience helper to verify an expression with optional assumptions.
    pub fn verify_expression(
        &self,
        name: &str,
        prop: &Expression,
        assumptions: &[Expression],
        span: Span,
    ) -> ObligationResult {
        let obl = Obligation {
            name: name.to_string(),
            source_span: span,
            kind: ObligationKind::Prove,
            property: prop.clone(),
            assumptions: assumptions.to_vec(),
            local_context: HashMap::new(),
        };
        self.prove_obligation(&obl)
    }
}
