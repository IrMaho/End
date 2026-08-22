use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::pattern::Block;
use crate::ast::decl::functions_traits::FunctionDef;
use crate::ast::stmt::Statement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDef {
    pub name: String,
    pub methods: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeDef {
    pub entity: String,
    pub name: String,
    pub fields: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurfaceDef {
    pub entity: String,
    pub name: String,
    pub condition: Option<String>,
    pub symbols: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixinDef {
    pub name: String,
    pub methods: Vec<FunctionDef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopeDef {
    pub name: String,
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextDef {
    pub environment: String,
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterceptDef {
    pub entity: String,
    pub method: String,
    pub before_block: Option<Block>,
    pub after_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HookDef {
    pub entity: String,
    pub event_point: String,
    pub body: Block,
    pub span: Span,
}
