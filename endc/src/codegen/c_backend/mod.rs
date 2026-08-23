pub mod expr_gen;
pub mod morphic_gen;
pub mod module_gen;
pub mod runtime;
pub mod state;
pub mod stmt_architecture_and_agents;
pub mod stmt_control_flow;
pub mod stmt_gen;
pub mod stmt_memory_and_regions;
pub mod stmt_ops_and_events;
pub mod stmt_sla_and_concurrency;
pub mod type_mapping;

pub use state::{escape_c_string, CBackend};

use super::backend_trait::{BackendError, CodeGenBackend};
use crate::ast::Module;

impl CodeGenBackend for CBackend {
    type Output = String;

    fn name(&self) -> &'static str {
        "c"
    }

    fn compile_module(&mut self, module: &Module) -> Result<Self::Output, BackendError> {
        Ok(self.generate(module))
    }
}
