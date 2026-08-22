use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::types::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Directive {
    pub name: String,
    pub args: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StructDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub is_partial: bool,
    pub is_sealed: bool,
    pub is_extension_only: bool,
    pub is_open: bool,
    pub is_closed: bool,
    pub friend_modules: Vec<String>,
    pub extension_points: Vec<String>,
    pub fields: Vec<StructField>,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub variants: Vec<EnumVariant>,
    pub directives: Vec<Directive>,
    pub span: Span,
}

