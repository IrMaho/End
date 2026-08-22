use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::pattern::Block;
use crate::ast::decl::structs_enums::{Directive, StructField};
use crate::ast::decl::functions_traits::{FunctionDef, FunctionParam};
use crate::ast::types::Type;
use crate::ast::expr::Expression;
use crate::ast::stmt::Statement;

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
pub enum EventChannelKind {
    SingleDirection, // A -> B
    Duplex,          // Client <-> Server
    HalfDuplex,      // Sensor <~> Hub
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDef {
    pub name: String,
    pub is_pub: bool,
    pub generic_params: Vec<String>,
    pub parent_event: Option<String>,
    pub channel_kind: Option<EventChannelKind>,
    pub channel_target: Option<String>,
    pub with_attributes: Vec<String>,
    pub fields: Vec<StructField>,
    pub directives: Vec<Directive>,
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
    pub routes: Vec<(String, String)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnEventDef {
    pub event_pattern: String,
    pub guard: Option<Expression>,
    pub filter: Option<Expression>,
    pub projection: Option<String>,
    pub body: Block,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnceEventDef {
    pub event_pattern: String,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EveryEventDef {
    pub interval_str: String,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AfterEventDef {
    pub delay_str: String,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeforeEventDef {
    pub event_pattern: String,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactiveStateDef {
    pub name: String,
    pub val_type: Option<Type>,
    pub initial_val: Expression,
    pub with_attributes: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeriveDef {
    pub target_var: String,
    pub source_vars: Vec<String>,
    pub expr: Expression,
    pub with_attributes: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyDef {
    pub name: String,
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventStreamOpDef {
    pub op_kind: String,
    pub target: String,
    pub params: Vec<String>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventTransactionDef {
    pub statements: Vec<Statement>,
    pub on_rollback: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventFormalInvariantDef {
    pub name: String,
    pub rule: String,
    pub is_temporal: bool,
    pub timeout_str: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEvolveDef {
    pub from_event: String,
    pub to_event: String,
    pub migration_fn: Option<String>,
    pub compatibility: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventControlDef {
    pub action: String,
    pub target: String,
    pub args: Vec<String>,
    pub span: Span,
}
