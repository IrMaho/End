use super::state::CBackend;
use crate::ast::{BinaryOp, Expression, Literal, Type};

impl CBackend {
    pub(crate) fn map_type(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "bool".to_string(),
            Type::I8 => "int8_t".to_string(),
            Type::I16 => "int16_t".to_string(),
            Type::I32 => "int32_t".to_string(),
            Type::I64 => "int64_t".to_string(),
            Type::U8 => "uint8_t".to_string(),
            Type::U16 => "uint16_t".to_string(),
            Type::U32 => "uint32_t".to_string(),
            Type::U64 => "uint64_t".to_string(),
            Type::F32 => "float".to_string(),
            Type::F64 => "double".to_string(),
            Type::Simd(inner, lanes) => format!("{}_vec{}", self.map_type(inner), lanes),
            Type::Str => "const char*".to_string(),
            Type::Custom(name) => name.clone(),
            Type::Pointer(inner) => format!("{}*", self.map_type(inner)),
            Type::Slice(inner) => format!("{}*", self.map_type(inner)),
            Type::Array(inner, size) => format!("{}[{}]", self.map_type(inner), size),
            Type::Tuple(_) => "void*".to_string(),
            Type::Generic(name, _) => name.clone(),
            Type::Result(inner, _) => self.map_type(inner),
            Type::Box(inner) => format!("{}*", self.map_type(inner)),
            Type::Rc(inner) => format!("{}*", self.map_type(inner)),
            Type::Arc(inner) => format!("{}*", self.map_type(inner)),
            Type::Channel(_) => "EndChannel*".to_string(),
            Type::Region(_) => "EndArena*".to_string(),
            Type::Allocator => "EndAllocator*".to_string(),
            Type::Operation(_, _) => "EndOperation*".to_string(),
            Type::Event(name) => format!("EndEvent_{}", name),
            Type::OperationResult => "EndOperationResult*".to_string(),
            Type::Unknown => "void*".to_string(),
        }
    }

    pub fn infer_type(&self, expr: &Expression) -> Type {
        match expr {
            Expression::Lit(Literal::String(_), _) => Type::Str,
            Expression::Lit(Literal::Int(_), _) => Type::I64,
            Expression::Lit(Literal::Float(_), _) => Type::F64,
            Expression::Lit(Literal::Bool(_), _) => Type::Bool,
            Expression::Ident(name, _) => self.var_types.get(name).cloned().unwrap_or(Type::Void),
            Expression::StructInit { name, .. } => Type::Custom(name.clone()),
            Expression::EnumInit { enum_name, variant_name, .. } => {
                let en = enum_name.clone().unwrap_or_else(|| self.find_enum_for_variant(variant_name));
                Type::Custom(en)
            }
            Expression::Call { callee, .. } => {
                if let Expression::Ident(fn_name, _) = callee.as_ref() {
                    self.function_return_types.get(fn_name).cloned().unwrap_or(Type::Void)
                } else {
                    Type::Void
                }
            }
            Expression::FieldAccess { object, field, .. } => {
                let parent_ty = self.infer_type(object);
                if let Type::Custom(struct_name) = parent_ty {
                    if let Some(fields) = self.struct_fields.get(&struct_name) {
                        return fields.get(field).cloned().unwrap_or(Type::Void);
                    }
                }
                Type::Void
            }
            Expression::Unary { op, expr, .. } => {
                let inner_ty = self.infer_type(expr);
                if matches!(op, crate::ast::UnaryOp::AddressOf) {
                    Type::Pointer(Box::new(inner_ty))
                } else if matches!(op, crate::ast::UnaryOp::Deref) {
                    if let Type::Pointer(inner) = inner_ty {
                        *inner
                    } else {
                        Type::Void
                    }
                } else {
                    inner_ty
                }
            }
            Expression::Binary { op, left, right: _, .. } => {
                let l_ty = self.infer_type(left);
                if matches!(op, BinaryOp::Add) && l_ty == Type::Str { Type::Str } else { l_ty }
            }
            Expression::Cast { target_type, .. } => target_type.clone(),
            Expression::Alloc { target_type, .. } => Type::Pointer(Box::new(target_type.clone())),
            Expression::Conditional { then_branch, else_branch, .. } => {
                let t_ty = self.infer_type(then_branch);
                if t_ty != Type::Void {
                    t_ty
                } else {
                    self.infer_type(else_branch)
                }
            }
            Expression::Match { arms, .. } => {
                for arm in arms {
                    if let Some(last) = arm.body.statements.last() {
                        let ty = match last {
                            crate::ast::Statement::Expression(e) => self.infer_type(e),
                            crate::ast::Statement::Return { value: Some(e), .. } => self.infer_type(e),
                            _ => Type::Void,
                        };
                        if ty != Type::Void {
                            return ty;
                        }
                    }
                }
                Type::Void
            }
            Expression::Index { array, .. } => {
                let arr_ty = self.infer_type(array);
                match arr_ty {
                    Type::Array(inner, _) | Type::Slice(inner) | Type::Pointer(inner) => *inner,
                    _ => Type::Void,
                }
            }
            Expression::Walrus { expr, .. } => self.infer_type(expr),
            Expression::Catch { expr, .. } | Expression::Await { expr, .. } => self.infer_type(expr),
            _ => Type::Void,
        }
    }

    pub(crate) fn is_operation_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::OperationLiteral { .. }
            | Expression::Compose { .. }
            | Expression::Repeat { .. }
            | Expression::Parallel { .. }
            | Expression::Alternative { .. }
            | Expression::ConditionalOp { .. }
            | Expression::Memoize { .. } => true,
            Expression::Ident(name, _) => {
                name.contains("op") || name.contains("step") || name.contains("flow") || name.contains("pipeline")
            }
            _ => false,
        }
    }
}
