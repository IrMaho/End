use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::decl::structs_enums::{Directive, EnumDef, StructDef};
use crate::ast::decl::functions_traits::{FunctionDef, ImplBlock, TraitDef};
use crate::ast::decl::modules::{ImportStmt, ModuleDef};
use crate::ast::decl::features::FeatureDef;
use crate::ast::decl::architecture::{ArchitectureRuleDef, ArchitectureTemplateDef, ContractDef, FeatureMigrationDef};
use crate::ast::decl::events::{EventDef, EventHandlerDef, EventHubDef, ExtensionBlock, OperationDef};
use crate::ast::stmt::Statement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub imports: Vec<ImportStmt>,
    pub enums: Vec<EnumDef>,
    pub structs: Vec<StructDef>,
    pub traits: Vec<TraitDef>,
    pub impls: Vec<ImplBlock>,
    pub functions: Vec<FunctionDef>,
    pub modules: Vec<ModuleDef>,
    pub extensions: Vec<ExtensionBlock>,
    pub features: Vec<FeatureDef>,
    pub contracts: Vec<ContractDef>,
    pub architecture_templates: Vec<ArchitectureTemplateDef>,
    pub architecture_rules: Vec<ArchitectureRuleDef>,
    pub feature_migrations: Vec<FeatureMigrationDef>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

impl Module {
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            imports: Vec::new(),
            enums: Vec::new(),
            structs: Vec::new(),
            traits: Vec::new(),
            impls: Vec::new(),
            functions: Vec::new(),
            modules: Vec::new(),
            extensions: Vec::new(),
            features: Vec::new(),
            contracts: Vec::new(),
            architecture_templates: Vec::new(),
            architecture_rules: Vec::new(),
            feature_migrations: Vec::new(),
            statements: Vec::new(),
            span: Span::default(),
        }
    }
}




