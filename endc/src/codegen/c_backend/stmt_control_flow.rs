use super::state::CBackend;
use crate::ast::{BinaryOp, Expression, Literal, Pattern, Statement, Type};

impl CBackend {
    pub(crate) fn gen_control_flow_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::VarDecl {
                name,
                var_type,
                initializer,
                ..
            } => {
                let ty = if let Some(t) = var_type {
                    t.clone()
                } else if let Some(Expression::StructInit { name: st_name, .. }) = initializer {
                    Type::Custom(st_name.clone())
                } else if let Some(Expression::Alloc { target_type, .. }) = initializer {
                    Type::Pointer(Box::new(target_type.clone()))
                } else if let Some(Expression::Promote { expr, .. }) = initializer {
                    if let Expression::Ident(id, _) = expr.as_ref() {
                        self.var_types.get(id).cloned().unwrap_or(Type::I64)
                    } else {
                        Type::I64
                    }
                } else if let Some(init_expr) = initializer {
                    let inferred = self.infer_type(init_expr);
                    if inferred != Type::Void {
                        inferred
                    } else {
                        Type::I64
                    }
                } else {
                    Type::I64
                };
                self.declare_c_var(name, ty.clone());

                let ty_str = if let Some(t) = var_type {
                    self.map_type(t)
                } else {
                    self.map_type(&ty)
                };

                if let Some(init) = initializer {
                    let init_str = self.gen_expression(init);
                    self.output.push_str(&format!(
                        "{}{} {} = {};\n",
                        self.indent(),
                        ty_str,
                        name,
                        init_str
                    ));
                } else {
                    self.output.push_str(&format!(
                        "{}{} {};\n",
                        self.indent(),
                        ty_str,
                        name
                    ));
                }
                true
            }
            Statement::Assignment { target, value, .. } => {
                let target_str = self.gen_expression(target);
                let value_str = self.gen_expression(value);
                self.output.push_str(&format!(
                    "{}{} = {};\n",
                    self.indent(),
                    target_str,
                    value_str
                ));
                true
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    let val_str = self.gen_expression(v);
                    self.output.push_str(&format!("{}return {};\n", self.indent(), val_str));
                } else {
                    self.output.push_str(&format!("{}return;\n", self.indent()));
                }
                true
            }
            Statement::Expression(expr) => {
                let expr_str = self.gen_expression(expr);
                self.output.push_str(&format!("{}{};\n", self.indent(), expr_str));
                true
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond_str = self.gen_expression(condition);
                self.output.push_str(&format!("{}if ({}) {{\n", self.indent(), cond_str));
                self.indent_level += 1;
                self.gen_block_statements(&then_block.statements);
                self.indent_level -= 1;
                if let Some(eb) = else_block {
                    self.output.push_str(&format!("{}}} else {{\n", self.indent()));
                    self.indent_level += 1;
                    self.gen_block_statements(&eb.statements);
                    self.indent_level -= 1;
                }
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::Guard {
                condition,
                else_block,
                ..
            } => {
                let cond_str = self.gen_expression(condition);
                self.output.push_str(&format!("{}if (!({})) {{\n", self.indent(), cond_str));
                self.indent_level += 1;
                self.gen_block_statements(&else_block.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::While { condition, body, .. } => {
                let cond_str = self.gen_expression(condition);
                self.output.push_str(&format!("{}while ({}) {{\n", self.indent(), cond_str));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::ParallelFor {
                item_name,
                iterable,
                body,
                ..
            } => {
                let iter_str = self.gen_expression(iterable);
                self.output.push_str(&format!(
                    "{}#pragma omp parallel for\n",
                    self.indent()
                ));
                self.output.push_str(&format!(
                    "{}for (int32_t {} = 0; {} < {}; {}++) {{\n",
                    self.indent(),
                    item_name,
                    item_name,
                    iter_str,
                    item_name
                ));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::ForIn {
                item_name,
                iterable,
                body,
                ..
            } => {
                let iter_str = self.gen_expression(iterable);
                self.output.push_str(&format!(
                    "{}#pragma unroll\n{}#pragma GCC ivdep\n",
                    self.indent(),
                    self.indent()
                ));
                self.output.push_str(&format!(
                    "{}for (int64_t {} = 0; {} < {}; {}++) {{\n",
                    self.indent(),
                    item_name,
                    item_name,
                    iter_str,
                    item_name
                ));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::Match { expr, arms, .. } => {
                let expr_str = self.gen_expression(expr);
                self.output.push_str(&format!("{}switch (({}).tag) {{\n", self.indent(), expr_str));
                self.indent_level += 1;
                for arm in arms {
                    match &arm.pattern {
                        Pattern::Variant { variant_name, enum_name, binding } => {
                            let en = enum_name.clone().unwrap_or_else(|| self.find_enum_for_variant(variant_name));
                            self.output.push_str(&format!("{}case {}_{}: {{\n", self.indent(), en, variant_name));
                            self.indent_level += 1;
                            if let Some(b) = binding {
                                self.output.push_str(&format!("{}__auto_type {} = ({}).data.{};\n", self.indent(), b, expr_str, variant_name));
                            }
                            for s in &arm.body.statements {
                                self.gen_statement(s);
                            }
                            self.output.push_str(&format!("{}break;\n", self.indent()));
                            self.indent_level -= 1;
                            self.output.push_str(&format!("{}}}\n", self.indent()));
                        }
                        Pattern::Wildcard => {
                            self.output.push_str(&format!("{}default: {{\n", self.indent()));
                            self.indent_level += 1;
                            for s in &arm.body.statements {
                                self.gen_statement(s);
                            }
                            self.output.push_str(&format!("{}break;\n", self.indent()));
                            self.indent_level -= 1;
                            self.output.push_str(&format!("{}}}\n", self.indent()));
                        }
                        _ => {}
                    }
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            _ => false,
        }
    }
}
