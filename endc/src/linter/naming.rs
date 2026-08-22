use super::types::{LintSeverity, LintViolation, Linter};
use crate::ast::{Block, Expression, Statement};

impl Linter {
pub(crate) fn check_function_naming(&mut self, name: &str, line: usize) {
        match self.config.naming.function_style.as_str() {
            "snake_case" => {
                if !is_snake_case(name) {
                    self.violations.push(LintViolation {
                        file: self.file_path.clone(),
                        line,
                        rule: "naming.function_style".to_string(),
                        message: format!(
                            "Function '{}' violates naming convention: expected snake_case.",
                            name
                        ),
                        suggestion: format!("Rename to '{}'.", to_snake_case(name)),
                        severity: LintSeverity::Error,
                    });
                }
            }
            "camelCase" => {
                if !is_camel_case(name) {
                    self.violations.push(LintViolation {
                        file: self.file_path.clone(),
                        line,
                        rule: "naming.function_style".to_string(),
                        message: format!(
                            "Function '{}' violates naming convention: expected camelCase.",
                            name
                        ),
                        suggestion: format!("Rename to camelCase form."),
                        severity: LintSeverity::Error,
                    });
                }
            }
            _ => {}
        }
    }

    pub(crate) fn check_struct_naming(&mut self, name: &str, line: usize) {
        match self.config.naming.struct_style.as_str() {
            "PascalCase" => {
                if !is_pascal_case(name) {
                    self.violations.push(LintViolation {
                        file: self.file_path.clone(),
                        line,
                        rule: "naming.struct_style".to_string(),
                        message: format!(
                            "Struct '{}' violates naming convention: expected PascalCase.",
                            name
                        ),
                        suggestion: format!("Rename to '{}'.", to_pascal_case(name)),
                        severity: LintSeverity::Error,
                    });
                }
            }
            "snake_case" => {
                if !is_snake_case(name) {
                    self.violations.push(LintViolation {
                        file: self.file_path.clone(),
                        line,
                        rule: "naming.struct_style".to_string(),
                        message: format!(
                            "Struct '{}' violates naming convention: expected snake_case.",
                            name
                        ),
                        suggestion: format!("Rename to '{}'.", to_snake_case(name)),
                        severity: LintSeverity::Error,
                    });
                }
            }
            _ => {}
        }
    }

    pub(crate) fn check_variable_naming(&mut self, name: &str, line: usize) {
        match self.config.naming.variable_style.as_str() {
            "snake_case" => {
                if !is_snake_case(name) {
                    self.violations.push(LintViolation {
                        file: self.file_path.clone(),
                        line,
                        rule: "naming.variable_style".to_string(),
                        message: format!(
                            "Variable '{}' violates naming convention: expected snake_case.",
                            name
                        ),
                        suggestion: format!("Rename to '{}'.", to_snake_case(name)),
                        severity: LintSeverity::Error,
                    });
                }
            }
            "camelCase" => {
                if !is_camel_case(name) {
                    self.violations.push(LintViolation {
                        file: self.file_path.clone(),
                        line,
                        rule: "naming.variable_style".to_string(),
                        message: format!(
                            "Variable '{}' violates naming convention: expected camelCase.",
                            name
                        ),
                        suggestion: format!("Rename to camelCase form."),
                        severity: LintSeverity::Error,
                    });
                }
            }
            _ => {}
        }
    }

    pub(crate) fn check_variables_in_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            match stmt {
                Statement::VarDecl { name, span, .. } => {
                    self.check_variable_naming(name, span.line);
                }
                Statement::If { then_block, else_block, .. } => {
                    self.check_variables_in_block(then_block);
                    if let Some(else_b) = else_block {
                        self.check_variables_in_block(else_b);
                    }
                }
                Statement::Guard { else_block, .. } => {
                    self.check_variables_in_block(else_block);
                }
                Statement::While { body, .. }
                | Statement::ForIn { body, .. }
                | Statement::ParallelFor { body, .. }
                | Statement::RegionBlock { body, .. }
                | Statement::TargetBlock { body, .. } => {
                    self.check_variables_in_block(body);
                }
                Statement::Match { arms, .. } => {
                    for arm in arms {
                        self.check_variables_in_block(&arm.body);
                    }
                }
                Statement::LeaseBlock { name, body, span, .. } => {
                    self.check_variable_naming(name, span.line);
                    self.check_variables_in_block(body);
                }
                Statement::LeaseCpu { body, .. }
                | Statement::LeaseEvent { body, .. }
                | Statement::LeaseLoop { body, .. } => {
                    self.check_variables_in_block(body);
                }
                Statement::Owned { name, span, .. } => {
                    self.check_variable_naming(name, span.line);
                }
                Statement::Intent { body: Some(body), .. }
                | Statement::ProtectBlock { body, .. }
                | Statement::DeterministicBlock { body, .. }
                | Statement::ReplayBlock { body, .. }
                | Statement::TransactionBlock { body, .. }
                | Statement::SpeculativeBlock { body, .. }
                | Statement::FallbackBlock { body, .. }
                | Statement::CancelSafeBlock { body, .. }
                | Statement::TaskDecl { body, .. }
                | Statement::PatchDecl { body, .. }
                | Statement::RaceFreeBlock { body, .. }
                | Statement::DeadlineBlock { body, .. }
                | Statement::PriorityBlock { body, .. }
                | Statement::QualityBlock { body, .. }
                | Statement::TradeoffBlock { body, .. }
                | Statement::WatchBlock { handler: body, .. }
                | Statement::ReactBlock { handler: body, .. } => {
                    self.check_variables_in_block(body);
                }
                Statement::ComputeBlock { body, fallback, .. } => {
                    self.check_variables_in_block(body);
                    if let Some(fb) = fallback {
                        self.check_variables_in_block(fb);
                    }
                }
                Statement::BudgetBlock { body: Some(body), .. }
                | Statement::ContextBlock { body: Some(body), .. }
                | Statement::AgentContract { body: Some(body), .. }
                | Statement::EvolveBlock { body: Some(body), .. } => {
                    self.check_variables_in_block(body);
                }
                Statement::AdaptBlock { branches, .. } => {
                    for (_, blk) in branches {
                        self.check_variables_in_block(blk);
                    }
                }
                Statement::ParallelChoose { branches, .. } => {
                    for (_, blk) in branches {
                        self.check_variables_in_block(blk);
                    }
                }
                Statement::RaceBlock { branches, .. } => {
                    for blk in branches {
                        self.check_variables_in_block(blk);
                    }
                }
                Statement::HedgeBlock { primary, fallback, .. } => {
                    self.check_variables_in_block(primary);
                    self.check_variables_in_block(fallback);
                }
                _ => {}
            }
        }
    }


}

fn is_snake_case(name: &str) -> bool {
    if name.is_empty() { return true; }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() && first != '_' {
        return false;
    }
    name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_pascal_case(name: &str) -> bool {
    if name.is_empty() { return true; }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }
    !name.contains('_')
}

fn is_camel_case(name: &str) -> bool {
    if name.is_empty() { return true; }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return false;
    }
    !name.contains('_')
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_ascii_uppercase().to_string() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect()
}

