use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::decl::structs_enums::{Directive, StructDef};
use crate::ast::decl::functions_traits::FunctionDef;
use crate::ast::stmt::Statement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleFacets {
    pub api: Vec<FunctionDef>,
    pub implementation: Vec<FunctionDef>,
    pub tests: Vec<FunctionDef>,
    pub extension: Vec<FunctionDef>,
    pub architecture: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleContract {
    pub requires: Vec<String>,
    pub provides: Vec<String>,
    pub guarantees: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleDef {
    pub name: String,
    pub parent: Option<String>,
    pub is_pub: bool,
    pub is_partial: bool,
    pub is_evolvable: bool,
    pub responsibility: Option<String>,
    pub owns: Vec<String>,
    pub exposes: Vec<String>,
    pub depends: Vec<String>,
    pub depends_only: Option<Vec<String>>,
    pub forbid: Vec<String>,
    pub is_sealed: bool,
    pub purity: Option<String>,
    pub cohesion: Option<f64>,
    pub facets: Option<ModuleFacets>,
    pub contract: Option<ModuleContract>,
    pub overlay_target: Option<String>,
    pub skills: Vec<String>,
    pub structs: Vec<StructDef>,
    pub functions: Vec<FunctionDef>,
    pub overrides: Vec<FunctionDef>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

// ── 50 Super Revolutionary Feature-Oriented Paradigm AST Structures ──


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportStmt {
    pub kind: ImportKind,
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportKind {
    Standard,
    C(String),
    Zig(String),
    Rust(String),
    Go(String),
}

