// ?? Lowering AST -> HIR -> MIR

use crate::ast::*;
use crate::ir::hir::*;
use crate::ir::mir::*;

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
            let params = f.params.iter().map(|p| (p.name.clone(), Self::lower_type(&p.param_type), p.is_mut)).collect();
            let return_type = Self::lower_type(&f.return_type);
            let is_pure = f.directives.iter().any(|d| d.name == "@pure");
            let is_async = f.directives.iter().any(|d| d.name == "@async");

            let mut body = Vec::new();
            for stmt in &f.body.statements {
                body.push(Self::lower_statement(stmt));
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

    fn lower_statement(stmt: &Statement) -> HirStatement {
        match stmt {
            Statement::VarDecl { name, var_type, is_mut, initializer, span } => {
                let ty = var_type.as_ref().map(Self::lower_type).unwrap_or(HirType::I64);
                let init = initializer.as_ref().map(Self::lower_expr);
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
                    value: Self::lower_expr(value),
                    line: span.line,
                }
            }
            Statement::Return { value, span } => {
                HirStatement::Return {
                    val: value.as_ref().map(Self::lower_expr),
                    line: span.line,
                }
            }
            Statement::Expression(e) => HirStatement::Expression(Self::lower_expr(e)),
            Statement::If { condition, then_block, else_block, span } => {
                let then_branch = then_block.statements.iter().map(Self::lower_statement).collect();
                let else_branch = else_block.as_ref().map(|eb| eb.statements.iter().map(Self::lower_statement).collect());
                HirStatement::If {
                    cond: Self::lower_expr(condition),
                    then_branch,
                    else_branch,
                    line: span.line,
                }
            }
            Statement::While { condition, body, span } => {
                let b = body.statements.iter().map(Self::lower_statement).collect();
                HirStatement::While {
                    cond: Self::lower_expr(condition),
                    body: b,
                    line: span.line,
                }
            }
            Statement::RegionBlock { name, body, span } => {
                let mut b = Vec::new();
                b.push(HirStatement::RegionEnter { name: name.clone(), line: span.line });
                for s in &body.statements {
                    b.push(Self::lower_statement(s));
                }
                b.push(HirStatement::RegionExit { name: name.clone(), line: span.line });
                HirStatement::Expression(HirExpression::LitBool(true))
            }
            _ => HirStatement::Expression(HirExpression::LitBool(true)),
        }
    }

    fn lower_expr(expr: &Expression) -> HirExpression {
        match expr {
            Expression::Lit(Literal::Int(n), _) => HirExpression::LitInt(*n, HirType::I64),
            Expression::Lit(Literal::Float(f), _) => HirExpression::LitFloat(*f, HirType::F64),
            Expression::Lit(Literal::String(s), _) => HirExpression::LitStr(s.clone()),
            Expression::Lit(Literal::Bool(b), _) => HirExpression::LitBool(*b),
            Expression::Ident(name, _) => HirExpression::Var(name.clone(), HirType::Custom("inferred".into())),
            Expression::Binary { left, op, right, .. } => {
                HirExpression::Binary {
                    op: format!("{:?}", op),
                    left: Box::new(Self::lower_expr(left)),
                    right: Box::new(Self::lower_expr(right)),
                    result_type: HirType::I64,
                }
            }
            Expression::Unary { expr, op, .. } => {
                HirExpression::Unary {
                    op: format!("{:?}", op),
                    expr: Box::new(Self::lower_expr(expr)),
                    result_type: HirType::I64,
                }
            }
            Expression::Call { callee, args, .. } => {
                let c_name = match callee.as_ref() {
                    Expression::Ident(n, _) => n.clone(),
                    _ => "callee".to_string(),
                };
                let h_args = args.iter().map(Self::lower_expr).collect();
                HirExpression::Call {
                    callee: c_name,
                    args: h_args,
                    result_type: HirType::Void,
                }
            }
            Expression::Alloc { target_type, allocator, .. } => {
                let el_ty = Self::lower_type(target_type);
                HirExpression::Alloc {
                    element_type: el_ty,
                    count: Box::new(Self::lower_expr(allocator)),
                    region_name: None,
                    result_type: HirType::Pointer(Box::new(HirType::Void)),
                }
            }
            _ => HirExpression::LitBool(true),
        }
    }
}

