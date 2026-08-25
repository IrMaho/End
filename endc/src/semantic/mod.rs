pub mod analyzer;
pub mod capability_checker;
pub mod effects;
pub mod graph;
pub mod inheritance_checker;
pub mod refactoring_analyzer;
pub mod smt_encode;
pub mod smt_solver;
pub mod smt_verifier;
pub mod tree_shaker;

#[cfg(test)]
pub mod smt_tests;

pub use analyzer::SemanticAnalyzer;
pub use capability_checker::*;
pub use inheritance_checker::*;
pub use refactoring_analyzer::*;
pub use smt_encode::{SmtEncoder, SmtExpr, SmtType, UnknownReason};
pub use smt_solver::{RawSolverResult, SmtSolverEngine};
pub use smt_verifier::{
    Obligation, ObligationKind, ObligationResult, ProofResult, SmtFormalProver, SmtProofReport,
};
pub use tree_shaker::TreeShaker;
