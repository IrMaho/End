use crate::ast::span::Span;
use super::expression::Expression;

impl Expression {
    pub fn span(&self) -> &Span {
        match self {
            Expression::Lit(_, s) => s,
            Expression::Ident(_, s) => s,
            Expression::Binary { span, .. } => span,
            Expression::Unary { span, .. } => span,
            Expression::Call { span, .. } => span,
            Expression::FieldAccess { span, .. } => span,
            Expression::Index { span, .. } => span,
            Expression::StructInit { span, .. } => span,
            Expression::EnumInit { span, .. } => span,
            Expression::Alloc { span, .. } => span,
            Expression::Promote { span, .. } => span,
            Expression::Catch { span, .. } => span,
            Expression::Match { span, .. } => span,
            Expression::Block(b) => &b.span,
            Expression::NameOf { span, .. } => span,
            Expression::PathOf { span, .. } => span,
            Expression::TypeOf { span, .. } => span,
            Expression::DocOf { span, .. } => span,
            Expression::CodeOf { span, .. } => span,
            Expression::Dbg { span, .. } => span,
            Expression::AssertDebug { span, .. } => span,
            Expression::Translate { span, .. } => span,
            Expression::FieldsOf { span, .. } => span,
            Expression::Pipe { span, .. } => span,
            Expression::InlineC { span, .. } => span,
            Expression::SqlExpr { span, .. } => span,
            Expression::Cast { span, .. } => span,
            Expression::Await { span, .. } => span,
            Expression::UnitLit { span, .. } => span,
            Expression::NullCollapse { span, .. } => span,
            Expression::OperationLiteral { span, .. } => span,
            Expression::Compose { span, .. } => span,
            Expression::Repeat { span, .. } => span,
            Expression::Parallel { span, .. } => span,
            Expression::Alternative { span, .. } => span,
            Expression::ConditionalOp { span, .. } => span,
            Expression::Memoize { span, .. } => span,
        }
    }
}

