// ?? Semantic AST -> Typed HIR Lowering

use crate::ast::*;
use crate::ir::hir::*;
use std::collections::HashMap;

pub struct AstLowering;

impl AstLowering {
    pub fn lower_type(ty: &Type) -> HirType {
        match ty {
            Type::Void => HirType::Void,
            Type::Bool => HirType::Bool,
            Type::I8 => HirType::I8,
            Type::I16 => HirType::I16,
            Type::I32 => HirType::I32,
            Type::I64 => HirType::I64,
            Type::U8 => HirType::U8,
            Type::U16 => HirType::U16,
            Type::U32 => HirType::U32,
            Type::U64 => HirType::U64,
            Type::F32 => HirType::F32,
            Type::F64 => HirType::F64,
            Type::Str => HirType::Str,
            Type::Pointer(t) => HirType::Pointer(Box::new(Self::lower_type(t))),
            Type::Region(r) => HirType::RegionPointer(Box::new(HirType::Void), r.clone()),
            Type::Array(t, sz) => HirType::Array(Box::new(Self::lower_type(t)), *sz),
            Type::Generic(n, args) => {
                let h_args = args.iter().map(Self::lower_type).collect();
                HirType::Generic(n.clone(), h_args)
            }
            Type::Custom(n) => HirType::Custom(n.clone()),
            _ => HirType::Custom(format!("{:?}", ty)),
        }
    }

    pub fn lower_module(module: &Module) -> HirModule {
        let mut structs = Vec::new();
        for s in &module.structs {
            let fields = s.fields.iter().map(|f| (f.name.clone(), Self::lower_type(&f.field_type))).collect();
            structs.push(HirStruct {
                name: s.name.clone(),
                fields,
            });
        }

        let mut functions = Vec::new();
        for f in &module.functions {
            let mut var_ctx = HashMap::new();
            let mut params = Vec::new();

            for p in &f.params {
                let h_ty = Self::lower_type(&p.param_type);
                var_ctx.insert(p.name.clone(), h_ty.clone());
                params.push((p.name.clone(), h_ty, p.is_mut));
            }

            let return_type = Self::lower_type(&f.return_type);
            let is_pure = f.directives.iter().any(|d| d.name == "@pure");
            let is_async = f.directives.iter().any(|d| d.name == "@async");

            let mut body = Vec::new();
            for stmt in &f.body.statements {
                body.push(Self::lower_statement(stmt, &mut var_ctx));
            }

            functions.push(HirFunction {
                name: f.name.clone(),
                params,
                return_type,
                body,
                is_pure,
                is_async,
            });
        }

        HirModule {
            name: module.name.clone(),
            structs,
            functions,
        }
    }

    fn lower_statement(stmt: &Statement, var_ctx: &mut HashMap<String, HirType>) -> HirStatement {
        match stmt {
            Statement::VarDecl { name, var_type, is_mut, is_lease: _, initializer, span } => {
                let init = initializer.as_ref().map(|i| Self::lower_expr(i, var_ctx));
                let ty = if let Some(v_ty) = var_type {
                    Self::lower_type(v_ty)
                } else if let Some(ref in_expr) = init {
                    in_expr.get_type()
                } else {
                    HirType::I64
                };

                var_ctx.insert(name.clone(), ty.clone());

                HirStatement::VarDecl {
                    name: name.clone(),
                    ty,
                    is_mut: *is_mut,
                    init,
                    line: span.line,
                }
            }
            Statement::Assignment { target, value, span } => {
                let name = match target {
                    Expression::Ident(n, _) => n.clone(),
                    _ => "target".to_string(),
                };
                HirStatement::Assign {
                    target: name,
                    value: Self::lower_expr(value, var_ctx),
                    line: span.line,
                }
            }
            Statement::Return { value, span } => {
                HirStatement::Return {
                    val: value.as_ref().map(|v| Self::lower_expr(v, var_ctx)),
                    line: span.line,
                }
            }
            Statement::Expression(e) => HirStatement::Expression(Self::lower_expr(e, var_ctx)),
            Statement::If { condition, then_block, else_block, span } => {
                let mut then_ctx = var_ctx.clone();
                let then_branch = then_block.statements.iter().map(|s| Self::lower_statement(s, &mut then_ctx)).collect();
                let else_branch = else_block.as_ref().map(|eb| {
                    let mut else_ctx = var_ctx.clone();
                    eb.statements.iter().map(|s| Self::lower_statement(s, &mut else_ctx)).collect()
                });
                HirStatement::If {
                    cond: Self::lower_expr(condition, var_ctx),
                    then_branch,
                    else_branch,
                    line: span.line,
                }
            }
            Statement::Guard { condition, else_block, span } => {
                let mut else_ctx = var_ctx.clone();
                let else_branch = else_block.statements.iter().map(|s| Self::lower_statement(s, &mut else_ctx)).collect();
                let lowered_cond = Self::lower_expr(condition, var_ctx);
                let negated_cond = HirExpression::Unary {
                    op: "!".to_string(),
                    expr: Box::new(lowered_cond),
                    result_type: HirType::Bool,
                };
                HirStatement::If {
                    cond: negated_cond,
                    then_branch: else_branch,
                    else_branch: None,
                    line: span.line,
                }
            }
            Statement::While { condition, body, span } => {
                let mut body_ctx = var_ctx.clone();
                let b = body.statements.iter().map(|s| Self::lower_statement(s, &mut body_ctx)).collect();
                HirStatement::While {
                    cond: Self::lower_expr(condition, var_ctx),
                    body: b,
                    line: span.line,
                }
            }
            Statement::RegionBlock { name, body, span } => {
                let mut reg_ctx = var_ctx.clone();
                let inner_stmts = body.statements.iter().map(|s| Self::lower_statement(s, &mut reg_ctx)).collect();
                HirStatement::RegionBlock {
                    name: name.clone(),
                    body: inner_stmts,
                    line: span.line,
                }
            }
            Statement::Defer { expr, span } => {
                HirStatement::Defer {
                    expr: Self::lower_expr(expr, var_ctx),
                    line: span.line,
                }
            }
            Statement::Spawn { call, span } => {
                HirStatement::Spawn {
                    call: Self::lower_expr(call, var_ctx),
                    line: span.line,
                }
            }
            Statement::Owned { name, var_type, initializer, span } => {
                let init = Some(Self::lower_expr(initializer, var_ctx));
                let ty = if let Some(v_ty) = var_type {
                    Self::lower_type(v_ty)
                } else if let Some(ref in_expr) = init {
                    in_expr.get_type()
                } else {
                    HirType::I64
                };
                var_ctx.insert(name.clone(), ty.clone());
                HirStatement::VarDecl {
                    name: name.clone(),
                    ty,
                    is_mut: false,
                    init,
                    line: span.line,
                }
            }
            _ => HirStatement::Expression(HirExpression::LitBool(true)),
        }
    }

    fn lower_expr(expr: &Expression, var_ctx: &HashMap<String, HirType>) -> HirExpression {
        match expr {
            Expression::Lit(Literal::Int(n), _) => HirExpression::LitInt(*n, HirType::I64),
            Expression::Lit(Literal::Float(f), _) => HirExpression::LitFloat(*f, HirType::F64),
            Expression::Lit(Literal::String(s), _) => HirExpression::LitStr(s.clone()),
            Expression::Lit(Literal::Bool(b), _) => HirExpression::LitBool(*b),
            Expression::Ident(name, _) => {
                let ty = var_ctx.get(name).cloned().unwrap_or(HirType::Custom(name.clone()));
                HirExpression::Var(name.clone(), ty)
            }
            Expression::Binary { left, op, right, .. } => {
                let l_lowered = Box::new(Self::lower_expr(left, var_ctx));
                let r_lowered = Box::new(Self::lower_expr(right, var_ctx));
                let res_type = match op {
                    BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::LessThan | BinaryOp::LessEqual | BinaryOp::GreaterThan | BinaryOp::GreaterEqual | BinaryOp::And | BinaryOp::Or => HirType::Bool,
                    _ => {
                        if l_lowered.get_type() == HirType::F64 || r_lowered.get_type() == HirType::F64 {
                            HirType::F64
                        } else {
                            HirType::I64
                        }
                    }
                };
                HirExpression::Binary {
                    op: format!("{:?}", op),
                    left: l_lowered,
                    right: r_lowered,
                    result_type: res_type,
                }
            }
            Expression::Unary { expr, op, .. } => {
                let inner = Box::new(Self::lower_expr(expr, var_ctx));
                let res_type = match op {
                    UnaryOp::Not => HirType::Bool,
                    UnaryOp::AddressOf => HirType::Pointer(Box::new(inner.get_type())),
                    _ => inner.get_type(),
                };
                HirExpression::Unary {
                    op: format!("{:?}", op),
                    expr: inner,
                    result_type: res_type,
                }
            }
            Expression::Call { callee, args, .. } => {
                let c_name = match callee.as_ref() {
                    Expression::Ident(n, _) => n.clone(),
                    _ => "callee".to_string(),
                };
                let h_args = args.iter().map(|a| Self::lower_expr(a, var_ctx)).collect();
                HirExpression::Call {
                    callee: c_name,
                    args: h_args,
                    result_type: HirType::Void,
                }
            }
            Expression::Alloc { target_type, allocator, .. } => {
                let el_ty = Self::lower_type(target_type);
                HirExpression::Alloc {
                    element_type: el_ty.clone(),
                    count: Box::new(Self::lower_expr(allocator, var_ctx)),
                    region_name: None,
                    result_type: HirType::Pointer(Box::new(el_ty)),
                }
            }
            _ => HirExpression::LitBool(true),
        }
    }
}
