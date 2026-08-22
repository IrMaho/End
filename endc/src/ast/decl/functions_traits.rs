use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::types::Type;
use crate::ast::pattern::Block;
use crate::ast::decl::structs_enums::Directive;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: Type,
    pub is_mut: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub body: Block,
    pub directives: Vec<Directive>,
    pub morphic_param: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethodDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub methods: Vec<TraitMethodDef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub target_type: Type,
    pub methods: Vec<FunctionDef>,
    pub span: Span,
}

