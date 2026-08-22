use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::operators::Literal;
use crate::ast::stmt::Statement;
use crate::ast::expr::Expression;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variant {
        enum_name: Option<String>,
        variant_name: String,
        binding: Option<String>,
    },
    Literal(Literal),
    Ident(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Block,
    pub span: Span,
}

