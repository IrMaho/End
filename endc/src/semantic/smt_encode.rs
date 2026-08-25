use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::ast::expr::Expression;
use crate::ast::operators::{BinaryOp, Literal, UnaryOp};
use crate::ast::span::Span;
use crate::ast::types::Type;

/// Explicit classification of reasons why a property cannot be verified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownReason {
    UnsupportedConstruct(String),
    QuantifiersUnsupported,
    NonlinearArithmeticUnsupported,
    SolverReturnedUnknown(String),
}

impl std::fmt::Display for UnknownReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnknownReason::UnsupportedConstruct(c) => write!(f, "UNSUPPORTED_CONSTRUCT: {}", c),
            UnknownReason::QuantifiersUnsupported => write!(f, "QUANTIFIERS_UNSUPPORTED"),
            UnknownReason::NonlinearArithmeticUnsupported => {
                write!(f, "NONLINEAR_ARITHMETIC_UNSUPPORTED")
            }
            UnknownReason::SolverReturnedUnknown(r) => write!(f, "SOLVER_RETURNED_UNKNOWN: {}", r),
        }
    }
}

/// Supported SMT data types in the verified fragment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmtType {
    Int,
    Bool,
    Array(Box<SmtType>, Box<SmtType>),
    Function(Vec<SmtType>, Box<SmtType>),
}

impl std::fmt::Display for SmtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmtType::Int => write!(f, "Int"),
            SmtType::Bool => write!(f, "Bool"),
            SmtType::Array(idx, elem) => write!(f, "(Array {} {})", idx, elem),
            SmtType::Function(params, ret) => {
                let params_str: Vec<_> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "({} -> {})", params_str.join(" "), ret)
            }
        }
    }
}

/// SMT AST representation for Linear Integer Arithmetic + Booleans + Arrays + UF.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmtExpr {
    IntConst(i64),
    BoolConst(bool),
    Var(String),
    Add(Vec<SmtExpr>),
    Sub(Box<SmtExpr>, Box<SmtExpr>),
    Mul(Box<SmtExpr>, Box<SmtExpr>),
    Div(Box<SmtExpr>, Box<SmtExpr>),
    Mod(Box<SmtExpr>, Box<SmtExpr>),
    Neg(Box<SmtExpr>),
    Eq(Box<SmtExpr>, Box<SmtExpr>),
    Distinct(Box<SmtExpr>, Box<SmtExpr>),
    Lt(Box<SmtExpr>, Box<SmtExpr>),
    Le(Box<SmtExpr>, Box<SmtExpr>),
    Gt(Box<SmtExpr>, Box<SmtExpr>),
    Ge(Box<SmtExpr>, Box<SmtExpr>),
    And(Vec<SmtExpr>),
    Or(Vec<SmtExpr>),
    Not(Box<SmtExpr>),
    Xor(Box<SmtExpr>, Box<SmtExpr>),
    Implies(Box<SmtExpr>, Box<SmtExpr>),
    Ite(Box<SmtExpr>, Box<SmtExpr>, Box<SmtExpr>),
    Select {
        array: Box<SmtExpr>,
        index: Box<SmtExpr>,
    },
    Store {
        array: Box<SmtExpr>,
        index: Box<SmtExpr>,
        value: Box<SmtExpr>,
    },
    FuncApp {
        name: String,
        args: Vec<SmtExpr>,
    },
}

impl SmtExpr {
    /// Check whether this SMT expression is a constant integer.
    pub fn is_int_const(&self) -> bool {
        match self {
            SmtExpr::IntConst(_) => true,
            SmtExpr::Neg(inner) => inner.is_int_const(),
            _ => false,
        }
    }

    /// Extract constant integer value if applicable.
    pub fn as_int_const(&self) -> Option<i64> {
        match self {
            SmtExpr::IntConst(n) => Some(*n),
            SmtExpr::Neg(inner) => inner.as_int_const().map(|n| -n),
            _ => None,
        }
    }

    /// Convert to standard SMT-LIB2 format string.
    pub fn to_smt2(&self) -> String {
        match self {
            SmtExpr::IntConst(n) => {
                if *n < 0 {
                    format!("(- {})", -*n)
                } else {
                    n.to_string()
                }
            }
            SmtExpr::BoolConst(b) => if *b { "true".to_string() } else { "false".to_string() },
            SmtExpr::Var(v) => sanitize_smt_ident(v),
            SmtExpr::Add(terms) => {
                if terms.is_empty() {
                    "0".to_string()
                } else if terms.len() == 1 {
                    terms[0].to_smt2()
                } else {
                    let formatted: Vec<_> = terms.iter().map(|t| t.to_smt2()).collect();
                    format!("(+ {})", formatted.join(" "))
                }
            }
            SmtExpr::Sub(a, b) => format!("(- {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Mul(a, b) => format!("(* {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Div(a, b) => format!("(div {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Mod(a, b) => format!("(mod {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Neg(a) => format!("(- {})", a.to_smt2()),
            SmtExpr::Eq(a, b) => format!("(= {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Distinct(a, b) => format!("(distinct {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Lt(a, b) => format!("(< {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Le(a, b) => format!("(<= {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Gt(a, b) => format!("(> {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Ge(a, b) => format!("(>= {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::And(terms) => {
                if terms.is_empty() {
                    "true".to_string()
                } else if terms.len() == 1 {
                    terms[0].to_smt2()
                } else {
                    let formatted: Vec<_> = terms.iter().map(|t| t.to_smt2()).collect();
                    format!("(and {})", formatted.join(" "))
                }
            }
            SmtExpr::Or(terms) => {
                if terms.is_empty() {
                    "false".to_string()
                } else if terms.len() == 1 {
                    terms[0].to_smt2()
                } else {
                    let formatted: Vec<_> = terms.iter().map(|t| t.to_smt2()).collect();
                    format!("(or {})", formatted.join(" "))
                }
            }
            SmtExpr::Not(a) => format!("(not {})", a.to_smt2()),
            SmtExpr::Xor(a, b) => format!("(xor {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Implies(a, b) => format!("(=> {} {})", a.to_smt2(), b.to_smt2()),
            SmtExpr::Ite(c, t, e) => {
                format!("(ite {} {} {})", c.to_smt2(), t.to_smt2(), e.to_smt2())
            }
            SmtExpr::Select { array, index } => {
                format!("(select {} {})", array.to_smt2(), index.to_smt2())
            }
            SmtExpr::Store { array, index, value } => {
                format!(
                    "(store {} {} {})",
                    array.to_smt2(),
                    index.to_smt2(),
                    value.to_smt2()
                )
            }
            SmtExpr::FuncApp { name, args } => {
                let formatted: Vec<_> = args.iter().map(|a| a.to_smt2()).collect();
                if formatted.is_empty() {
                    sanitize_smt_ident(name)
                } else {
                    format!("({} {})", sanitize_smt_ident(name), formatted.join(" "))
                }
            }
        }
    }

    /// Collect all referenced variables and infer/detect their sorts.
    pub fn collect_var_types(&self, vars: &mut HashMap<String, SmtType>) {
        match self {
            SmtExpr::Var(v) => {
                vars.entry(v.clone()).or_insert(SmtType::Int);
            }
            SmtExpr::Add(terms) | SmtExpr::And(terms) | SmtExpr::Or(terms) => {
                for t in terms {
                    t.collect_var_types(vars);
                }
            }
            SmtExpr::Sub(a, b)
            | SmtExpr::Mul(a, b)
            | SmtExpr::Div(a, b)
            | SmtExpr::Mod(a, b)
            | SmtExpr::Eq(a, b)
            | SmtExpr::Distinct(a, b)
            | SmtExpr::Lt(a, b)
            | SmtExpr::Le(a, b)
            | SmtExpr::Gt(a, b)
            | SmtExpr::Ge(a, b)
            | SmtExpr::Xor(a, b)
            | SmtExpr::Implies(a, b) => {
                a.collect_var_types(vars);
                b.collect_var_types(vars);
            }
            SmtExpr::Neg(a) | SmtExpr::Not(a) => {
                a.collect_var_types(vars);
            }
            SmtExpr::Ite(c, t, e) => {
                c.collect_var_types(vars);
                t.collect_var_types(vars);
                e.collect_var_types(vars);
            }
            SmtExpr::Select { array, index } => {
                if let SmtExpr::Var(arr_var) = array.as_ref() {
                    vars.insert(
                        arr_var.clone(),
                        SmtType::Array(Box::new(SmtType::Int), Box::new(SmtType::Int)),
                    );
                } else {
                    array.collect_var_types(vars);
                }
                index.collect_var_types(vars);
            }
            SmtExpr::Store { array, index, value } => {
                if let SmtExpr::Var(arr_var) = array.as_ref() {
                    vars.insert(
                        arr_var.clone(),
                        SmtType::Array(Box::new(SmtType::Int), Box::new(SmtType::Int)),
                    );
                } else {
                    array.collect_var_types(vars);
                }
                index.collect_var_types(vars);
                value.collect_var_types(vars);
            }
            SmtExpr::FuncApp { args, .. } => {
                for a in args {
                    a.collect_var_types(vars);
                }
            }
            SmtExpr::IntConst(_) | SmtExpr::BoolConst(_) => {}
        }
    }

    /// Collect all uninterpreted functions and their arity.
    pub fn collect_functions(&self, funcs: &mut HashMap<String, usize>) {
        match self {
            SmtExpr::FuncApp { name, args } => {
                funcs.insert(name.clone(), args.len());
                for a in args {
                    a.collect_functions(funcs);
                }
            }
            SmtExpr::Add(terms) | SmtExpr::And(terms) | SmtExpr::Or(terms) => {
                for t in terms {
                    t.collect_functions(funcs);
                }
            }
            SmtExpr::Sub(a, b)
            | SmtExpr::Mul(a, b)
            | SmtExpr::Div(a, b)
            | SmtExpr::Mod(a, b)
            | SmtExpr::Eq(a, b)
            | SmtExpr::Distinct(a, b)
            | SmtExpr::Lt(a, b)
            | SmtExpr::Le(a, b)
            | SmtExpr::Gt(a, b)
            | SmtExpr::Ge(a, b)
            | SmtExpr::Xor(a, b)
            | SmtExpr::Implies(a, b) => {
                a.collect_functions(funcs);
                b.collect_functions(funcs);
            }
            SmtExpr::Neg(a) | SmtExpr::Not(a) => {
                a.collect_functions(funcs);
            }
            SmtExpr::Ite(c, t, e) => {
                c.collect_functions(funcs);
                t.collect_functions(funcs);
                e.collect_functions(funcs);
            }
            SmtExpr::Select { array, index } => {
                array.collect_functions(funcs);
                index.collect_functions(funcs);
            }
            SmtExpr::Store { array, index, value } => {
                array.collect_functions(funcs);
                index.collect_functions(funcs);
                value.collect_functions(funcs);
            }
            SmtExpr::Var(_) | SmtExpr::IntConst(_) | SmtExpr::BoolConst(_) => {}
        }
    }
}

/// SMT AST Encoder that translates End AST expressions into SmtExpr.
pub struct SmtEncoder {
    pub symbol_table: HashMap<String, SmtType>,
    pub function_signatures: HashMap<String, (Vec<SmtType>, SmtType)>,
}

impl SmtEncoder {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            function_signatures: HashMap::new(),
        }
    }

    pub fn with_context(context: &HashMap<String, SmtType>) -> Self {
        Self {
            symbol_table: context.clone(),
            function_signatures: HashMap::new(),
        }
    }

    /// Declare a variable with an SMT type.
    pub fn declare_var(&mut self, name: &str, smt_type: SmtType) {
        self.symbol_table.insert(name.to_string(), smt_type);
    }

    /// Declare an uninterpreted function with argument types and return type.
    pub fn declare_function(&mut self, name: &str, params: Vec<SmtType>, ret: SmtType) {
        self.function_signatures.insert(name.to_string(), (params, ret));
    }

    /// Translate an End AST Expression into an SmtExpr.
    pub fn encode(&self, expr: &Expression) -> Result<SmtExpr, UnknownReason> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => Ok(SmtExpr::IntConst(*n)),
            Expression::Lit(Literal::Bool(b), _) => Ok(SmtExpr::BoolConst(*b)),
            Expression::Lit(Literal::Float(f), span) => Err(UnknownReason::UnsupportedConstruct(
                format!("Floating point literal '{}' at line {}", f, span.line),
            )),
            Expression::Lit(Literal::String(s), span) => Err(UnknownReason::UnsupportedConstruct(
                format!("String literal \"{}\" at line {}", s, span.line),
            )),
            Expression::Lit(Literal::Null, span) => Err(UnknownReason::UnsupportedConstruct(
                format!("Null literal at line {}", span.line),
            )),

            Expression::Ident(name, _) => {
                // If not in symbol table, infer as Int by default
                Ok(SmtExpr::Var(name.clone()))
            }

            Expression::Unary { op, expr: inner, span } => {
                let encoded_inner = self.encode(inner)?;
                match op {
                    UnaryOp::Not => Ok(SmtExpr::Not(Box::new(encoded_inner))),
                    UnaryOp::Negate => Ok(SmtExpr::Neg(Box::new(encoded_inner))),
                    UnaryOp::AddressOf | UnaryOp::Deref | UnaryOp::BitNot => {
                        Err(UnknownReason::UnsupportedConstruct(format!(
                            "Unary pointer/bitwise operator {:?} at line {}",
                            op, span.line
                        )))
                    }
                }
            }

            Expression::Binary { left, op, right, span } => {
                let l = self.encode(left)?;
                let r = self.encode(right)?;

                match op {
                    BinaryOp::Add => Ok(SmtExpr::Add(vec![l, r])),
                    BinaryOp::Sub => Ok(SmtExpr::Sub(Box::new(l), Box::new(r))),
                    BinaryOp::Mul => {
                        // Linear Integer Arithmetic check: at least one operand MUST be constant
                        if l.is_int_const() || r.is_int_const() {
                            Ok(SmtExpr::Mul(Box::new(l), Box::new(r)))
                        } else {
                            Err(UnknownReason::NonlinearArithmeticUnsupported)
                        }
                    }
                    BinaryOp::Div => {
                        // Div by constant integer is allowed in linear arithmetic
                        if r.is_int_const() && r.as_int_const() != Some(0) {
                            Ok(SmtExpr::Div(Box::new(l), Box::new(r)))
                        } else {
                            Err(UnknownReason::NonlinearArithmeticUnsupported)
                        }
                    }
                    BinaryOp::Mod => {
                        if r.is_int_const() && r.as_int_const() != Some(0) {
                            Ok(SmtExpr::Mod(Box::new(l), Box::new(r)))
                        } else {
                            Err(UnknownReason::NonlinearArithmeticUnsupported)
                        }
                    }
                    BinaryOp::Equal => Ok(SmtExpr::Eq(Box::new(l), Box::new(r))),
                    BinaryOp::NotEqual => Ok(SmtExpr::Distinct(Box::new(l), Box::new(r))),
                    BinaryOp::LessThan => Ok(SmtExpr::Lt(Box::new(l), Box::new(r))),
                    BinaryOp::LessEqual => Ok(SmtExpr::Le(Box::new(l), Box::new(r))),
                    BinaryOp::GreaterThan => Ok(SmtExpr::Gt(Box::new(l), Box::new(r))),
                    BinaryOp::GreaterEqual => Ok(SmtExpr::Ge(Box::new(l), Box::new(r))),
                    BinaryOp::And => Ok(SmtExpr::And(vec![l, r])),
                    BinaryOp::Or => Ok(SmtExpr::Or(vec![l, r])),
                    BinaryOp::BitXor => Ok(SmtExpr::Xor(Box::new(l), Box::new(r))),
                    BinaryOp::Shl | BinaryOp::Shr | BinaryOp::BitAnd | BinaryOp::BitOr => {
                        Err(UnknownReason::UnsupportedConstruct(format!(
                            "Bitwise operation {:?} at line {}",
                            op, span.line
                        )))
                    }
                }
            }

            Expression::Index { array, index, .. } => {
                let arr = self.encode(array)?;
                let idx = self.encode(index)?;
                Ok(SmtExpr::Select {
                    array: Box::new(arr),
                    index: Box::new(idx),
                })
            }

            Expression::Call { callee, args, span } => {
                if let Expression::Ident(fname, _) = callee.as_ref() {
                    match fname.as_str() {
                        "select" if args.len() == 2 => {
                            let arr = self.encode(&args[0])?;
                            let idx = self.encode(&args[1])?;
                            Ok(SmtExpr::Select {
                                array: Box::new(arr),
                                index: Box::new(idx),
                            })
                        }
                        "store" if args.len() == 3 => {
                            let arr = self.encode(&args[0])?;
                            let idx = self.encode(&args[1])?;
                            let val = self.encode(&args[2])?;
                            Ok(SmtExpr::Store {
                                array: Box::new(arr),
                                index: Box::new(idx),
                                value: Box::new(val),
                            })
                        }
                        "implies" if args.len() == 2 => {
                            let ant = self.encode(&args[0])?;
                            let cons = self.encode(&args[1])?;
                            Ok(SmtExpr::Implies(Box::new(ant), Box::new(cons)))
                        }
                        "ite" if args.len() == 3 => {
                            let cond = self.encode(&args[0])?;
                            let then_branch = self.encode(&args[1])?;
                            let else_branch = self.encode(&args[2])?;
                            Ok(SmtExpr::Ite(
                                Box::new(cond),
                                Box::new(then_branch),
                                Box::new(else_branch),
                            ))
                        }
                        _ => {
                            // Uninterpreted function application
                            let mut encoded_args = Vec::new();
                            for a in args {
                                encoded_args.push(self.encode(a)?);
                            }
                            Ok(SmtExpr::FuncApp {
                                name: fname.clone(),
                                args: encoded_args,
                            })
                        }
                    }
                } else {
                    Err(UnknownReason::UnsupportedConstruct(format!(
                        "Complex callee expression at line {}",
                        span.line
                    )))
                }
            }

            // Quantifiers explicitly unsupported in this bounded fragment
            Expression::StructInit { name, .. } => Err(UnknownReason::UnsupportedConstruct(
                format!("Struct initialization of '{}'", name),
            )),
            Expression::EnumInit { variant_name, .. } => Err(
                UnknownReason::UnsupportedConstruct(format!("Enum variant '{}'", variant_name)),
            ),
            Expression::Alloc { .. } => {
                Err(UnknownReason::UnsupportedConstruct("Alloc expression".to_string()))
            }
            Expression::Promote { .. } => {
                Err(UnknownReason::UnsupportedConstruct("Promote expression".to_string()))
            }
            Expression::Catch { .. } => {
                Err(UnknownReason::UnsupportedConstruct("Catch expression".to_string()))
            }
            Expression::Match { .. } => {
                Err(UnknownReason::UnsupportedConstruct("Match expression".to_string()))
            }
            Expression::Block(..) => {
                Err(UnknownReason::UnsupportedConstruct("Block expression".to_string()))
            }
            _ => Err(UnknownReason::UnsupportedConstruct(format!(
                "Unsupported AST expression: {:?}",
                expr
            ))),
        }
    }
}

/// Helper function to sanitize variable/function identifiers for SMT-LIB2 format.
pub fn sanitize_smt_ident(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if sanitized.is_empty() {
        "v_empty".to_string()
    } else {
        sanitized
    }
}
