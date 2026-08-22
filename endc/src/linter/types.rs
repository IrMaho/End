use crate::config::CompilerConfig;

#[derive(Debug, Clone)]
pub struct LintViolation {
    pub file: String,
    pub line: usize,
    pub rule: String,
    pub message: String,
    pub suggestion: String,
    pub severity: LintSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LintSeverity {
    Error,
    Warning,
}

pub struct Linter {
    pub(crate) config: CompilerConfig,
    pub(crate) violations: Vec<LintViolation>,
    pub(crate) file_path: String,
}
