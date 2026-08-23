use super::types::{LintSeverity, LintViolation, Linter};
use crate::ast::{Block, Statement};

impl Linter {
pub(crate) fn check_function_size(&mut self, name: &str, body: &Block, line: usize) {
        let body_lines = count_block_statements(body);
        if body_lines > self.config.files.max_function_lines {
            self.violations.push(LintViolation {
                file: self.file_path.clone(),
                line,
                rule: "files.max_function_lines".to_string(),
                message: format!(
                    "Function '{}' has {} statements, exceeding maximum of {}.",
                    name, body_lines, self.config.files.max_function_lines
                ),
                suggestion: format!(
                    "Break '{}' into smaller helper functions with ≤{} statements each.",
                    name, self.config.files.max_function_lines
                ),
                severity: LintSeverity::Error,
            });
        }
    }

    pub(crate) fn check_param_count(&mut self, name: &str, count: usize, line: usize) {
        if count > self.config.files.max_params {
            self.violations.push(LintViolation {
                file: self.file_path.clone(),
                line,
                rule: "files.max_params".to_string(),
                message: format!(
                    "Function '{}' has {} parameters, exceeding maximum of {}.",
                    name, count, self.config.files.max_params
                ),
                suggestion: format!(
                    "Group parameters of '{}' into a struct or config object.",
                    name
                ),
                severity: LintSeverity::Error,
            });
        }
    }


pub(crate) fn check_cyclomatic_complexity(&mut self, name: &str, body: &Block, line: usize) {
        let complexity = compute_cyclomatic_complexity(body);
        if complexity > self.config.quality.max_cyclomatic_complexity {
            self.violations.push(LintViolation {
                file: self.file_path.clone(),
                line,
                rule: "quality.max_cyclomatic_complexity".to_string(),
                message: format!(
                    "Function '{}' has cyclomatic complexity of {} (max: {}).",
                    name, complexity, self.config.quality.max_cyclomatic_complexity
                ),
                suggestion: format!(
                    "Refactor '{}' by extracting conditional branches into smaller functions.",
                    name
                ),
                severity: LintSeverity::Error,
            });
        }
    }

}

fn count_block_statements(block: &Block) -> usize {
    let mut count = block.statements.len();
    for stmt in &block.statements {
        count += count_nested_statements(stmt);
    }
    count
}

fn count_nested_statements(stmt: &Statement) -> usize {
    match stmt {
        Statement::If { then_block, else_block, .. } => {
            let mut c = then_block.statements.len();
            if let Some(else_b) = else_block {
                c += else_b.statements.len();
            }
            c
        }
        Statement::Guard { else_block, .. } => {
            else_block.statements.len()
        }
        Statement::While { body, .. } | Statement::ForIn { body, .. } | Statement::ParallelFor { body, .. } => {
            body.statements.len()
        }
        Statement::LeaseBlock { body, .. } | Statement::LeaseCpu { body, .. }
        | Statement::LeaseEvent { body, .. } | Statement::LeaseLoop { body, .. } => {
            body.statements.len()
        }
        _ => 0,
    }
}

fn compute_cyclomatic_complexity(block: &Block) -> usize {
    let mut complexity: usize = 1;
    for stmt in &block.statements {
        complexity += complexity_of_statement(stmt);
    }
    complexity
}

fn complexity_of_statement(stmt: &Statement) -> usize {
    match stmt {
        Statement::If { then_block, else_block, .. } => {
            let mut c: usize = 1;
            for s in &then_block.statements {
                c += complexity_of_statement(s);
            }
            if let Some(else_b) = else_block {
                c += 1;
                for s in &else_b.statements {
                    c += complexity_of_statement(s);
                }
            }
            c
        }
        Statement::Guard { else_block, .. } => {
            let mut c: usize = 1;
            for s in &else_block.statements {
                c += complexity_of_statement(s);
            }
            c
        }
        Statement::While { body, .. } => {
            let mut c: usize = 1;
            for s in &body.statements {
                c += complexity_of_statement(s);
            }
            c
        }
        Statement::ForIn { body, .. } | Statement::ParallelFor { body, .. } => {
            let mut c: usize = 1;
            for s in &body.statements {
                c += complexity_of_statement(s);
            }
            c
        }
        Statement::Match { arms, .. } => {
            let mut c: usize = 0;
            c += arms.len();
            for arm in arms {
                for s in &arm.body.statements {
                    c += complexity_of_statement(s);
                }
            }
            c
        }
        Statement::LeaseBlock { body, .. } | Statement::LeaseCpu { body, .. }
        | Statement::LeaseEvent { body, .. } | Statement::LeaseLoop { body, .. } => {
            let mut c: usize = 0;
            for s in &body.statements {
                c += complexity_of_statement(s);
            }
            c
        }
        _ => 0,
    }
}

