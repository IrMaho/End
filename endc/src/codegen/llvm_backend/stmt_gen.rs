use super::state::LlvmBackend;
use crate::ast::*;
use crate::codegen::backend_trait::BackendError;
use std::fmt::Write;

impl LlvmBackend {
    pub(crate) fn generate_statement(&mut self, stmt: &Statement, out: &mut String) -> Result<(), BackendError> {
        match stmt {
            Statement::VarDecl { name, var_type, initializer, .. } => {
                let (val_reg, inferred_ty) = if let Some(init) = initializer {
                    self.generate_expression(init, out)?
                } else {
                    ("0".to_string(), "i64".to_string())
                };

                let ty_str = if let Some(t) = var_type {
                    self.map_type(t)
                } else {
                    inferred_ty
                };

                let ptr_reg = self.next_temp();
                writeln!(out, "  {} = alloca {}", ptr_reg, ty_str).unwrap();
                writeln!(out, "  store {} {}, {}* {}", ty_str, val_reg, ty_str, ptr_reg).unwrap();
                self.variables.insert(name.clone(), (ty_str, ptr_reg));
            }
            Statement::Assignment { target, value, .. } => {
                let (val_reg, _) = self.generate_expression(value, out)?;
                if let Expression::Ident(name, _) = target {
                    if let Some((ty, ptr)) = self.variables.get(name).cloned() {
                        writeln!(out, "  store {} {}, {}* {}", ty, val_reg, ty, ptr).unwrap();
                    } else {
                        return Err(BackendError::CodegenFailed(format!("Undefined variable in assignment: {}", name)));
                    }
                } else if let Expression::FieldAccess { object, field, .. } = target {
                    if let Expression::Ident(obj_name, _) = object.as_ref() {
                        if let Some((struct_ptr_ty, struct_ptr)) = self.variables.get(obj_name).cloned() {
                            let struct_name = struct_ptr_ty.trim_start_matches('%').trim_end_matches('*');
                            let field_idx = self.get_field_index(struct_name, field);
                            let gep_reg = self.next_temp();
                            writeln!(out, "  {} = getelementptr inbounds {}, {} {}, i32 0, i32 {}", 
                                     gep_reg, struct_name, struct_ptr_ty, struct_ptr, field_idx).unwrap();
                            writeln!(out, "  store i64 {}, i64* {}", val_reg, gep_reg).unwrap();
                        }
                    }
                }
            }
            Statement::If { condition, then_block, else_block, .. } => {
                let (cond_reg, _) = self.generate_expression(condition, out)?;
                let then_lbl = self.next_label("then");
                let else_lbl = self.next_label("else");
                let merge_lbl = self.next_label("merge");

                if else_block.is_some() {
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cond_reg, then_lbl, else_lbl).unwrap();
                } else {
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cond_reg, then_lbl, merge_lbl).unwrap();
                }

                // Then Block
                writeln!(out, "{}:", then_lbl).unwrap();
                for s in &then_block.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  br label %{}", merge_lbl).unwrap();

                // Else Block
                if let Some(eb) = else_block {
                    writeln!(out, "{}:", else_lbl).unwrap();
                    for s in &eb.statements {
                        self.generate_statement(s, out)?;
                    }
                    writeln!(out, "  br label %{}", merge_lbl).unwrap();
                }

                writeln!(out, "{}:", merge_lbl).unwrap();
            }
            Statement::Guard { condition, else_block, .. } => {
                let (cond_reg, _) = self.generate_expression(condition, out)?;
                let else_lbl = self.next_label("guard_else");
                let merge_lbl = self.next_label("guard_merge");

                writeln!(out, "  br i1 {}, label %{}, label %{}", cond_reg, merge_lbl, else_lbl).unwrap();

                writeln!(out, "{}:", else_lbl).unwrap();
                for s in &else_block.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  br label %{}", merge_lbl).unwrap();

                writeln!(out, "{}:", merge_lbl).unwrap();
            }
            Statement::While { condition, body, .. } => {
                let cond_lbl = self.next_label("while_cond");
                let body_lbl = self.next_label("while_body");
                let end_lbl = self.next_label("while_end");

                writeln!(out, "  br label %{}", cond_lbl).unwrap();
                writeln!(out, "{}:", cond_lbl).unwrap();
                let (cond_reg, _) = self.generate_expression(condition, out)?;
                writeln!(out, "  br i1 {}, label %{}, label %{}", cond_reg, body_lbl, end_lbl).unwrap();

                writeln!(out, "{}:", body_lbl).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  br label %{}", cond_lbl).unwrap();

                writeln!(out, "{}:", end_lbl).unwrap();
            }
            Statement::ForIn { item_name, iterable, body, .. } => {
                let (iter_reg, _) = self.generate_expression(iterable, out)?;
                let counter_ptr = self.next_temp();
                writeln!(out, "  {} = alloca i64", counter_ptr).unwrap();
                writeln!(out, "  store i64 0, i64* {}", counter_ptr).unwrap();
                self.variables.insert(item_name.clone(), ("i64".to_string(), counter_ptr.clone()));

                let cond_lbl = self.next_label("for_cond");
                let body_lbl = self.next_label("for_body");
                let end_lbl = self.next_label("for_end");

                writeln!(out, "  br label %{}", cond_lbl).unwrap();
                writeln!(out, "{}:", cond_lbl).unwrap();
                let current_i = self.next_temp();
                writeln!(out, "  {} = load i64, i64* {}", current_i, counter_ptr).unwrap();
                let cmp_reg = self.next_temp();
                writeln!(out, "  {} = icmp slt i64 {}, {}", cmp_reg, current_i, iter_reg).unwrap();
                writeln!(out, "  br i1 {}, label %{}, label %{}", cmp_reg, body_lbl, end_lbl).unwrap();

                writeln!(out, "{}:", body_lbl).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                let inc_reg = self.next_temp();
                writeln!(out, "  {} = add i64 {}, 1", inc_reg, current_i).unwrap();
                writeln!(out, "  store i64 {}, i64* {}", inc_reg, counter_ptr).unwrap();
                writeln!(out, "  br label %{}", cond_lbl).unwrap();

                writeln!(out, "{}:", end_lbl).unwrap();
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    let (val_reg, ty) = self.generate_expression(v, out)?;
                    writeln!(out, "  ret {} {}", ty, val_reg).unwrap();
                } else {
                    writeln!(out, "  ret void").unwrap();
                }
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr, out)?;
            }
            Statement::RegionBlock { name, body, .. } => {
                let arena_ptr = self.next_temp();
                writeln!(out, "  ; Region Arena Allocation ({})", name).unwrap();
                writeln!(out, "  {} = call i8* @end_arena_create(i64 65536)", arena_ptr).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  call void @end_arena_destroy(i8* {})", arena_ptr).unwrap();
            }
            Statement::LeaseBlock { name, initializer, body, .. } => {
                let (res_reg, res_ty) = self.generate_expression(initializer, out)?;
                let lease_ptr = self.next_temp();
                writeln!(out, "  ; Tier 0 Scoped Lease ({})", name).unwrap();
                writeln!(out, "  {} = alloca {}", lease_ptr, res_ty).unwrap();
                writeln!(out, "  call void @llvm.lifetime.start.p0i8(i64 8, i8* {})", lease_ptr).unwrap();
                writeln!(out, "  store {} {}, {}* {}", res_ty, res_reg, res_ty, lease_ptr).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  call void @llvm.lifetime.end.p0i8(i64 8, i8* {})", lease_ptr).unwrap();
            }
            Statement::AsmBlock { code, .. } => {
                writeln!(out, "  call void asm sideeffect \"{}\", \"\"()", code.replace("\"", "\\\"")).unwrap();
            }
            Statement::Defer { expr, .. } => {
                self.generate_expression(expr, out)?;
            }
            _ => {}
        }
        Ok(())
    }
}
