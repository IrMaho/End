use serde::{Deserialize, Serialize};
use crate::ast::Span;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefactorSessionDef {
    pub agent_name: String,
    pub target: String,
    pub scope: Vec<String>,
    pub forbid: Vec<String>,
    pub goals: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmodulePlan {
    pub name: String,
    pub role: String,
    pub symbols: Vec<String>,
    pub max_loc: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecompositionPlanDef {
    pub source: String,
    pub target_architecture: String,
    pub submodules: Vec<SubmodulePlan>,
    pub facade_name: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConservationAuditDef {
    pub original_source: String,
    pub original_loc: usize,
    pub original_symbols: Vec<String>,
    pub new_loc: usize,
    pub accounted_symbols: Vec<String>,
    pub unaccounted_count: usize,
    pub allow_semantic_deletion: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolidAuditDef {
    pub module_name: String,
    pub verify_srp: bool,
    pub verify_ocp: bool,
    pub verify_lsp: bool,
    pub verify_isp: bool,
    pub verify_dip: bool,
    pub max_responsibilities: usize,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefactoringTxDef {
    pub tx_name: String,
    pub checkpoint: String,
    pub steps: Vec<String>,
    pub auto_rollback: bool,
    pub run_test_gate: bool,
    pub run_build_gate: bool,
    pub max_lines_limit: usize,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolInventoryDef {
    pub module_name: String,
    pub classes: Vec<String>,
    pub functions: Vec<String>,
    pub types: Vec<String>,
    pub public_exports: Vec<String>,
    pub internal_symbols: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceableMapDef {
    pub source_module: String,
    pub mappings: Vec<(String, String)>,
    pub span: Span,
}
