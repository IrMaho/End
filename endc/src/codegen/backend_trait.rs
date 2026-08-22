use crate::ast::Module;

#[derive(Debug, Clone)]
pub enum BackendError {
    UnsupportedFeature(String),
    CodegenFailed(String),
    LinkerFailed(String),
    TypeMismatch(String),
    Internal(String),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::UnsupportedFeature(msg) => write!(f, "Unsupported Feature: {}", msg),
            BackendError::CodegenFailed(msg) => write!(f, "Codegen Failed: {}", msg),
            BackendError::LinkerFailed(msg) => write!(f, "Linker Failed: {}", msg),
            BackendError::TypeMismatch(msg) => write!(f, "Type Mismatch: {}", msg),
            BackendError::Internal(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

pub trait CodeGenBackend {
    type Output;
    
    fn compile_module(&mut self, module: &Module) -> Result<Self::Output, BackendError>;
    fn name(&self) -> &'static str;
    fn supports_jit(&self) -> bool { false }
    fn supports_aot(&self) -> bool { true }
}
