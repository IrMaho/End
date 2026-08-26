use super::types::*;
use super::SemanticAnalyzer;
use crate::ast::*;
use std::collections::HashSet;

impl SemanticAnalyzer {
    pub(crate) fn analyze_expression(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::Lit(lit, _) => match lit {
                Literal::Int(_) => Type::I64,
                Literal::Float(_) => Type::F64,
                Literal::String(_) => Type::Str,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Pointer(Box::new(Type::Void)),
            },
            Expression::Ident(name, span) => {
                if let Some(domain) = self.domain_ownership.get(name) {
                    if domain != "cpu" && domain != "host" {
                        self.errors.push(
                            DiagnosticError::new(
                                "E0909",
                                format!(
                                    "DomainBorrowConflict: resource '{}' has been handed off to domain '{}' and cannot be accessed on CPU at line {} before 'return_to cpu {}'",
                                    name, domain, span.line, name
                                ),
                                span.line,
                                span.col,
                                "DomainBorrowConflictError",
                            )
                            .with_suggestion(format!("call 'return_to cpu {}' before reading or modifying on host CPU", name)),
                        );
                    }
                }

                // Check borrow/ownership state (use-after-move or use-after-free)
                self.borrow_checker.check_use_after_move(
                    name,
                    self.get_ownership_state(name).as_ref(),
                    span,
                    &mut self.errors,
                );

                if let Some((ty, _, _)) = self.lookup_var(name) {
                    ty
                } else if self.structs.contains_key(name)
                    || self.enums.contains_key(name)
                    || self.features.contains_key(name)
                    || self.contracts.contains_key(name)
                    || self.module_responsibilities.contains_key(name)
                    || self.function_signatures.contains_key(name)
                    || is_known_builtin(name)
                    || is_known_method(name)
                    || name.starts_with(|c: char| c.is_uppercase())
                {
                    get_builtin_return_type(name)
                } else {
                    self.errors.push(
                        DiagnosticError::new(
                            "E003",
                            format!("UndefinedName: identifier '{}' is not defined in the current scope", name),
                            span.line,
                            span.col,
                            "UndefinedNameError",
                        )
                        .with_expected("declared variable, function, struct, or enum")
                        .with_actual(format!("undefined identifier '{}'", name))
                        .with_suggestion(format!("declare '{}' before using it", name)),
                    );
                    Type::Unknown
                }
            }
            Expression::Binary {
                left,
                op,
                right,
                span,
            } => {
                let l_ty = self.analyze_expression(left);
                let r_ty = self.analyze_expression(right);

                match op {
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::LessThan
                    | BinaryOp::LessEqual
                    | BinaryOp::GreaterThan
                    | BinaryOp::GreaterEqual => {
                        if !l_ty.is_unknown()
                            && !r_ty.is_unknown()
                            && !l_ty.is_compatible_with(&r_ty)
                        {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!(
                                        "TypeMismatch: binary comparison '{:?}' between incompatible types '{}' and '{}'",
                                        op, l_ty, r_ty
                                    ),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected(l_ty.to_string())
                                .with_actual(r_ty.to_string())
                                .with_suggestion("compare expressions of the same type"),
                            );
                        }
                        Type::Bool
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if !l_ty.is_unknown() && !l_ty.is_bool() {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("TypeMismatch: left operand of logical operator must be bool, found '{}'", l_ty),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("bool")
                                .with_actual(l_ty.to_string())
                                .with_suggestion("use a boolean expression"),
                            );
                        }
                        if !r_ty.is_unknown() && !r_ty.is_bool() {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("TypeMismatch: right operand of logical operator must be bool, found '{}'", r_ty),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("bool")
                                .with_actual(r_ty.to_string())
                                .with_suggestion("use a boolean expression"),
                            );
                        }
                        Type::Bool
                    }
                    _ => {
                        if (l_ty == Type::Str || r_ty == Type::Str) && *op == BinaryOp::Add {
                            if l_ty == Type::Str && (r_ty == Type::Str || r_ty.is_unknown()) {
                                Type::Str
                            } else if r_ty == Type::Str && (l_ty == Type::Str || l_ty.is_unknown()) {
                                Type::Str
                            } else {
                                self.errors.push(
                                    DiagnosticError::new(
                                        "E002",
                                        format!(
                                            "TypeMismatch: cannot concatenate string with type '{}'",
                                            if l_ty == Type::Str { &r_ty } else { &l_ty }
                                        ),
                                        span.line,
                                        span.col,
                                        "TypeMismatchError",
                                    )
                                    .with_expected("str")
                                    .with_actual(if l_ty == Type::Str { r_ty.to_string() } else { l_ty.to_string() })
                                    .with_suggestion("convert operand to string first"),
                                );
                                Type::Str
                            }
                        } else if !l_ty.is_unknown()
                            && !r_ty.is_unknown()
                            && (!l_ty.is_numeric() || !r_ty.is_numeric())
                        {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!(
                                        "TypeMismatch: arithmetic operation '{:?}' requires numeric operands, found '{}' and '{}'",
                                        op, l_ty, r_ty
                                    ),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("numeric types")
                                .with_actual(format!("'{}' and '{}'", l_ty, r_ty))
                                .with_suggestion("use numeric types for arithmetic operations"),
                            );
                            Type::Unknown
                        } else if l_ty == Type::F64 || r_ty == Type::F64 {
                            Type::F64
                        } else if l_ty == Type::F32 || r_ty == Type::F32 {
                            Type::F32
                        } else if l_ty.is_integer() {
                            l_ty
                        } else if r_ty.is_integer() {
                            r_ty
                        } else if l_ty.is_unknown() {
                            r_ty
                        } else {
                            l_ty
                        }
                    }
                }
            }
            Expression::Unary { expr, op, span } => {
                let inner = self.analyze_expression(expr);
                match op {
                    UnaryOp::AddressOf => Type::Pointer(Box::new(inner)),
                    UnaryOp::Deref => match inner {
                        Type::Pointer(t) => *t,
                        Type::Unknown => Type::Unknown,
                        _ => {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("TypeMismatch: cannot dereference non-pointer type '{}'", inner),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("pointer type (*T)")
                                .with_actual(inner.to_string())
                                .with_suggestion("ensure expression is a pointer before dereferencing"),
                            );
                            Type::Unknown
                        }
                    },
                    UnaryOp::Not => {
                        if !inner.is_unknown() && !inner.is_bool() {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("TypeMismatch: logical NOT operator '!' requires bool operand, found '{}'", inner),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("bool")
                                .with_actual(inner.to_string())
                                .with_suggestion("apply '!' only to boolean expressions"),
                            );
                        }
                        Type::Bool
                    }
                    UnaryOp::Negate => {
                        if !inner.is_unknown() && !inner.is_numeric() {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("TypeMismatch: negate '-' requires numeric operand, found '{}'", inner),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("numeric type (i64, f64, etc.)")
                                .with_actual(inner.to_string())
                                .with_suggestion("apply '-' only to numbers"),
                            );
                        }
                        inner
                    }
                    UnaryOp::BitNot => {
                        if !inner.is_unknown() && !inner.is_integer() {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("TypeMismatch: bitwise NOT '~' requires integer operand, found '{}'", inner),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("integer type (i64, u32, etc.)")
                                .with_actual(inner.to_string())
                                .with_suggestion("apply '~' only to integers"),
                            );
                        }
                        inner
                    }
                }
            }
            Expression::Call { callee, args, span } => {
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.analyze_expression(arg));
                }

                if let Expression::Ident(callee_name, callee_span) = callee.as_ref() {
                    // Check explicit free(p) for ownership tracking
                    if callee_name == "free" {
                        if args.len() != 1 {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!("ArityMismatch: function 'free' expects 1 argument, but {} were provided", args.len()),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected("1 argument")
                                .with_actual(format!("{} arguments", args.len()))
                                .with_suggestion("provide exactly 1 pointer argument to free()"),
                            );
                        } else if let Expression::Ident(arg_name, arg_span) = &args[0] {
                            let state = self.get_ownership_state(arg_name);
                            if self.borrow_checker.check_free(arg_name, state.as_ref(), arg_span, &mut self.errors) {
                                self.set_ownership_state(arg_name, OwnershipState::Freed { at_line: span.line });
                            }
                        }
                        return Type::Void;
                    }

                    if let Some((param_types, ret_type, _)) = self.function_signatures.get(callee_name).cloned() {
                        // Check arity
                        if param_types.len() != args.len() {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E002",
                                    format!(
                                        "ArityMismatch: function '{}' expects {} arguments, but {} were provided",
                                        callee_name,
                                        param_types.len(),
                                        args.len()
                                    ),
                                    span.line,
                                    span.col,
                                    "TypeMismatchError",
                                )
                                .with_expected(format!("{} arguments", param_types.len()))
                                .with_actual(format!("{} arguments", args.len()))
                                .with_suggestion(format!("provide exactly {} arguments", param_types.len())),
                            );
                        }

                        // Check argument types
                        for (idx, (arg_ty, param_ty)) in arg_types.iter().zip(param_types.iter()).enumerate() {
                            if !arg_ty.is_unknown() && !param_ty.is_unknown() && !arg_ty.is_compatible_with(param_ty) {
                                self.errors.push(
                                    DiagnosticError::new(
                                        "E002",
                                        format!(
                                            "TypeMismatch: argument {} of function '{}' expects '{}', but '{}' was provided",
                                            idx + 1,
                                            callee_name,
                                            param_ty,
                                            arg_ty
                                        ),
                                        span.line,
                                        span.col,
                                        "TypeMismatchError",
                                    )
                                    .with_expected(param_ty.to_string())
                                    .with_actual(arg_ty.to_string())
                                    .with_suggestion(format!("pass a value of type '{}'", param_ty)),
                                );
                            }
                        }

                        if let Some(curr_fn) = &self.current_function {
                            self.graph.add_call(curr_fn, callee_name);
                            if let Some(effects) = self.function_effects.get_mut(curr_fn) {
                                if callee_name.contains("socket")
                                    || callee_name.contains("send")
                                    || callee_name.contains("recv")
                                    || callee_name.contains("http")
                                    || callee_name.contains("net")
                                {
                                    effects.insert("network".to_string());
                                }
                                if callee_name.contains("print")
                                    || callee_name.contains("write")
                                    || callee_name.contains("read")
                                {
                                    effects.insert("io".to_string());
                                }
                            }
                        }

                        return ret_type;
                    } else if self.structs.contains_key(callee_name) {
                        return Type::Custom(callee_name.clone());
                    } else if let Some(e_def) = self.enums.values().find(|e| e.variants.iter().any(|v| &v.name == callee_name)) {
                        return Type::Custom(e_def.name.clone());
                    } else if is_known_builtin(callee_name) || is_known_method(callee_name) {
                        return get_builtin_return_type(callee_name);
                    } else if self.lookup_var(callee_name).is_some() {
                        return Type::Unknown;
                    } else if callee_name.starts_with(|c: char| c.is_uppercase()) {
                        return Type::Custom(callee_name.clone());
                    } else {
                        self.errors.push(
                            DiagnosticError::new(
                                "E003",
                                format!("UndefinedFunction: function '{}' is not defined", callee_name),
                                callee_span.line,
                                callee_span.col,
                                "UndefinedNameError",
                            )
                            .with_expected("defined function name")
                            .with_actual(format!("undefined function '{}'", callee_name))
                            .with_suggestion(format!("define function '{}' before calling it", callee_name)),
                        );
                        return Type::Unknown;
                    }
                } else if let Expression::FieldAccess { object, field, span: _m_span } = callee.as_ref() {
                    let obj_ty = self.analyze_expression(object);
                    if let Type::Custom(sname) = obj_ty {
                        let method_key = format!("{}_{}", sname, field);
                        if let Some((_param_types, ret_type, _)) = self.function_signatures.get(&method_key).cloned() {
                            return ret_type;
                        }
                    }
                    if is_known_method(field) {
                        return get_method_return_type(field);
                    }
                    return Type::Unknown;
                }

                self.analyze_expression(callee);
                Type::Unknown
            }
            Expression::FieldAccess { object, field, span } => {
                let obj_ty = self.analyze_expression(object);
                if is_known_method(field) {
                    return get_method_return_type(field);
                }

                match obj_ty {
                    Type::Custom(ref sname) => {
                        if let Some(st_def) = self.structs.get(sname) {
                            if let Some(f_def) = st_def.fields.iter().find(|f| &f.name == field) {
                                return f_def.field_type.clone();
                            } else {
                                let method_key = format!("{}_{}", sname, field);
                                if self.function_signatures.contains_key(&method_key) {
                                    return Type::Unknown;
                                }
                                let field_names: Vec<&String> = st_def.fields.iter().map(|f| &f.name).collect();
                                self.errors.push(
                                    DiagnosticError::new(
                                        "E021",
                                        format!("FieldNotFound: struct '{}' has no field named '{}'", sname, field),
                                        span.line,
                                        span.col,
                                        "FieldNotFoundError",
                                    )
                                    .with_expected(format!("one of the declared fields of struct '{}'", sname))
                                    .with_actual(format!("field '{}'", field))
                                    .with_suggestion(format!("available fields: {:?}", field_names)),
                                );
                                return Type::Unknown;
                            }
                        } else {
                            // If struct metadata is known externally or builtin
                            return Type::Unknown;
                        }
                    }
                    Type::Pointer(inner) => match inner.as_ref() {
                        Type::Custom(sname) => {
                            if let Some(st_def) = self.structs.get(sname) {
                                if let Some(f_def) = st_def.fields.iter().find(|f| &f.name == field) {
                                    return f_def.field_type.clone();
                                } else {
                                    let field_names: Vec<&String> = st_def.fields.iter().map(|f| &f.name).collect();
                                    self.errors.push(
                                        DiagnosticError::new(
                                            "E021",
                                            format!("FieldNotFound: struct '{}' has no field named '{}'", sname, field),
                                            span.line,
                                            span.col,
                                            "FieldNotFoundError",
                                        )
                                        .with_expected(format!("one of the declared fields of struct '{}'", sname))
                                        .with_actual(format!("field '{}'", field))
                                        .with_suggestion(format!("available fields: {:?}", field_names)),
                                    );
                                    return Type::Unknown;
                                }
                            }
                            Type::Unknown
                        }
                        _ => Type::Unknown,
                    },
                    Type::Str if field == "len" => Type::I64,
                    Type::Slice(_) if field == "len" => Type::I64,
                    Type::Array(_, _) if field == "len" => Type::I64,
                    Type::Unknown => Type::Unknown,
                    _ => {
                        self.errors.push(
                            DiagnosticError::new(
                                "E021",
                                format!("FieldNotFound: cannot access field '{}' on non-struct type '{}'", field, obj_ty),
                                span.line,
                                span.col,
                                "FieldNotFoundError",
                            )
                            .with_expected("struct type")
                            .with_actual(obj_ty.to_string())
                            .with_suggestion("ensure target is a struct before accessing fields"),
                        );
                        Type::Unknown
                    }
                }
            }
            Expression::Index { array, index, span } => {
                let arr_ty = self.analyze_expression(array);
                let idx_ty = self.analyze_expression(index);

                if !idx_ty.is_unknown() && !idx_ty.is_integer() {
                    self.errors.push(
                        DiagnosticError::new(
                            "E002",
                            format!("TypeMismatch: array index must be an integer, found '{}'", idx_ty),
                            span.line,
                            span.col,
                            "TypeMismatchError",
                        )
                        .with_expected("integer (i64, u32, etc.)")
                        .with_actual(idx_ty.to_string())
                        .with_suggestion("use an integer index"),
                    );
                }

                match arr_ty {
                    Type::Pointer(inner) | Type::Slice(inner) | Type::Array(inner, _) => *inner,
                    Type::Str => Type::U8,
                    Type::Unknown => Type::Unknown,
                    _ => {
                        self.errors.push(
                            DiagnosticError::new(
                                "E002",
                                format!("TypeMismatch: cannot index into non-indexable type '{}'", arr_ty),
                                span.line,
                                span.col,
                                "TypeMismatchError",
                            )
                            .with_expected("indexable type (array, slice, pointer, or string)")
                            .with_actual(arr_ty.to_string())
                            .with_suggestion("index only into arrays, slices, or strings"),
                        );
                        Type::Unknown
                    }
                }
            }
            Expression::StructInit { name, fields, span } => {
                if let Some(st_def) = self.structs.get(name).cloned() {
                    for (fname, fexpr) in fields {
                        let fty = self.analyze_expression(fexpr);
                        if let Some(f_def) = st_def.fields.iter().find(|f| &f.name == fname) {
                            if !fty.is_unknown() && !f_def.field_type.is_unknown() && !fty.is_compatible_with(&f_def.field_type) {
                                self.errors.push(
                                    DiagnosticError::new(
                                        "E002",
                                        format!(
                                            "TypeMismatch: struct field '{}.{}' expects '{}', but '{}' was provided",
                                            name, fname, f_def.field_type, fty
                                        ),
                                        span.line,
                                        span.col,
                                        "TypeMismatchError",
                                    )
                                    .with_expected(f_def.field_type.to_string())
                                    .with_actual(fty.to_string())
                                    .with_suggestion(format!("provide a value of type '{}'", f_def.field_type)),
                                );
                            }
                        } else {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E021",
                                    format!("FieldNotFound: struct '{}' has no field named '{}'", name, fname),
                                    span.line,
                                    span.col,
                                    "FieldNotFoundError",
                                )
                                .with_expected(format!("declared field of struct '{}'", name))
                                .with_actual(format!("non-existent field '{}'", fname))
                                .with_suggestion("check field names in struct definition"),
                            );
                        }
                    }
                    Type::Custom(name.clone())
                } else {
                    for (_, fexpr) in fields {
                        self.analyze_expression(fexpr);
                    }
                    Type::Custom(name.clone())
                }
            }
            Expression::EnumInit {
                enum_name,
                variant_name,
                payload,
                span,
            } => {
                if let Some(ename) = enum_name {
                    if let Some(e_def) = self.enums.get(ename).cloned() {
                        if let Some(v_def) = e_def.variants.iter().find(|v| &v.name == variant_name) {
                            if let Some(expected_payload) = &v_def.payload {
                                if let Some(p) = payload {
                                    let p_ty = self.analyze_expression(p);
                                    if !p_ty.is_unknown() && !expected_payload.is_unknown() && !p_ty.is_compatible_with(expected_payload) {
                                        self.errors.push(
                                            DiagnosticError::new(
                                                "E002",
                                                format!(
                                                    "TypeMismatch: enum variant '{}::{}' expects payload of type '{}', but '{}' was provided",
                                                    ename, variant_name, expected_payload, p_ty
                                                ),
                                                span.line,
                                                span.col,
                                                "TypeMismatchError",
                                            )
                                            .with_expected(expected_payload.to_string())
                                            .with_actual(p_ty.to_string())
                                            .with_suggestion(format!("provide a payload of type '{}'", expected_payload)),
                                        );
                                    }
                                } else {
                                    self.errors.push(
                                        DiagnosticError::new(
                                            "E002",
                                            format!(
                                                "TypeMismatch: enum variant '{}::{}' requires a payload of type '{}', but none was provided",
                                                ename, variant_name, expected_payload
                                            ),
                                            span.line,
                                            span.col,
                                            "TypeMismatchError",
                                        )
                                        .with_expected(expected_payload.to_string())
                                        .with_actual("no payload")
                                        .with_suggestion(format!("provide a payload of type '{}'", expected_payload)),
                                    );
                                }
                            } else if payload.is_some() {
                                self.errors.push(
                                    DiagnosticError::new(
                                        "E002",
                                        format!("TypeMismatch: enum variant '{}::{}' does not accept a payload", ename, variant_name),
                                        span.line,
                                        span.col,
                                        "TypeMismatchError",
                                    )
                                    .with_expected("no payload")
                                    .with_actual("payload provided")
                                    .with_suggestion("remove the payload argument"),
                                );
                            }
                        } else {
                            let available: Vec<&String> = e_def.variants.iter().map(|v| &v.name).collect();
                            self.errors.push(
                                DiagnosticError::new(
                                    "E009",
                                    format!("EnumVariantInvalid: enum '{}' has no variant named '{}'", ename, variant_name),
                                    span.line,
                                    span.col,
                                    "EnumVariantError",
                                )
                                .with_expected(format!("one of variants {:?}", available))
                                .with_actual(format!("variant '{}'", variant_name))
                                .with_suggestion(format!("available variants: {:?}", available)),
                            );
                        }
                    }
                    if let Some(p) = payload {
                        self.analyze_expression(p);
                    }
                    Type::Custom(ename.clone())
                } else {
                    let mut matched_enum = None;
                    for (ename, e_def) in &self.enums {
                        if let Some(v_def) = e_def.variants.iter().find(|v| &v.name == variant_name) {
                            matched_enum = Some((ename.clone(), v_def.clone()));
                            break;
                        }
                    }

                    if let Some((ename, v_def)) = matched_enum {
                        if let Some(expected_payload) = &v_def.payload {
                            if let Some(p) = payload {
                                let p_ty = self.analyze_expression(p);
                                if !p_ty.is_unknown() && !expected_payload.is_unknown() && !p_ty.is_compatible_with(expected_payload) {
                                    self.errors.push(
                                        DiagnosticError::new(
                                            "E002",
                                            format!(
                                                "TypeMismatch: enum variant '{}::{}' expects payload of type '{}', but '{}' was provided",
                                                ename, variant_name, expected_payload, p_ty
                                            ),
                                            span.line,
                                            span.col,
                                            "TypeMismatchError",
                                        )
                                        .with_expected(expected_payload.to_string())
                                        .with_actual(p_ty.to_string())
                                        .with_suggestion(format!("provide a payload of type '{}'", expected_payload)),
                                    );
                                }
                            }
                        } else if let Some(p) = payload {
                            self.analyze_expression(p);
                        }
                        Type::Custom(ename)
                    } else {
                        let all_variants: Vec<String> = self
                            .enums
                            .values()
                            .flat_map(|e| e.variants.iter().map(|v| v.name.clone()))
                            .collect();
                        self.errors.push(
                            DiagnosticError::new(
                                "E009",
                                format!("EnumVariantInvalid: no enum has a variant named '{}'", variant_name),
                                span.line,
                                span.col,
                                "EnumVariantError",
                            )
                            .with_expected("known enum variant")
                            .with_actual(format!("unrecognized variant '{}'", variant_name))
                            .with_suggestion(format!("available variants: {:?}", all_variants)),
                        );
                        if let Some(p) = payload {
                            self.analyze_expression(p);
                        }
                        Type::Unknown
                    }
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
            Expression::Await { expr, .. } => self.analyze_expression(expr),
            Expression::Match { expr, arms, span } => {
                let match_ty = self.analyze_expression(expr);
                self.check_match_exhaustiveness(&match_ty, arms, span);

                let result_ty = Type::Unknown;
                for arm in arms {
                    self.push_scope();
                    if let Pattern::Ident(id) = &arm.pattern {
                        self.declare_var(id, match_ty.clone(), arm.span.line, false);
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
                    for stmt in &arm.body.statements {
                        self.analyze_statement(stmt);
                    }
                    self.pop_scope();
                }

                result_ty
            }
            Expression::InterpolatedString { parts, .. } => {
                for part in parts {
                    if let crate::ast::expr::collections::StringPart::Expr(e) = part {
                        self.analyze_expression(e);
                    }
                }
                Type::Str
            }
            Expression::Tuple(elements, _) => {
                let tys = elements.iter().map(|e| self.analyze_expression(e)).collect();
                Type::Tuple(tys)
            }
            Expression::Range { start, end, .. } => {
                let s_ty = self.analyze_expression(start);
                self.analyze_expression(end);
                Type::Generic("Range".to_string(), vec![s_ty])
            }
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                let c_ty = self.analyze_expression(condition);
                if !c_ty.is_unknown() && !c_ty.is_bool() {
                    self.errors.push(
                        DiagnosticError::new(
                            "E002",
                            format!("TypeMismatch: ternary condition must be bool, found '{}'", c_ty),
                            span.line,
                            span.col,
                            "TypeMismatchError",
                        )
                        .with_expected("bool")
                        .with_actual(c_ty.to_string())
                        .with_suggestion("use a boolean condition"),
                    );
                }
                let t_ty = self.analyze_expression(then_branch);
                let e_ty = self.analyze_expression(else_branch);
                if !t_ty.is_unknown() && !e_ty.is_unknown() && !t_ty.is_compatible_with(&e_ty) {
                    self.errors.push(
                        DiagnosticError::new(
                            "E002",
                            format!("TypeMismatch: branches of ternary have incompatible types '{}' and '{}'", t_ty, e_ty),
                            span.line,
                            span.col,
                            "TypeMismatchError",
                        )
                        .with_expected(t_ty.to_string())
                        .with_actual(e_ty.to_string())
                        .with_suggestion("ensure both branches return compatible types"),
                    );
                }
                if t_ty.is_unknown() {
                    e_ty
                } else {
                    t_ty
                }
            }
            Expression::Lambda { params, body, .. } => {
                self.push_scope();
                for p in params {
                    self.declare_var(&p.name, p.param_type.clone(), p.span.line, p.is_mut);
                }
                let ret_ty = self.analyze_expression(body);
                self.pop_scope();
                Type::Generic("Fn".to_string(), vec![ret_ty])
            }
            Expression::ResultBuilder { name, body, .. } => {
                self.analyze_block(body);
                Type::Custom(name.clone())
            }
            Expression::Cascade { target, operations, .. } => {
                let t_ty = self.analyze_expression(target);
                for op in operations {
                    self.analyze_expression(op);
                }
                t_ty
            }
            Expression::Spread { expr, .. } => {
                self.analyze_expression(expr)
            }
            Expression::CopyExpr { target, overrides, .. } => {
                let t_ty = self.analyze_expression(target);
                for (_, val) in overrides {
                    self.analyze_expression(val);
                }
                t_ty
            }
            Expression::Pipe { lhs, rhs, .. } => {
                let l_ty = self.analyze_expression(lhs);
                let r_ty = self.analyze_expression(rhs);
                if r_ty.is_unknown() || matches!(r_ty, Type::Custom(_)) {
                    l_ty
                } else {
                    r_ty
                }
            }
            Expression::Walrus { name, expr, span } => {
                let e_ty = self.analyze_expression(expr);
                self.declare_var(name, e_ty.clone(), span.line, true);
                e_ty
            }
            Expression::ListLiteral(elements, _) => {
                let mut elem_ty = Type::Unknown;
                for el in elements {
                    match el {
                        crate::ast::expr::collections::CollectionElement::Expr(e) => {
                            let t = self.analyze_expression(e);
                            if elem_ty.is_unknown() {
                                elem_ty = t;
                            }
                        }
                        crate::ast::expr::collections::CollectionElement::Spread { expr, .. } => {
                            let _ = self.analyze_expression(expr);
                        }
                        _ => {}
                    }
                }
                Type::Slice(Box::new(elem_ty))
            }
            Expression::ListComprehension { expr, var, iterable, condition, span } => {
                let iter_ty = self.analyze_expression(iterable);
                let elem_ty = match iter_ty {
                    Type::Array(inner, _) | Type::Slice(inner) | Type::Pointer(inner) => *inner,
                    Type::Generic(ref name, ref args) if name == "Range" && !args.is_empty() => args[0].clone(),
                    Type::Str => Type::U8,
                    _ => Type::Unknown,
                };
                self.push_scope();
                self.declare_var(var, elem_ty, span.line, false);
                if let Some(c) = condition {
                    self.analyze_expression(c);
                }
                let e_ty = self.analyze_expression(expr);
                self.pop_scope();
                Type::Slice(Box::new(e_ty))
            }
            Expression::DictComprehension { key, value, key_var, val_var, iterable, condition, span } => {
                let _iter_ty = self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(key_var, Type::Unknown, span.line, false);
                if let Some(v_var) = val_var {
                    self.declare_var(v_var, Type::Unknown, span.line, false);
                }
                if let Some(c) = condition {
                    self.analyze_expression(c);
                }
                let k_ty = self.analyze_expression(key);
                let v_ty = self.analyze_expression(value);
                self.pop_scope();
                Type::Generic("Dict".to_string(), vec![k_ty, v_ty])
            }
            Expression::SetComprehension { expr, var, iterable, condition, span } => {
                let _iter_ty = self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(var, Type::Unknown, span.line, false);
                if let Some(c) = condition {
                    self.analyze_expression(c);
                }
                let e_ty = self.analyze_expression(expr);
                self.pop_scope();
                Type::Slice(Box::new(e_ty))
            }
            Expression::Block(block) => {
                self.push_scope();
                let mut last_ty = Type::Void;
                for stmt in &block.statements {
                    match stmt {
                        Statement::Return { value: Some(v), .. } => {
                            last_ty = self.analyze_expression(v);
                        }
                        Statement::Expression(e) => {
                            last_ty = self.analyze_expression(e);
                        }
                        _ => {
                            self.analyze_statement(stmt);
                        }
                    }
                }
                self.pop_scope();
                last_ty
            }
            Expression::IsPattern { expr, .. } => {
                self.analyze_expression(expr);
                Type::Bool
            }
            Expression::NamedArg { value, .. } => {
                self.analyze_expression(value)
            }
            _ => Type::Unknown,
        }
    }

    pub(crate) fn check_match_exhaustiveness(&mut self, match_ty: &Type, arms: &[MatchArm], span: &Span) {
        if let Type::Custom(ename) = match_ty {
            if let Some(e_def) = self.enums.get(ename).cloned() {
                let declared: HashSet<String> = e_def.variants.iter().map(|v| v.name.clone()).collect();
                let mut handled: HashSet<String> = HashSet::new();
                let mut has_wildcard = false;

                for arm in arms {
                    if arm.guard.is_none() {
                        match &arm.pattern {
                            Pattern::Wildcard | Pattern::Ident(_) | Pattern::Binding(_) => {
                                has_wildcard = true;
                            }
                            Pattern::Variant { variant_name, .. } => {
                                handled.insert(variant_name.clone());
                            }
                            _ => {}
                        }
                    }
                }

                if !has_wildcard {
                    let mut missing: Vec<String> = declared.difference(&handled).cloned().collect();
                    missing.sort();
                    if !missing.is_empty() {
                        self.errors.push(
                            DiagnosticError::new(
                                "E010",
                                format!(
                                    "PatternUnhandled: non-exhaustive match on enum '{}', missing variants: {:?}",
                                    ename, missing
                                ),
                                span.line,
                                span.col,
                                "PatternUnhandledError",
                            )
                            .with_expected(format!("all variants of enum '{}' covered", ename))
                            .with_actual(format!("missing constructors: {:?}", missing))
                            .with_suggestion(format!("add match arms for missing variants: {:?}", missing))
                            .with_context(format!("missing variants: {:?}", missing)),
                        );
                    }
                }
            }
        } else if let Type::Bool = match_ty {
            let mut has_true = false;
            let mut has_false = false;
            let mut has_wildcard = false;

            for arm in arms {
                if arm.guard.is_none() {
                    match &arm.pattern {
                        Pattern::Wildcard | Pattern::Ident(_) | Pattern::Binding(_) => {
                            has_wildcard = true;
                        }
                        Pattern::Literal(Literal::Bool(true)) => {
                            has_true = true;
                        }
                        Pattern::Literal(Literal::Bool(false)) => {
                            has_false = true;
                        }
                        _ => {}
                    }
                }
            }

            if !has_wildcard && (!has_true || !has_false) {
                let mut missing = Vec::new();
                if !has_true {
                    missing.push("true".to_string());
                }
                if !has_false {
                    missing.push("false".to_string());
                }
                self.errors.push(
                    DiagnosticError::new(
                        "E010",
                        format!("PatternUnhandled: non-exhaustive match on boolean, missing cases: {:?}", missing),
                        span.line,
                        span.col,
                        "PatternUnhandledError",
                    )
                    .with_expected("both true and false covered")
                    .with_actual(format!("missing cases: {:?}", missing))
                    .with_suggestion(format!("add match arms for: {:?}", missing)),
                );
            }
        }
    }

    pub(crate) fn expr_allocates(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Alloc { .. } => true,
            Expression::Call { callee, .. } => {
                if let Expression::Ident(name, _) = callee.as_ref() {
                    name == "alloc" || name.contains("create") || name == "malloc"
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

fn is_known_method(name: &str) -> bool {
    matches!(
        name,
        "copy"
            | "clone"
            | "len"
            | "map"
            | "filter"
            | "for_each"
            | "reduce"
            | "find"
            | "contains"
            | "push"
            | "pop"
            | "get"
            | "set"
            | "is_empty"
            | "keys"
            | "values"
            | "items"
            | "build"
            | "configure"
            | "connect"
            | "close"
            | "as_str"
            | "to_string"
            | "unwrap"
            | "expect"
            | "is_ok"
            | "is_err"
            | "is_some"
            | "is_none"
            | "param"
            | "sha256"
            | "acquire"
            | "setMethod"
            | "setBody"
            | "setTitle"
            | "setWidth"
            | "open"
            | "refresh"
            | "save"
            | "active"
            | "valid"
            | "id"
            | "name"
    )
}

fn get_method_return_type(name: &str) -> Type {
    match name {
        "len" => Type::I64,
        "to_string" | "as_str" => Type::Str,
        "is_ok" | "is_err" | "is_some" | "is_none" | "is_empty" | "contains" => Type::Bool,
        _ => Type::Unknown,
    }
}

fn is_known_builtin(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "println"
            | "eprintln"
            | "assert"
            | "assert_eq"
            | "len"
            | "malloc"
            | "free"
            | "memcpy"
            | "memset"
            | "strlen"
            | "sin"
            | "cos"
            | "sqrt"
            | "tan"
            | "fabs"
            | "pow"
            | "exit"
            | "time"
            | "clock"
            | "socket"
            | "connect"
            | "send"
            | "recv"
            | "close"
            | "atomic_add"
            | "atomic_sub"
            | "atomic_load"
            | "atomic_store"
            | "plot"
            | "load_heavy_data"
            | "widget_tree"
            | "node"
            | "div"
            | "stack"
            | "button"
            | "column"
            | "row"
            | "text"
            | "view"
            | "card"
            | "container"
            | "SqlValue"
            | "crypto"
            | "authority"
            | "query"
            | "render_html"
            | "items"
            | "users"
            | "accounts"
            | "points"
            | "point"
            | "builder"
            | "state"
            | "HttpRequest"
            | "getNode"
            | "get_tuple"
            | "calculate"
            | "get_optional_list"
            | "call_subroutine"
            | "double"
            | "increment"
            | "filter_active"
            | "sort_by_age"
            | "extract_names"
            | "nums"
            | "data"
            | "optionalItems"
            | "required_data"
            | "maybe_extra_data"
            | "user_dict"
            | "args"
            | "input"
            | "output"
            | "break"
            | "continue"
            | "end_channel_create"
            | "end_channel_send"
            | "end_channel_recv"
            | "end_channel_close"
            | "end_gpu_create_buffer"
            | "end_gpu_dispatch"
            | "end_gpu_destroy_buffer"
            | "end_ui_canvas_create"
            | "end_ui_canvas_clear"
            | "end_ui_canvas_draw_rect"
            | "end_ui_canvas_draw_circle"
            | "end_ui_canvas_get_pixel"
            | "end_ui_canvas_destroy"
            | "end_sqlite_open"
            | "end_sqlite_execute"
            | "end_sqlite_query"
            | "end_sqlite_begin"
            | "end_sqlite_commit"
            | "end_sqlite_rollback"
            | "end_sqlite_close"
            | "end_pg_connect"
            | "end_pg_execute"
            | "end_pg_execute_params"
            | "end_pg_query"
            | "end_pg_query_params"
            | "end_pg_begin"
            | "end_pg_commit"
            | "end_pg_rollback"
            | "end_pg_close"
            | "end_net_tcp_listen"
            | "end_net_tcp_accept"
            | "end_net_set_nonblocking"
            | "end_net_tcp_connect"
            | "end_net_tcp_send"
            | "end_net_tcp_recv"
            | "end_net_tcp_close"
            | "_"
    )
}

fn get_builtin_return_type(name: &str) -> Type {
    match name {
        "len" | "strlen" | "calculate" | "get_tuple" | "getNode" | "get_optional_list" | "inline_c_expr" | "end_channel_create" | "end_gpu_create_buffer" | "end_gpu_dispatch" | "end_ui_canvas_create" | "end_ui_canvas_get_pixel" | "end_sqlite_open" | "end_sqlite_execute" | "end_sqlite_begin" | "end_sqlite_commit" | "end_sqlite_rollback" | "end_pg_connect" | "end_pg_execute" | "end_pg_execute_params" | "end_pg_begin" | "end_pg_commit" | "end_pg_rollback" | "end_net_tcp_listen" | "end_net_tcp_accept" | "end_net_set_nonblocking" | "end_net_tcp_connect" | "end_net_tcp_send" => Type::I64,
        "sin" | "cos" | "sqrt" | "tan" | "fabs" | "pow" => Type::F64,
        "malloc" => Type::Pointer(Box::new(Type::Void)),
        "atomic_add" | "atomic_sub" | "atomic_load" => Type::I64,
        "end_channel_recv" | "end_sqlite_query" | "end_pg_query" | "end_pg_query_params" | "end_net_tcp_recv" => Type::Str,
        "end_gpu_destroy_buffer" | "end_ui_canvas_clear" | "end_ui_canvas_draw_rect" | "end_ui_canvas_draw_circle" | "end_ui_canvas_destroy" | "end_sqlite_close" | "end_pg_close" | "end_net_tcp_close" => Type::Void,
        s if s.starts_with(|c: char| c.is_uppercase()) => Type::Custom(s.to_string()),
        _ => Type::Unknown,
    }
}
