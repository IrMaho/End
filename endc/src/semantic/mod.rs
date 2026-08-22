pub mod analyzer;
pub mod capability_checker;
pub mod effects;
pub mod graph;
pub mod inheritance_checker;
pub mod smt_verifier;
pub mod tree_shaker;

pub use analyzer::SemanticAnalyzer;
pub use capability_checker::*;
pub use inheritance_checker::*;
pub use smt_verifier::{SmtFormalProver, SmtProofReport};
pub use tree_shaker::TreeShaker;
