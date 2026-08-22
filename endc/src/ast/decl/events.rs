use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::pattern::Block;
use crate::ast::decl::structs_enums::{Directive, StructField};
use crate::ast::decl::functions_traits::{FunctionDef, FunctionParam};
use crate::ast::types::Type;
use crate::ast::expr::Expression;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExtensionBlock {
    pub target: String,
    pub is_struct: bool,
    pub is_augment: bool,
    pub trait_name: Option<String>,
    pub at_hook: Option<String>,
    pub required_capability: Option<String>,
    pub when_feature: Option<String>,
    pub generic_params: Vec<String>,
    pub version_req: Option<String>,
    pub owned_by: Option<String>,
    pub lifecycle: Option<String>,
    pub functions: Vec<FunctionDef>,
    pub overrides: Vec<FunctionDef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationDef {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub is_pub: bool,
    pub requires: Vec<String>,
    pub guarantees: Vec<String>,
    pub effects: Vec<String>,
    pub emits: Vec<String>,
    pub version: Option<usize>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDef {
    pub name: String,
    pub is_pub: bool,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventHandlerDef {
    pub event_name: String,
    pub handler_op: Option<Expression>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventHubDef {
    pub name: String,
    pub is_pub: bool,
    pub owns_events: Vec<String>,
    pub handlers: Vec<EventHandlerDef>,
    pub span: Span,
}

