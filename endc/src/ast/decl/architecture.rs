use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::decl::functions_traits::TraitMethodDef;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ContractDef {
    pub name: String,
    pub methods: Vec<TraitMethodDef>,
    pub clauses: Vec<String>,
    pub is_evolved: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ArchitectureTemplateDef {
    pub name: String,
    pub required_layers: Vec<String>,
    pub rules: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ArchitectureRuleDef {
    pub name: String,
    pub allowed_flows: Vec<(String, String)>,
    pub forbidden_flows: Vec<(String, String)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureMigrationDef {
    pub feature_name: String,
    pub from_version: String,
    pub to_version: String,
    pub renames: Vec<(String, String)>,
    pub replacements: Vec<(String, String)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BlastRadiusReport {
    pub target_symbol: String,
    pub affected_features: Vec<String>,
    pub affected_modules: Vec<String>,
    pub affected_symbols: Vec<String>,
    pub affected_public_apis: Vec<String>,
    pub required_migrations: Vec<String>,
}


