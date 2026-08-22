use serde::{Deserialize, Serialize};
use crate::ast::decl::structs_enums::{StructField, Directive};
use crate::ast::decl::functions_traits::FunctionDef;
use crate::ast::expr::Expression;
use crate::ast::pattern::Block;
use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassDef {
    pub name: String,
    pub is_pub: bool,
    pub is_abstract: bool,
    pub is_sealed: bool,
    pub is_open: bool,
    pub extends: Vec<String>,
    pub mixins: Vec<String>,
    pub implements: Vec<String>,
    pub shared_parents: Vec<String>,
    pub virtual_parents: Vec<String>,
    pub locked_contracts: Vec<String>,
    pub fields: Vec<StructField>,
    pub methods: Vec<FunctionDef>,
    pub directives: Vec<Directive>,
    pub span: Span,
}



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InheritKind {
    Standard,
    Surface(String),
    Shape(String),
    Behavior(String),
    Contract(String),
    Capabilities,
    Permissions,
    Events,
    Feature,
    Architecture,
    Policy,
    Lifecycle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InheritDef {
    pub target: String,
    pub parent: String,
    pub kind: InheritKind,
    pub alias: Option<String>,
    pub only: Vec<String>,
    pub except: Vec<String>,
    pub transforms: Vec<(String, String)>,
    pub mappings: Vec<(String, String)>,
    pub condition: Option<Expression>,
    pub is_contractual: bool,
    pub is_replaceable: bool,
    pub is_delegation: bool,
    pub capability_grants: Vec<String>,
    pub capability_denials: Vec<String>,
    pub permission_removals: Vec<String>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuperCall {
    pub target_parent: Option<String>,
    pub method: String,
    pub args: Vec<Expression>,
    pub is_superchain: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictDef {
    pub left: String,
    pub right: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionDef {
    pub preferred: String,
    pub over: Option<String>,
    pub is_merge: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InspectInheritanceDef {
    pub target: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactInheritanceDef {
    pub target: String,
    pub span: Span,
}
