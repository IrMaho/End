use super::types::*;
use super::SemanticAnalyzer;
use crate::ast::*;

impl SemanticAnalyzer {
    pub(crate) fn analyze_expression(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::Lit(Literal::Int(_), _) => Type::I64,
            Expression::Lit(Literal::Float(_), _) => Type::F64,
            Expression::Lit(Literal::String(_), _) => Type::Str,
            Expression::Lit(Literal::Bool(_), _) => Type::Bool,
            Expression::Lit(Literal::Null, _) => Type::Pointer(Box::new(Type::Void)),
            Expression::Ident(name, span) => {
                if let Some(domain) = self.domain_ownership.get(name) {
                    if domain != "cpu" && domain != "host" {
                        self.errors.push(DiagnosticError {
                            code: "E0909".to_string(),
                            message: format!("DomainBorrowConflict: resource '{}' has been handed off to domain '{}' and cannot be accessed on CPU at line {} before 'return_to cpu {}'", name, domain, span.line, name),
                            line: span.line,
                            col: span.col,
                            kind: "DomainBorrowConflictError".to_string(),
                            repair_suggestion: Some(format!("call 'return_to cpu {}' before reading or modifying on host CPU", name)),
                        });
                    }
                }
                if let Some(OwnershipState::Moved { to, at_line }) = self.get_ownership_state(name) {
                    self.errors.push(DiagnosticError {
                        code: "E0906".to_string(),
                        message: format!("UseAfterMove: use of moved value '{}' at line {} (moved to '{}' at line {})", name, span.line, to, at_line),
                        line: span.line,
                        col: span.col,
                        kind: "UseAfterMoveError".to_string(),
                        repair_suggestion: Some(format!("clone '{}' or reinitialize before transferring ownership", name)),
                    });
                }
                if let Some((ty, _, _)) = self.lookup_var(name) {
                    ty
                } else {
                    Type::Custom(name.clone())
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let l_ty = self.analyze_expression(left);
                let r_ty = self.analyze_expression(right);
                match op {
                    BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::LessThan | BinaryOp::LessEqual | BinaryOp::GreaterThan | BinaryOp::GreaterEqual | BinaryOp::And | BinaryOp::Or => Type::Bool,
                    _ => {
                        if l_ty == Type::F64 || r_ty == Type::F64 {
                            Type::F64
                        } else if l_ty == Type::F32 || r_ty == Type::F32 {
                            Type::F32
                        } else if l_ty != Type::I64 && l_ty != Type::Void && l_ty != Type::Bool {
                            l_ty
                        } else {
                            r_ty
                        }
                    }
                }
            }
            Expression::Unary { expr, op, .. } => {
                let inner = self.analyze_expression(expr);
                match op {
                    UnaryOp::AddressOf => Type::Pointer(Box::new(inner)),
                    UnaryOp::Deref => match inner {
                        Type::Pointer(t) => *t,
                        _ => inner,
                    },
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Negate | UnaryOp::BitNot => inner,
                }
            }
            Expression::Call { callee, args, .. } => {
                for arg in args {
                    self.analyze_expression(arg);
                }
                let mut ret_type = Type::Void;
                if let Expression::Ident(callee_name, _) = callee.as_ref() {
                    if let Some((_params, sig_ret, _)) = self.function_signatures.get(callee_name) {
                        ret_type = sig_ret.clone();
                    }
                    if let Some(curr_fn) = &self.current_function {
                        self.graph.add_call(curr_fn, callee_name);
                        if let Some(effects) = self.function_effects.get_mut(curr_fn) {
                            if callee_name.contains("socket") || callee_name.contains("send") || callee_name.contains("recv") || callee_name.contains("http") || callee_name.contains("net") {
                                effects.insert("network".to_string());
                            }
                            if callee_name.contains("print") || callee_name.contains("write") || callee_name.contains("read") {
                                effects.insert("io".to_string());
                            }
                        }
                    }
                }
                ret_type
            }
            Expression::FieldAccess { object, field, .. } => {
                let obj_ty = self.analyze_expression(object);
                match obj_ty {
                    Type::Custom(ref sname) => {
                        if let Some(st_def) = self.structs.get(sname) {
                            if let Some(f_def) = st_def.fields.iter().find(|f| &f.name == field) {
                                return f_def.field_type.clone();
                            }
                        }
                        Type::I64
                    }
                    _ => Type::I64,
                }
            }
            Expression::Index { array, index, .. } => {
                let arr_ty = self.analyze_expression(array);
                self.analyze_expression(index);
                match arr_ty {
                    Type::Pointer(inner) | Type::Slice(inner) | Type::Array(inner, _) => *inner,
                    Type::Str => Type::U8,
                    _ => Type::I64,
                }
            }
            Expression::Alloc { target_type, allocator, .. } => {
                self.analyze_expression(allocator);
                Type::Pointer(Box::new(target_type.clone()))
            }
            Expression::Cast { target_type, expr, .. } => {
                self.analyze_expression(expr);
                target_type.clone()
            }
            Expression::Await { expr, .. } => {
                self.analyze_expression(expr)
            }
            Expression::StructInit { name, fields, .. } => {
                for (_, f_expr) in fields {
                    self.analyze_expression(f_expr);
                }
                Type::Custom(name.clone())
            }
            Expression::Match { expr, arms, .. } => {
                self.analyze_expression(expr);
                for arm in arms {
                    for stmt in &arm.body.statements {
                        self.analyze_statement(stmt);
                    }
                }
                Type::I64
            }
            _ => Type::Void,
        }
    }

    pub(crate) fn expr_allocates(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Alloc { .. } => true,
            Expression::Call { callee, .. } => {
                if let Expression::Ident(name, _) = callee.as_ref() {
                    name == "alloc" || name.contains("create")
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
