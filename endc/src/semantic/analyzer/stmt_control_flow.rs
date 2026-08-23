use super::types::*;
use super::SemanticAnalyzer;
use crate::ast::*;
use std::collections::HashSet;

impl SemanticAnalyzer {
    pub(crate) fn analyze_control_flow_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::VarDecl { name, var_type, is_mut, is_lease: _, initializer, span } => {
                let inferred_ty = if let Some(init) = initializer {
                    self.analyze_expression(init)
                } else {
                    var_type.clone().unwrap_or(Type::Void)
                };

                let ty = var_type.clone().unwrap_or(inferred_ty);
                self.declare_var(name, ty.clone(), span.line, *is_mut);

                // Check borrow creation from initializer (e.g. val r = &x)
                if let Some(init) = initializer {
                    if let Expression::Unary { expr, op, .. } = init {
                        if *op == UnaryOp::AddressOf {
                            if let Expression::Ident(place_name, _) = expr.as_ref() {
                                // Check if already mutably borrowed
                                if let Some(existing_loan) = self.active_loans.iter().find(|l| l.place == *place_name && l.kind == LoanKind::Mutable) {
                                    self.errors.push(DiagnosticError {
                                        code: "E0907".to_string(),
                                        message: format!("BorrowConflict: cannot borrow '{}' at line {} because it is already mutably borrowed by '{}' at line {}", place_name, span.line, existing_loan.holder, existing_loan.borrowed_at),
                                        line: span.line,
                                        col: span.col,
                                        kind: "BorrowConflictError".to_string(),
                                        repair_suggestion: Some("release previous mutable reference before borrowing again".to_string()),
                                    });
                                } else {
                                    self.active_loans.push(ActiveLoan {
                                        place: place_name.clone(),
                                        kind: LoanKind::Shared,
                                        borrowed_at: span.line,
                                        holder: name.clone(),
                                    });
                                }
                            }
                        }
                    } else if let Expression::Ident(src_name, _) = init {
                        // Check Move State
                        if let Some(OwnershipState::Moved { to, at_line }) = self.get_ownership_state(src_name) {
                            self.errors.push(DiagnosticError {
                                code: "E0906".to_string(),
                                message: format!("UseAfterMove: use of moved value '{}' at line {} (previously moved to '{}' at line {})", src_name, span.line, to, at_line),
                                line: span.line,
                                col: span.col,
                                kind: "UseAfterMoveError".to_string(),
                                repair_suggestion: Some(format!("clone '{}' or reinitialize before transferring ownership", src_name)),
                            });
                        } else {
                            if !matches!(ty, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::F32 | Type::F64 | Type::Bool) {
                                self.set_ownership_state(src_name, OwnershipState::Moved { to: name.clone(), at_line: span.line });
                            }
                        }
                    }

                    if self.expr_allocates(init) && self.region_depth > 0 {
                        if let Some(reg_set) = self.region_allocations.last_mut() {
                            reg_set.insert(name.clone());
                        }
                    }
                }

                // Strict leak check
                let memory_allocated = initializer.as_ref().map(|i| self.expr_allocates(i)).unwrap_or(false);
                if self.strict_leaks && (memory_allocated || matches!(ty, Type::Pointer(_))) && self.region_depth == 0 {
                    self.errors.push(DiagnosticError {
                        code: "E0901".to_string(),
                        message: format!("Memory leak detected: pointer allocated at line {} escapes without safe region boundary", span.line),
                        line: span.line,
                        col: span.col,
                        kind: "MemoryLeakError".to_string(),
                        repair_suggestion: Some("wrap in 'region arena { ... }' to guarantee zero memory leak".to_string()),
                    });
                }
                true
            }
            Statement::Assignment { target, value, span } => {
                self.analyze_expression(target);
                self.analyze_expression(value);

                if let Expression::Ident(target_name, _) = target {
                    // Enforce frozen symbol guarantee
                    if self.frozen_symbols.contains(target_name) {
                        self.errors.push(DiagnosticError {
                            code: "E0908".to_string(),
                            message: format!("FrozenViolation: cannot mutate frozen symbol '{}' at line {}", target_name, span.line),
                            line: span.line,
                            col: span.col,
                            kind: "FrozenSymbolError".to_string(),
                            repair_suggestion: Some(format!("remove 'frozen {}' or perform mutations before freezing", target_name)),
                        });
                    }

                    // Check mutation while actively borrowed
                    if let Some(loan) = self.active_loans.iter().find(|l| l.place == *target_name && l.holder != *target_name) {
                        self.errors.push(DiagnosticError {
                            code: "E0907".to_string(),
                            message: format!("BorrowConflict: cannot mutate '{}' at line {} because it is currently borrowed by '{}' (borrowed at line {})", target_name, span.line, loan.holder, loan.borrowed_at),
                            line: span.line,
                            col: span.col,
                            kind: "BorrowConflictError".to_string(),
                            repair_suggestion: Some(format!("ensure borrow '{}' goes out of scope before modifying '{}'", loan.holder, target_name)),
                        });
                    }

                    // Enforce race_free static guarantee: no shared mutable outer variable mutation
                    if self.in_race_free_block {
                        if let Some(current_scope) = self.var_scopes.last() {
                            if !current_scope.contains_key(target_name) && self.lookup_var(target_name).is_some() {
                                self.errors.push(DiagnosticError {
                                    code: "E0910".to_string(),
                                    message: format!("RaceConditionDetected: mutation of shared outer variable '{}' at line {} inside race_free block violates data-race freedom", target_name, span.line),
                                    line: span.line,
                                    col: span.col,
                                    kind: "RaceConditionError".to_string(),
                                    repair_suggestion: Some(format!("use atomic operation 'atomic_add(&{}, ...)' or declare '{}' locally inside race_free block", target_name, target_name)),
                                });
                            }
                        }
                    }
                }
                true
            }
            Statement::Return { value, span } => {
                if let Some(val) = value {
                    self.analyze_expression(val);

                    // Formal Escape Analysis: check if returning an inner-region allocated pointer
                    if let Expression::Ident(ret_var, _) = val {
                        if self.region_depth > 0 {
                            for reg_set in &self.region_allocations {
                                if reg_set.contains(ret_var) {
                                    self.errors.push(DiagnosticError {
                                        code: "E0903".to_string(),
                                        message: format!("EscapeViolation: reference to region-scoped memory '{}' escapes region boundary at line {}", ret_var, span.line),
                                        line: span.line,
                                        col: span.col,
                                        kind: "EscapeViolationError".to_string(),
                                        repair_suggestion: Some("copy data or allocate on parent region before returning".to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
                true
            }
            Statement::RegionBlock { name, body, span } => {
                self.region_depth += 1;
                self.region_allocations.push(HashSet::new());
                self.push_scope();
                self.declare_var(&format!("region_{}", name), Type::Region(name.clone()), span.line, false);
                self.analyze_block(body);
                self.pop_scope();
                self.region_allocations.pop();
                self.region_depth = self.region_depth.saturating_sub(1);
                true
            }
            Statement::Spawn { call, span } => {
                self.analyze_expression(call);
                // In Thread Spawn: capture variables transfer ownership to spawn
                if let Expression::Call { args, .. } = call {
                    for arg in args {
                        if let Expression::Ident(var_name, _) = arg {
                            self.set_ownership_state(var_name, OwnershipState::Moved {
                                to: "thread_spawn".to_string(),
                                at_line: span.line,
                            });
                        }
                    }
                }
                true
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr);
                true
            }
            Statement::If { condition, then_block, else_block, .. } => {
                self.analyze_expression(condition);
                self.analyze_block(then_block);
                if let Some(eb) = else_block {
                    self.analyze_block(eb);
                }
                true
            }
            Statement::Guard { condition, else_block, .. } => {
                self.analyze_expression(condition);
                self.analyze_block(else_block);
                true
            }
            Statement::While { condition, body, .. } => {
                self.analyze_expression(condition);
                self.analyze_block(body);
                true
            }
            Statement::ForIn { item_name, iterable, body, span } => {
                self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(item_name, Type::I32, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
                true
            }
            Statement::ParallelFor { item_name, iterable, body, span } => {
                self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(item_name, Type::I32, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
                true
            }
            Statement::Match { expr, arms, .. } => {
                let match_type = self.analyze_expression(expr);
                for arm in arms {
                    self.push_scope();
                    if let Pattern::Ident(id) = &arm.pattern {
                        self.declare_var(id, match_type.clone(), arm.span.line, false);
                    }
                    if let Some(g) = &arm.guard {
                        self.analyze_expression(g);
                    }
                    self.analyze_block(&arm.body);
                    self.pop_scope();
                }
                true
            }
            Statement::Defer { expr, .. } => {
                self.analyze_expression(expr);
                true
            }
            _ => false,
        }
    }
}
