use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::decl::structs_enums::{EnumDef, StructDef};
use crate::ast::decl::functions_traits::{FunctionDef, TraitDef};
use crate::ast::expr::Expression;
use crate::ast::stmt::Statement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureApi {
    pub functions: Vec<FunctionDef>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub traits: Vec<TraitDef>,
    pub exposed_symbols: Vec<String>,
    pub raw_signatures: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureImpl {
    pub name: Option<String>,
    pub target_contract: Option<String>,
    pub functions: Vec<FunctionDef>,
    pub structs: Vec<StructDef>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureContractClause {
    pub rule: String,
    pub is_negative: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureDependency {
    pub name: String,
    pub sub_contract: Option<String>, // e.g. "api" in "Authentication.api"
    pub type_params: Vec<String>,     // e.g. ["Transactional"]
    pub why: Option<String>,          // e.g. "Payment signatures require cryptographic verification"
    pub is_typed: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureExtensionPoint {
    pub name: String,
    pub allowed_types: Vec<String>,
    pub priority: Option<i64>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureBoundary {
    pub layers: Vec<String>, // ["api", "domain", "infrastructure"]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeaturePermission {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureLifecycle {
    pub state: String, // "experimental", "stable", "deprecated"
    pub replace_with: Option<String>,
    pub migration_path: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureDecision {
    pub target: String,
    pub reason: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureDef {
    pub name: String,
    pub version: Option<String>,
    pub owner: Option<String>,
    pub parent: Option<String>,
    pub architecture_template: Option<String>,
    pub is_pub: bool,
    pub is_replaceable: bool,
    pub is_evolvable: bool,
    pub api: Option<FeatureApi>,
    pub implementations: Vec<FeatureImpl>,
    pub needs: Vec<FeatureDependency>,
    pub boundary: Option<FeatureBoundary>,
    pub exposes: Vec<String>,
    pub extensions: Vec<FeatureExtensionPoint>,
    pub compose: Vec<String>,
    pub contracts: Vec<FeatureContractClause>,
    pub invariants: Vec<Expression>,
    pub tests: Vec<FunctionDef>,
    pub requires_capabilities: Vec<String>,
    pub permissions: Option<FeaturePermission>,
    pub lifecycle: Option<FeatureLifecycle>,
    pub decisions: Vec<FeatureDecision>,
    pub nested_features: Vec<FeatureDef>,
    pub forbids: Vec<String>,
    pub allows: Vec<String>,
    pub decorations: Vec<String>,
    pub span: Span,
}

