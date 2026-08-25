use super::types::*;
use super::SemanticAnalyzer;
use crate::ast::*;
use std::collections::HashSet;

impl SemanticAnalyzer {
    pub(crate) fn analyze_control_flow_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::VarDecl {
                name,
                var_type,
                is_mut,
                is_lease: _,
                initializer,
                span,
            } => {
                let inferred_ty = if let Some(init) = initializer {
                    self.analyze_expression(init)
                } else {
                    var_type.clone().unwrap_or(Type::Unknown)
                };

                if let Some(declared) = var_type {
                    if !inferred_ty.is_unknown()
                        && !declared.is_unknown()
                        && !inferred_ty.is_compatible_with(declared)
                    {
                        self.errors.push(
                            DiagnosticError::new(
                                "E002",
                                format!(
                                    "TypeMismatch: variable '{}' declared with type '{}' but initialized with incompatible type '{}'",
                                    name, declared, inferred_ty
                                ),
                                span.line,
                                span.col,
                                "TypeMismatchError",
                            )
                            .with_expected(declared.to_string())
                            .with_actual(inferred_ty.to_string())
                            .with_suggestion(format!("provide an initializer of type '{}'", declared)),
                        );
                    }
                }

                let ty = var_type.clone().unwrap_or(inferred_ty);
                self.declare_var(name, ty.clone(), span.line, *is_mut);

                // Check borrow creation from initializer (e.g. val r = &x or val r = &mut x)
                if let Some(init) = initializer {
                    if let Expression::Unary { expr, op, .. } = init {
                        if *op == UnaryOp::AddressOf {
                            if let Expression::Ident(place_name, _) = expr.as_ref() {
                                self.borrow_checker.check_borrow_creation(
                                    place_name,
                                    name,
                                    LoanKind::Shared,
                                    span,
                                    &mut self.errors,
                                );
                            }
                        }
                    } else if let Expression::Ident(src_name, _) = init {
                        self.borrow_checker.check_use_after_move(
                            src_name,
                            self.get_ownership_state(src_name).as_ref(),
                            span,
                            &mut self.errors,
                        );

                        if !matches!(
                            ty,
                            Type::I8
                                | Type::I16
                                | Type::I32
                                | Type::I64
                                | Type::U8
                                | Type::U16
                                | Type::U32
                                | Type::U64
                                | Type::F32
                                | Type::F64
                                | Type::Bool
                        ) {
                            self.set_ownership_state(
                                src_name,
                                OwnershipState::Moved {
                                    to: name.clone(),
                                    at_line: span.line,
                                },
                            );
                        }
                    }

                    if self.expr_allocates(init) && self.region_depth > 0 {
                        if let Some(reg_set) = self.region_allocations.last_mut() {
                            reg_set.insert(name.clone());
                        }
                    }
                }

                // Strict leak check
                let memory_allocated = initializer
                    .as_ref()
                    .map(|i| self.expr_allocates(i))
                    .unwrap_or(false);
                if self.strict_leaks
                    && (memory_allocated || matches!(ty, Type::Pointer(_)))
                    && self.region_depth == 0
                {
                    self.errors.push(
                        DiagnosticError::new(
                            "E0901",
                            format!(
                                "Memory leak detected: pointer allocated at line {} escapes without safe region boundary",
                                span.line
                            ),
                            span.line,
                            span.col,
                            "MemoryLeakError",
                        )
                        .with_suggestion("wrap in 'region arena { ... }' to guarantee zero memory leak"),
                    );
                }
                true
            }
            Statement::Assignment {
                target,
                value,
                span,
            } => {
                let target_ty = self.analyze_expression(target);
                let val_ty = self.analyze_expression(value);

                if let Expression::Ident(target_name, _) = target {
                    // Enforce mutability
                    if let Some((_, _, is_mut)) = self.lookup_var(target_name) {
                        if !is_mut {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E0908",
                                    format!(
                                        "ImmutableAssignment: cannot assign to immutable variable '{}' at line {}",
                                        target_name, span.line
                                    ),
                                    span.line,
                                    span.col,
                                    "ImmutableAssignmentError",
                                )
                                .with_suggestion(format!("declare '{}' as 'mut' instead of 'val'", target_name)),
                            );
                        }
                    }

                    // Enforce type compatibility on assignment
                    if !target_ty.is_unknown() && !val_ty.is_unknown() && !val_ty.is_compatible_with(&target_ty) {
                        self.errors.push(
                            DiagnosticError::new(
                                "E002",
                                format!(
                                    "TypeMismatch: cannot assign value of type '{}' to variable '{}' of type '{}'",
                                    val_ty, target_name, target_ty
                                ),
                                span.line,
                                span.col,
                                "TypeMismatchError",
                            )
                            .with_expected(target_ty.to_string())
                            .with_actual(val_ty.to_string())
                            .with_suggestion(format!("convert or provide a value of type '{}'", target_ty)),
                        );
                    }

                    // Enforce frozen symbol guarantee
                    if self.frozen_symbols.contains(target_name) {
                        self.errors.push(
                            DiagnosticError::new(
                                "E0908",
                                format!(
                                    "FrozenViolation: cannot mutate frozen symbol '{}' at line {}",
                                    target_name, span.line
                                ),
                                span.line,
                                span.col,
                                "FrozenSymbolError",
                            )
                            .with_suggestion(format!(
                                "remove 'frozen {}' or perform mutations before freezing",
                                target_name
                            )),
                        );
                    }

                    // Check mutation while actively borrowed
                    self.borrow_checker.check_mutation_while_borrowed(target_name, span, &mut self.errors);

                    // Enforce race_free static guarantee: outer variables cannot be mutated
                    if self.in_race_free_block && !self.env.is_in_current_scope(target_name) && self.lookup_var(target_name).is_some() {
                        self.errors.push(
                            DiagnosticError::new(
                                "E0910",
                                format!(
                                    "RaceConditionDetected: mutation of shared variable '{}' at line {} inside race_free block violates data-race freedom",
                                    target_name, span.line
                                ),
                                span.line,
                                span.col,
                                "RaceConditionError",
                            )
                            .with_suggestion(format!(
                                "use atomic operation 'atomic_add(&{}, ...)' or declare '{}' locally inside race_free block",
                                target_name, target_name
                            )),
                        );
                    }

                    // Check move if value is an identifier
                    if let Expression::Ident(src_name, _) = value {
                        self.borrow_checker.check_use_after_move(
                            src_name,
                            self.get_ownership_state(src_name).as_ref(),
                            span,
                            &mut self.errors,
                        );

                        if !matches!(
                            val_ty,
                            Type::I8
                                | Type::I16
                                | Type::I32
                                | Type::I64
                                | Type::U8
                                | Type::U16
                                | Type::U32
                                | Type::U64
                                | Type::F32
                                | Type::F64
                                | Type::Bool
                        ) {
                            self.set_ownership_state(
                                src_name,
                                OwnershipState::Moved {
                                    to: target_name.clone(),
                                    at_line: span.line,
                                },
                            );
                        }
                    }
                }
                true
            }
            Statement::DestructureAssign { pattern, value, span } => {
                let val_ty = self.analyze_expression(value);
                self.bind_pattern(pattern, &val_ty, span.line);
                true
            }
            Statement::LocalFunction(func) => {
                let param_types = func.params.iter().map(|p| p.param_type.clone()).collect();
                self.function_signatures.insert(func.name.clone(), (param_types, func.return_type.clone(), true));
                self.declare_var(&func.name, Type::Custom(func.name.clone()), func.span.line, false);
                self.analyze_function(func);
                true
            }
            Statement::ExtendDecl { target, methods, .. } => {
                for m in methods {
                    let mangled = format!("{}_{}", target, m.name);
                    let param_types = m.params.iter().map(|p| p.param_type.clone()).collect();
                    self.function_signatures.insert(mangled, (param_types, m.return_type.clone(), true));
                    self.analyze_function(m);
                }
                true
            }
            Statement::QuantumUnwrap { name, var_type, expr, fallback, span } => {
                let e_ty = self.analyze_expression(expr);
                self.analyze_expression(fallback);
                let ty = var_type.clone().unwrap_or(e_ty);
                self.declare_var(name, ty, span.line, false);
                true
            }
            Statement::AtomicOp { value, .. } => {
                self.analyze_expression(value);
                true
            }
            Statement::Return { value, span } => {
                let actual_ret = if let Some(val) = value {
                    self.analyze_expression(val)
                } else {
                    Type::Void
                };

                if let Some(ref curr_fn) = self.current_function {
                    if let Some((_, ret_ty, _)) = self.function_signatures.get(curr_fn).cloned() {
                        if !actual_ret.is_unknown() && !ret_ty.is_unknown() && !actual_ret.is_compatible_with(&ret_ty) {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!(
                                        "TypeMismatch: function '{}' expects return type '{}', but returned '{}'",
                                        curr_fn, ret_ty, actual_ret
                                    ),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected(ret_ty.to_string())
                                .with_actual(actual_ret.to_string())
                                .with_suggestion(format!("return an expression of type '{}'", ret_ty)),
                            );
                        }
                    }
                }

                if let Some(val) = value {
                    // Escape analysis for region pointers
                    if let Expression::Ident(ret_var, _) = val {
                        if self.region_depth > 0 {
                            for reg_set in &self.region_allocations {
                                if reg_set.contains(ret_var) {
                                    self.errors.push(
                                        DiagnosticError::new(
                                            "E0903",
                                            format!(
                                                "EscapeViolation: reference to region-scoped memory '{}' escapes region boundary at line {}",
                                                ret_var, span.line
                                            ),
                                            span.line,
                                            span.col,
                                            "EscapeViolationError",
                                        )
                                        .with_suggestion("copy data or allocate on parent region before returning"),
                                    );
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
                self.declare_var(
                    &format!("region_{}", name),
                    Type::Region(name.clone()),
                    span.line,
                    false,
                );
                self.analyze_block(body);
                self.pop_scope();
                self.region_allocations.pop();
                self.region_depth = self.region_depth.saturating_sub(1);
                true
            }
            Statement::Spawn { call, span } => {
                self.analyze_expression(call);
                if let Expression::Call { args, .. } = call {
                    for arg in args {
                        if let Expression::Ident(var_name, _) = arg {
                            self.set_ownership_state(
                                var_name,
                                OwnershipState::Moved {
                                    to: "thread_spawn".to_string(),
                                    at_line: span.line,
                                },
                            );
                        }
                    }
                }
                true
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr);
                true
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let c_ty = self.analyze_expression(condition);
                if !c_ty.is_unknown() && !c_ty.is_bool() {
                    self.errors.push(
                        DiagnosticError::new(
                            "E002",
                            format!("TypeMismatch: if condition must be bool, found '{}'", c_ty),
                            condition.span().line,
                            condition.span().col,
                            "TypeMismatchError",
                        )
                        .with_expected("bool")
                        .with_actual(c_ty.to_string())
                        .with_suggestion("use a boolean condition"),
                    );
                }
                self.analyze_block(then_block);
                if let Some(eb) = else_block {
                    self.analyze_block(eb);
                }
                true
            }
            Statement::Guard {
                condition,
                else_block,
                ..
            } => {
                let c_ty = self.analyze_expression(condition);
                if !c_ty.is_unknown() && !c_ty.is_bool() {
                    self.errors.push(
                        DiagnosticError::new(
                            "E002",
                            format!("TypeMismatch: guard condition must be bool, found '{}'", c_ty),
                            condition.span().line,
                            condition.span().col,
                            "TypeMismatchError",
                        )
                        .with_expected("bool")
                        .with_actual(c_ty.to_string())
                        .with_suggestion("use a boolean condition"),
                    );
                }
                self.analyze_block(else_block);
                true
            }
            Statement::While { condition, body, .. } => {
                let c_ty = self.analyze_expression(condition);
                if !c_ty.is_unknown() && !c_ty.is_bool() {
                    self.errors.push(
                        DiagnosticError::new(
                            "E002",
                            format!("TypeMismatch: while loop condition must be bool, found '{}'", c_ty),
                            condition.span().line,
                            condition.span().col,
                            "TypeMismatchError",
                        )
                        .with_expected("bool")
                        .with_actual(c_ty.to_string())
                        .with_suggestion("use a boolean condition"),
                    );
                }
                self.analyze_block(body);
                true
            }
            Statement::ForIn {
                item_name,
                iterable,
                body,
                span,
            } => {
                let iter_ty = self.analyze_expression(iterable);
                let elem_ty = match iter_ty {
                    Type::Array(inner, _) | Type::Slice(inner) | Type::Pointer(inner) => *inner,
                    Type::Generic(ref name, ref args) if name == "Range" && !args.is_empty() => args[0].clone(),
                    Type::Str => Type::U8,
                    Type::Unknown => Type::Unknown,
                    _ => Type::Unknown,
                };
                self.push_scope();
                self.declare_var(item_name, elem_ty, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
                true
            }
            Statement::ParallelFor {
                item_name,
                iterable,
                body,
                span,
            } => {
                let iter_ty = self.analyze_expression(iterable);
                let elem_ty = match iter_ty {
                    Type::Array(inner, _) | Type::Slice(inner) | Type::Pointer(inner) => *inner,
                    Type::Generic(ref name, ref args) if name == "Range" && !args.is_empty() => args[0].clone(),
                    Type::Str => Type::U8,
                    Type::Unknown => Type::Unknown,
                    _ => Type::Unknown,
                };
                self.push_scope();
                self.declare_var(item_name, elem_ty, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
                true
            }
            Statement::Match { expr, arms, span } => {
                let match_type = self.analyze_expression(expr);
                self.check_match_exhaustiveness(&match_type, arms, span);

                for arm in arms {
                    self.push_scope();
                    if let Pattern::Ident(id) = &arm.pattern {
                        self.declare_var(id, match_type.clone(), arm.span.line, false);
                    }
                    if let Some(g) = &arm.guard {
                        let g_ty = self.analyze_expression(g);
                        if !g_ty.is_unknown() && !g_ty.is_bool() {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("TypeMismatch: match guard must be bool, found '{}'", g_ty),
                                    arm.span.line,
                                    arm.span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("bool")
                                .with_actual(g_ty.to_string())
                                .with_suggestion("ensure match guard evaluates to a boolean"),
                            );
                        }
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

    pub(crate) fn bind_pattern(&mut self, pattern: &Pattern, ty: &Type, line: usize) {
        match pattern {
            Pattern::Ident(name) | Pattern::Binding(name) => {
                self.declare_var(name, ty.clone(), line, false);
            }
            Pattern::Tuple(patterns) => {
                match ty {
                    Type::Tuple(tys) => {
                        for (p, t) in patterns.iter().zip(tys.iter()) {
                            self.bind_pattern(p, t, line);
                        }
                    }
                    _ => {
                        for p in patterns {
                            self.bind_pattern(p, &Type::Unknown, line);
                        }
                    }
                }
            }
            Pattern::Struct { fields, .. } => {
                for (_, p) in fields {
                    self.bind_pattern(p, &Type::Unknown, line);
                }
            }
            _ => {}
        }
    }
}
