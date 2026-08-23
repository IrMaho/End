use super::SemanticAnalyzer;
use crate::ast::{BinaryOp, Expression, Literal, UnaryOp};

impl SemanticAnalyzer {
    pub fn eval_static_const_int(&self, expr: &Expression) -> Option<i64> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => Some(*n),
            Expression::Binary { left, op, right, .. } => {
                let l = self.eval_static_const_int(left)?;
                let r = self.eval_static_const_int(right)?;
                match op {
                    BinaryOp::Add => Some(l + r),
                    BinaryOp::Sub => Some(l - r),
                    BinaryOp::Mul => Some(l * r),
                    BinaryOp::Div if r != 0 => Some(l / r),
                    BinaryOp::Mod if r != 0 => Some(l % r),
                    BinaryOp::BitAnd => Some(l & r),
                    BinaryOp::BitOr => Some(l | r),
                    BinaryOp::BitXor => Some(l ^ r),
                    BinaryOp::Shl => Some(l << r),
                    BinaryOp::Shr => Some(l >> r),
                    _ => None,
                }
            }
            Expression::Unary { expr, op, .. } => {
                let val = self.eval_static_const_int(expr)?;
                match op {
                    UnaryOp::Negate => Some(-val),
                    UnaryOp::BitNot => Some(!val),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn eval_static_const_bool(&self, expr: &Expression) -> Option<bool> {
        match expr {
            Expression::Lit(Literal::Bool(b), _) => Some(*b),
            Expression::Binary { left, op, right, .. } => {
                if let (Some(l), Some(r)) = (self.eval_static_const_int(left), self.eval_static_const_int(right)) {
                    match op {
                        BinaryOp::Equal => Some(l == r),
                        BinaryOp::NotEqual => Some(l != r),
                        BinaryOp::LessThan => Some(l < r),
                        BinaryOp::LessEqual => Some(l <= r),
                        BinaryOp::GreaterThan => Some(l > r),
                        BinaryOp::GreaterEqual => Some(l >= r),
                        _ => None,
                    }
                } else if let (Some(l_b), Some(r_b)) = (self.eval_static_const_bool(left), self.eval_static_const_bool(right)) {
                    match op {
                        BinaryOp::And => Some(l_b && r_b),
                        BinaryOp::Or => Some(l_b || r_b),
                        BinaryOp::Equal => Some(l_b == r_b),
                        BinaryOp::NotEqual => Some(l_b != r_b),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Expression::Unary { expr, op, .. } => {
                if *op == UnaryOp::Not {
                    self.eval_static_const_bool(expr).map(|b| !b)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
