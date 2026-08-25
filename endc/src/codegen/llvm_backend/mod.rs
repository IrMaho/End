pub mod expr_gen;
pub mod module_gen;
pub mod pipeline;
pub mod state;
pub mod stmt_gen;
pub mod verification_tests;

pub use state::{LlvmBackend, LlvmBuildArtifacts};

use crate::ast::Module;
use crate::codegen::backend_trait::{BackendError, CodeGenBackend};

impl CodeGenBackend for LlvmBackend {
    type Output = String;

    fn compile_module(&mut self, module: &Module) -> Result<Self::Output, BackendError> {
        self.generate_llvm_ir(module)
    }

    fn name(&self) -> &'static str {
        "llvm"
    }

    fn supports_jit(&self) -> bool {
        false
    }

    fn supports_aot(&self) -> bool {
        true
    }
}
