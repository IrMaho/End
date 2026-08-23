use super::state::{escape_c_string, CBackend};
use crate::ast::{BinaryOp, Expression, Literal, Pattern, Statement, Type, UnaryOp};

impl CBackend {
    pub(crate) fn gen_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Lit(lit, _) => match lit {
                Literal::Int(n) => n.to_string(),
                Literal::Float(f) => format!("{:.6}f", f),
                Literal::String(s) => format!("\"{}\"", escape_c_string(s)),
                Literal::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                Literal::Null => "NULL".to_string(),
            },
            Expression::Ident(name, _) => name.clone(),
            Expression::Binary { left, op, right, .. } => {
                let l = self.gen_expression(left);
                let r = self.gen_expression(right);
                let is_op_l = self.is_operation_expr(left);
                let is_op_r = self.is_operation_expr(right);
                match op {
                    BinaryOp::Add => {
                        if is_op_l && is_op_r {
                            format!("end_compose_ops({}, {})", l, r)
                        } else {
                            let is_str_l = matches!(left.as_ref(), Expression::Lit(Literal::String(_), _)) || l.starts_with('"') || l.contains("_str") || l.contains("_concat");
                            let is_str_r = matches!(right.as_ref(), Expression::Lit(Literal::String(_), _)) || r.starts_with('"') || r.contains("_str") || r.contains("_concat");
                            if is_str_l || is_str_r {
                                format!("_end_str_concat({}, {})", l, r)
                            } else {
                                format!("({} + {})", l, r)
                            }
                        }
                    },
                    BinaryOp::Sub => format!("({} - {})", l, r),
                    BinaryOp::Mul => {
                        if is_op_l && r.chars().all(|c| c.is_digit(10)) {
                            format!("end_repeat_op({}, {})", l, r)
                        } else {
                            format!("({} * {})", l, r)
                        }
                    },
                    BinaryOp::Div => format!("({} / {})", l, r),
                    BinaryOp::Mod => format!("({} % {})", l, r),
                    BinaryOp::Equal => {
                        let mut is_str = matches!(left.as_ref(), Expression::Lit(Literal::String(_), _)) || matches!(right.as_ref(), Expression::Lit(Literal::String(_), _))
                            || l.starts_with('"') || r.starts_with('"')
                            || l.contains("str") || r.contains("str")
                            || l.contains("token") || r.contains("hash")
                            || l.contains("text") || r.contains("msg");
                        if l == "true" || l == "false" || r == "true" || r == "false" { is_str = false; }
                        if is_str && !l.chars().all(|c| c.is_digit(10)) && !r.chars().all(|c| c.is_digit(10)) {
                            format!("(strcmp({}, {}) == 0)", l, r)
                        } else {
                            format!("({} == {})", l, r)
                        }
                    },
                    BinaryOp::NotEqual => {
                        let mut is_str = matches!(left.as_ref(), Expression::Lit(Literal::String(_), _)) || matches!(right.as_ref(), Expression::Lit(Literal::String(_), _))
                            || l.starts_with('"') || r.starts_with('"')
                            || l.contains("str") || r.contains("str")
                            || l.contains("token") || r.contains("hash")
                            || l.contains("text") || r.contains("msg");
                        if l == "true" || l == "false" || r == "true" || r == "false" { is_str = false; }
                        if is_str && !l.chars().all(|c| c.is_digit(10)) && !r.chars().all(|c| c.is_digit(10)) {
                            format!("(strcmp({}, {}) != 0)", l, r)
                        } else {
                            format!("({} != {})", l, r)
                        }
                    },
                    BinaryOp::LessThan => format!("({} < {})", l, r),
                    BinaryOp::LessEqual => format!("({} <= {})", l, r),
                    BinaryOp::GreaterThan => format!("({} > {})", l, r),
                    BinaryOp::GreaterEqual => format!("({} >= {})", l, r),
                    BinaryOp::And => format!("({} && {})", l, r),
                    BinaryOp::Or => format!("({} || {})", l, r),
                    BinaryOp::Shl => format!("(((uint64_t)({})) << ({}) )", l, r),
                    BinaryOp::Shr => {
                        if is_op_l {
                            format!("end_compose_ops({}, {})", l, r)
                        } else {
                            format!("(((uint64_t)({})) >> ({}) )", l, r)
                        }
                    },
                    BinaryOp::BitAnd => {
                        if is_op_l {
                            format!("end_parallel_op({}, {})", l, r)
                        } else {
                            format!("(((uint64_t)({})) & ((uint64_t)({})))", l, r)
                        }
                    },
                    BinaryOp::BitOr => {
                        if is_op_l {
                            format!("end_alternative_op({}, {})", l, r)
                        } else {
                            format!("(((uint64_t)({})) | ((uint64_t)({})))", l, r)
                        }
                    },
                    BinaryOp::BitXor => format!("(((uint64_t)({})) ^ ((uint64_t)({})))", l, r),
                }
            }
            Expression::Unary { op, expr, .. } => {
                let e = self.gen_expression(expr);
                match op {
                    UnaryOp::Negate => format!("(-{})", e),
                    UnaryOp::Not => format!("(!{})", e),
                    UnaryOp::AddressOf => format!("(&{})", e),
                    UnaryOp::Deref => format!("(*{})", e),
                    UnaryOp::BitNot => format!("(~{})", e),
                }
            }
            Expression::Call { callee, args, .. } => {
                let callee_str = self.gen_expression(callee);
                if callee_str == "println" {
                    if let Some(first_arg) = args.first() {
                        let arg_str = self.gen_expression(first_arg);
                        return format!("end_println({})", arg_str);
                    }
                }

                // Check for struct extension methods & module methods
                if let Expression::FieldAccess { object, field, .. } = callee.as_ref() {
                    let obj_str = self.gen_expression(object);

                    // 1. Check if object is a module name, e.g. MathModule.calc(...)
                    if let Expression::Ident(mod_name, _) = object.as_ref() {
                        if self.module_methods.contains_key(mod_name) {
                            let mut mod_args = Vec::new();
                            for a in args {
                                mod_args.push(self.gen_expression(a));
                            }
                            return format!("{}_{}({})", mod_name, field, mod_args.join(", "));
                        }
                    }

                    // 2. Struct Method / Extension method dispatch
                    let is_ptr = if let Expression::Ident(id, _) = object.as_ref() {
                        self.var_types.get(id).map_or(false, |t| matches!(t, Type::Pointer(_) | Type::Box(_) | Type::Rc(_) | Type::Arc(_)))
                    } else {
                        false
                    };
                    let receiver_arg = if is_ptr { obj_str.clone() } else { format!("&{}", obj_str) };
                    let mut ext_args = vec![receiver_arg];
                    for a in args {
                        ext_args.push(self.gen_expression(a));
                    }

                    if let Expression::Ident(id, _) = object.as_ref() {
                        if let Some(Type::Custom(struct_name)) = self.var_types.get(id) {
                            return format!("{}_{}({})", struct_name, field, ext_args.join(", "));
                        }
                    }

                    for (st_name, methods) in &self.struct_methods {
                        if methods.contains(field) {
                            return format!("{}_{}({})", st_name, field, ext_args.join(", "));
                        }
                    }
                }

                let mut args_str = Vec::new();
                for a in args {
                    args_str.push(self.gen_expression(a));
                }
                format!("{}({})", callee_str, args_str.join(", "))
            }
            Expression::FieldAccess { object, field, .. } => {
                let obj_str = self.gen_expression(object);
                if let Expression::Ident(mod_name, _) = object.as_ref() {
                    if self.module_methods.contains_key(mod_name) {
                        return format!("{}_{}", mod_name, field);
                    }
                }
                if let Expression::Ident(name, _) = object.as_ref() {
                    if let Some(ty) = self.var_types.get(name) {
                        match ty {
                            Type::Pointer(_) | Type::Box(_) | Type::Rc(_) | Type::Arc(_) => {
                                return format!("{}->{}", obj_str, field);
                            }
                            _ => {}
                        }
                    }
                }
                format!("{}.{}", obj_str, field)
            }
            Expression::Index { array, index, .. } => {
                let arr_str = self.gen_expression(array);
                let idx_str = self.gen_expression(index);
                format!("{}[{}]", arr_str, idx_str)
            }
            Expression::StructInit { name, fields, .. } => {
                let mut field_inits = Vec::new();
                for (fname, fval) in fields {
                    field_inits.push(format!(".{} = {}", fname, self.gen_expression(fval)));
                }
                format!("({}){{ {} }}", name, field_inits.join(", "))
            }
            Expression::EnumInit { enum_name, variant_name, payload, .. } => {
                let en = enum_name.clone().unwrap_or_else(|| self.find_enum_for_variant(variant_name));
                if let Some(p) = payload {
                    format!("({}){{ .tag = {}_{}, .data.{} = {} }}", en, en, variant_name, variant_name, self.gen_expression(p))
                } else {
                    format!("({}){{ .tag = {}_{} }}", en, en, variant_name)
                }
            }
            Expression::InlineC { code, .. } => code.clone(),
            Expression::Pipe { lhs, rhs, .. } => {
                let lhs_str = self.gen_expression(lhs.as_ref());
                match rhs.as_ref() {
                    Expression::Call { callee, args, .. } => {
                        let callee_str = self.gen_expression(callee.as_ref());
                        let mut all_args = vec![lhs_str];
                        for a in args {
                            all_args.push(self.gen_expression(a));
                        }
                        format!("{}({})", callee_str, all_args.join(", "))
                    }
                    Expression::Ident(name, _) => {
                        format!("{}({})", name, lhs_str)
                    }
                    _ => {
                        let rhs_str = self.gen_expression(rhs.as_ref());
                        format!("{}({})", rhs_str, lhs_str)
                    }
                }
            }
            Expression::Alloc { target_type, .. } => {
                if let Some(curr_region) = self.active_regions.last() {
                    match target_type {
                        Type::Array(inner, size) => format!("({}*)end_arena_alloc(region_{}, (size_t)({}) * sizeof({}))", self.map_type(inner), curr_region, size, self.map_type(inner)),
                        _ => format!("({}*)end_arena_alloc(region_{}, sizeof({}))", self.map_type(target_type), curr_region, self.map_type(target_type)),
                    }
                } else {
                    match target_type {
                        Type::Array(inner, size) => format!("({}*)malloc((size_t)({}) * sizeof({}))", self.map_type(inner), size, self.map_type(inner)),
                        _ => format!("({}*)malloc(sizeof({}))", self.map_type(target_type), self.map_type(target_type)),
                    }
                }
            }
            Expression::Promote { expr, target_region, .. } => {
                let e = self.gen_expression(expr);
                format!("end_promote({}, region_{}, sizeof(*({})))", e, target_region, e)
            }
            Expression::Catch { expr, .. } => {
                self.gen_expression(expr)
            }
            Expression::Match { expr, arms, .. } => {
                let target = self.gen_expression(expr);
                let mut result = String::new();
                let mut open_parens = 0;
                for arm in arms {
                    let arm_val = if let Some(last) = arm.body.statements.last() {
                        match last {
                            Statement::Expression(e) => self.gen_expression(e),
                            Statement::Return { value: Some(e), .. } => self.gen_expression(e),
                            _ => "0".to_string(),
                        }
                    } else {
                        "0".to_string()
                    };

                    match &arm.pattern {
                        Pattern::Literal(lit) => {
                            let lit_str = match lit {
                                Literal::Int(n) => n.to_string(),
                                Literal::Float(f) => format!("{:.6}f", f),
                                Literal::Bool(b) => if *b { "1".into() } else { "0".into() },
                                Literal::String(s) => format!("\"{}\"", escape_c_string(s)),
                                Literal::Null => "0".into(),
                            };
                            result.push_str(&format!("(({} == {}) ? ({}) : ", target, lit_str, arm_val));
                            open_parens += 1;
                        }
                        Pattern::Wildcard | Pattern::Ident(_) | Pattern::Binding(_) | Pattern::Tuple(_) | Pattern::Struct { .. } => {
                            result.push_str(&format!("({})", arm_val));
                            for _ in 0..open_parens {
                                result.push(')');
                            }
                            return result;
                        }
                        Pattern::Variant { variant_name, enum_name, .. } => {
                            let en = enum_name.clone().unwrap_or_else(|| self.find_enum_for_variant(variant_name));
                            result.push_str(&format!("(({}.tag == {}_{}) ? ({}) : ", target, en, variant_name, arm_val));
                            open_parens += 1;
                        }
                    }
                }
                result.push_str("0");
                for _ in 0..open_parens {
                    result.push(')');
                }
                result
            }
            Expression::Block(_) => "0".to_string(),
            Expression::NameOf { target, .. } => format!("\"{}\"", escape_c_string(target)),
            Expression::PathOf { target, .. } => format!("\"{}\"", escape_c_string(target)),
            Expression::TypeOf { expr, .. } => {
                let e = self.gen_expression(expr);
                format!("\"type_of({})\"", escape_c_string(&e))
            }
            Expression::DocOf { target, .. } => format!("\"[Documentation for symbol '{}']\"", escape_c_string(target)),
            Expression::CodeOf { code, .. } => format!("\"{}\"", escape_c_string(code)),
            Expression::Dbg { expr, code, span } => {
                let e = self.gen_expression(expr);
                let clean_file = span.file.replace('\\', "/");
                format!("(printf(\"\\033[1;36m[DBG %s:%d]\\033[0m \\033[1;33m%s\\033[0m = \", \"{}\", {}, \"{}\"), end_println({}), {})", clean_file, span.line, escape_c_string(code), e, e)
            }
            Expression::AssertDebug { condition, code, span } => {
                let cond = self.gen_expression(condition);
                let clean_file = span.file.replace('\\', "/");
                format!("if (!({})) {{ fprintf(stderr, \"\\033[1;31m[ASSERTION FAILURE %s:%d]\\033[0m Condition failed: %s\\n\", \"{}\", {}, \"{}\"); abort(); }}", cond, clean_file, span.line, escape_c_string(code))
            }
            Expression::Translate { key, .. } => format!("\"[Localized: {}]\"", escape_c_string(key)),
            Expression::FieldsOf { target, .. } => format!("\"[Fields of struct {}]\"", escape_c_string(target)),
            Expression::SqlExpr { expr, .. } => {
                let e = self.gen_expression(expr);
                format!("\"SELECT * WHERE {}\"", escape_c_string(&e))
            }
            Expression::Cast { expr, target_type, .. } => {
                let inner = self.gen_expression(expr);
                let c_ty = self.map_type(target_type);
                format!("(({})({}))", c_ty, inner)
            }
            Expression::Await { expr, .. } => {
                self.gen_expression(expr)
            }
            Expression::UnitLit { value, unit, .. } => {
                match unit.as_str() {
                    "km/h" => format!("((int64_t)({:.2} * 1000.0 / 3600.0))", value),
                    "m/s" => format!("((int64_t){:.0})", value),
                    "usd" | "eur" => format!("((int64_t)({:.2} * 100.0))", value),
                    _ => format!("((int64_t){:.0})", value),
                }
            }
            Expression::NullCollapse { left, right, .. } => {
                let l = self.gen_expression(left);
                let r = self.gen_expression(right);
                format!("((void)({}), ({}))", l, r)
            }
            Expression::OperationLiteral { name, .. } => {
                format!("end_create_operation(\"{}\")", name.as_deref().unwrap_or("anon"))
            }
            Expression::Compose { ops, .. } => {
                let ops_str = ops.iter().map(|o| self.gen_expression(o)).collect::<Vec<_>>().join(", ");
                format!("end_compose_ops({})", ops_str)
            }
            Expression::Repeat { op, count, is_retry, .. } => {
                let o = self.gen_expression(op);
                let c = self.gen_expression(count);
                if *is_retry {
                    format!("end_retry_op({}, {})", o, c)
                } else {
                    format!("end_repeat_op({}, {})", o, c)
                }
            }
            Expression::Alternative { left, right, .. } => {
                let l = self.gen_expression(left);
                let r = self.gen_expression(right);
                format!("end_alternative_op({}, {})", l, r)
            }
            Expression::Parallel { left, right, .. } => {
                let l = self.gen_expression(left);
                let r = self.gen_expression(right);
                format!("end_parallel_op({}, {})", l, r)
            }
            Expression::ConditionalOp { op, condition, .. } => {
                let o = self.gen_expression(op);
                let c = self.gen_expression(condition);
                format!("({} ? {} : NULL)", c, o)
            }
            Expression::Memoize { op, .. } => {
                let o = self.gen_expression(op);
                format!("end_memoize_op({})", o)
            }
            _ => format!("/* expressive_expr */ 0"),
        }
    }
}
