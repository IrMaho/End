use super::state::LlvmBackend;
use crate::ast::*;
use crate::codegen::backend_trait::BackendError;
use std::fmt::Write;

impl LlvmBackend {
    pub(crate) fn generate_expression(&mut self, expr: &Expression, out: &mut String) -> Result<(String, String), BackendError> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => Ok((n.to_string(), "i64".to_string())),
            Expression::Lit(Literal::Float(f), _) => Ok((format!("{:.6}", f), "double".to_string())),
            Expression::Lit(Literal::Bool(b), _) => Ok(((if *b { 1 } else { 0 }).to_string(), "i1".to_string())),
            Expression::Lit(Literal::String(s), _) => {
                let glob_name = self.register_string_literal(s);
                let byte_len = s.as_bytes().len() + 1;
                let gep_reg = self.next_temp();
                writeln!(out, "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0", 
                         gep_reg, byte_len, byte_len, glob_name).unwrap();
                Ok((gep_reg, "i8*".to_string()))
            }
            Expression::Lit(Literal::Null, _) => Ok(("null".to_string(), "i8*".to_string())),
            Expression::Ident(name, _) => {
                if let Some((ty, ptr)) = self.variables.get(name).cloned() {
                    let load_reg = self.next_temp();
                    writeln!(out, "  {} = load {}, {}* {}", load_reg, ty, ty, ptr).unwrap();
                    Ok((load_reg, ty))
                } else {
                    Ok((format!("@{}", name), "i8*".to_string()))
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let (lhs, l_ty) = self.generate_expression(left, out)?;
                let (rhs, r_ty) = self.generate_expression(right, out)?;
                let res_reg = self.next_temp();

                let is_float = l_ty == "double" || l_ty == "float" || r_ty == "double" || r_ty == "float";
                let is_str = l_ty == "i8*" || r_ty == "i8*";

                if is_str && *op == BinaryOp::Add {
                    writeln!(out, "  {} = call i8* @end_str_concat(i8* {}, i8* {})", res_reg, lhs, rhs).unwrap();
                    return Ok((res_reg, "i8*".to_string()));
                }

                match op {
                    BinaryOp::Add => {
                        let ins = if is_float { "fadd double" } else { "add i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Sub => {
                        let ins = if is_float { "fsub double" } else { "sub i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Mul => {
                        let ins = if is_float { "fmul double" } else { "mul i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Div => {
                        let ins = if is_float { "fdiv double" } else { "sdiv i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Mod => {
                        writeln!(out, "  {} = srem i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::Equal => {
                        let ins = if is_float { "fcmp oeq double" } else { "icmp eq i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::NotEqual => {
                        let ins = if is_float { "fcmp one double" } else { "icmp ne i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::LessThan => {
                        let ins = if is_float { "fcmp olt double" } else { "icmp slt i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::LessEqual => {
                        let ins = if is_float { "fcmp ole double" } else { "icmp sle i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::GreaterThan => {
                        let ins = if is_float { "fcmp ogt double" } else { "icmp sgt i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::GreaterEqual => {
                        let ins = if is_float { "fcmp oge double" } else { "icmp sge i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::BitAnd => {
                        writeln!(out, "  {} = and i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::BitOr => {
                        writeln!(out, "  {} = or i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::BitXor => {
                        writeln!(out, "  {} = xor i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::Shl => {
                        writeln!(out, "  {} = shl i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::Shr => {
                        writeln!(out, "  {} = ashr i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::And => {
                        writeln!(out, "  {} = and i1 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::Or => {
                        writeln!(out, "  {} = or i1 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                }
            }
            Expression::Unary { op, expr: inner_expr, .. } => {
                let (val, ty) = self.generate_expression(inner_expr, out)?;
                let res_reg = self.next_temp();
                match op {
                    UnaryOp::Negate => {
                        writeln!(out, "  {} = sub i64 0, {}", res_reg, val).unwrap();
                        Ok((res_reg, ty))
                    }
                    UnaryOp::Not => {
                        writeln!(out, "  {} = xor i1 {}, 1", res_reg, val).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    UnaryOp::BitNot => {
                        writeln!(out, "  {} = xor i64 {}, -1", res_reg, val).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    _ => Ok((val, ty)),
                }
            }
            Expression::Call { callee, args, .. } => {
                let callee_name = if let Expression::Ident(name, _) = callee.as_ref() {
                    name.clone()
                } else {
                    "unknown_callee".to_string()
                };

                // Standard Print Specialization
                if callee_name == "println" || callee_name == "print" {
                    if let Some(first_arg) = args.first() {
                        let (arg_val, arg_ty) = self.generate_expression(first_arg, out)?;
                        let fmt_str = if arg_ty == "i8*" {
                            if callee_name == "println" { "%s\\0A" } else { "%s" }
                        } else if arg_ty == "double" || arg_ty == "float" {
                            if callee_name == "println" { "%f\\0A" } else { "%f" }
                        } else {
                            if callee_name == "println" { "%lld\\0A" } else { "%lld" }
                        };
                        let fmt_name = self.register_string_literal(fmt_str);
                        let gep_reg = self.next_temp();
                        writeln!(out, "  {} = getelementptr inbounds [5 x i8], [5 x i8]* {}, i32 0, i32 0", 
                                 gep_reg, fmt_name).unwrap();
                        let call_reg = self.next_temp();
                        writeln!(out, "  {} = call i32 (i8*, ...) @printf(i8* {}, {} {})", 
                                 call_reg, gep_reg, arg_ty, arg_val).unwrap();
                        return Ok((call_reg, "i32".to_string()));
                    }
                }

                let mut arg_vals = Vec::new();
                for a in args {
                    let (a_val, a_ty) = self.generate_expression(a, out)?;
                    arg_vals.push(format!("{} {}", a_ty, a_val));
                }

                let res_reg = self.next_temp();
                writeln!(out, "  {} = call i64 @{}({})", res_reg, callee_name, arg_vals.join(", ")).unwrap();
                Ok((res_reg, "i64".to_string()))
            }
            Expression::StructInit { name, fields, .. } => {
                let struct_alloca = self.next_temp();
                writeln!(out, "  {} = alloca %struct.{}", struct_alloca, name).unwrap();
                for (idx, (_, f_expr)) in fields.iter().enumerate() {
                    let (f_val, f_ty) = self.generate_expression(f_expr, out)?;
                    let field_gep = self.next_temp();
                    writeln!(out, "  {} = getelementptr inbounds %struct.{}, %struct.{}* {}, i32 0, i32 {}", 
                             field_gep, name, name, struct_alloca, idx).unwrap();
                    writeln!(out, "  store {} {}, {}* {}", f_ty, f_val, f_ty, field_gep).unwrap();
                }
                Ok((struct_alloca, format!("%struct.{}*", name)))
            }
            Expression::FieldAccess { object, field, .. } => {
                if let Expression::Ident(obj_name, _) = object.as_ref() {
                    if let Some((struct_ptr_ty, struct_ptr)) = self.variables.get(obj_name).cloned() {
                        let struct_name = struct_ptr_ty.trim_start_matches('%').trim_end_matches('*');
                        let field_idx = self.get_field_index(struct_name, field);
                        let gep_reg = self.next_temp();
                        writeln!(out, "  {} = getelementptr inbounds {}, {} {}, i32 0, i32 {}", 
                                 gep_reg, struct_name, struct_ptr_ty, struct_ptr, field_idx).unwrap();
                        let load_reg = self.next_temp();
                        writeln!(out, "  {} = load i64, i64* {}", load_reg, gep_reg).unwrap();
                        return Ok((load_reg, "i64".to_string()));
                    }
                }
                Ok(("0".to_string(), "i64".to_string()))
            }
            Expression::Index { array, index, .. } => {
                let (arr_val, _) = self.generate_expression(array, out)?;
                let (idx_val, _) = self.generate_expression(index, out)?;
                let gep_reg = self.next_temp();
                writeln!(out, "  {} = getelementptr inbounds i64, i64* {}, i64 {}", gep_reg, arr_val, idx_val).unwrap();
                let load_reg = self.next_temp();
                writeln!(out, "  {} = load i64, i64* {}", load_reg, gep_reg).unwrap();
                Ok((load_reg, "i64".to_string()))
            }
            Expression::Cast { expr: inner, target_type, .. } => {
                let (val, ty) = self.generate_expression(inner, out)?;
                let target_ty_str = self.map_type(target_type);
                let res_reg = self.next_temp();
                if ty == "i64" && target_ty_str == "double" {
                    writeln!(out, "  {} = sitofp i64 {} to double", res_reg, val).unwrap();
                } else if ty == "double" && target_ty_str == "i64" {
                    writeln!(out, "  {} = fptosi double {} to i64", res_reg, val).unwrap();
                } else {
                    writeln!(out, "  {} = bitcast {} {} to {}", res_reg, ty, val, target_ty_str).unwrap();
                }
                Ok((res_reg, target_ty_str))
            }
            Expression::Pipe { lhs, rhs, .. } => {
                let (l_val, l_ty) = self.generate_expression(lhs, out)?;
                if let Expression::Call { callee, args, .. } = rhs.as_ref() {
                    let callee_name = if let Expression::Ident(n, _) = callee.as_ref() { n.clone() } else { "pipe_fn".to_string() };
                    let mut call_args = vec![format!("{} {}", l_ty, l_val)];
                    for a in args {
                        let (a_val, a_ty) = self.generate_expression(a, out)?;
                        call_args.push(format!("{} {}", a_ty, a_val));
                    }
                    let res_reg = self.next_temp();
                    writeln!(out, "  {} = call i64 @{}({})", res_reg, callee_name, call_args.join(", ")).unwrap();
                    Ok((res_reg, "i64".to_string()))
                } else if let Expression::Ident(callee_name, _) = rhs.as_ref() {
                    let res_reg = self.next_temp();
                    writeln!(out, "  {} = call i64 @{}({} {})", res_reg, callee_name, l_ty, l_val).unwrap();
                    Ok((res_reg, "i64".to_string()))
                } else {
                    Ok((l_val, l_ty))
                }
            }
            Expression::Match { expr: subj_expr, arms, .. } => {
                let (subj_val, _) = self.generate_expression(subj_expr, out)?;
                let res_alloca = self.next_temp();
                writeln!(out, "  {} = alloca i64", res_alloca).unwrap();
                let merge_lbl = self.next_label("match_merge");

                for (idx, arm) in arms.iter().enumerate() {
                    let arm_lbl = self.next_label(&format!("match_arm_{}", idx));
                    let next_arm_lbl = self.next_label(&format!("match_next_{}", idx));
                    
                    let pat_val = match &arm.pattern {
                        Pattern::Literal(Literal::Int(n)) => n.to_string(),
                        Pattern::Literal(Literal::Bool(b)) => if *b { "1".to_string() } else { "0".to_string() },
                        _ => idx.to_string(),
                    };

                    let cmp_reg = self.next_temp();
                    writeln!(out, "  {} = icmp eq i64 {}, {}", cmp_reg, subj_val, pat_val).unwrap();
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cmp_reg, arm_lbl, next_arm_lbl).unwrap();

                    writeln!(out, "{}:", arm_lbl).unwrap();
                    for s in &arm.body.statements {
                        self.generate_statement(s, out)?;
                    }
                    writeln!(out, "  store i64 {}, i64* {}", pat_val, res_alloca).unwrap();
                    writeln!(out, "  br label %{}", merge_lbl).unwrap();

                    writeln!(out, "{}:", next_arm_lbl).unwrap();
                }
                writeln!(out, "  br label %{}", merge_lbl).unwrap();
                writeln!(out, "{}:", merge_lbl).unwrap();
                let final_res = self.next_temp();
                writeln!(out, "  {} = load i64, i64* {}", final_res, res_alloca).unwrap();
                Ok((final_res, "i64".to_string()))
            }
            _ => Ok(("0".to_string(), "i64".to_string())),
        }
    }
}
