// End Compiler: src/linter.rs
// Project-Level Linter Pass — Enforces end.config.toml Rules at Compile Time
// Validates: file size, function size, naming conventions, comment language,
//            cyclomatic complexity, doc comments, and param counts.

use crate::ast::{Module, Statement, Block, FunctionDef};
use crate::config::CompilerConfig;
use colored::*;
use std::path::Path;

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
    config: CompilerConfig,
    violations: Vec<LintViolation>,
    file_path: String,
}

impl Linter {
    pub fn new(config: CompilerConfig, file_path: &str) -> Self {
        Linter {
            config,
            violations: Vec::new(),
            file_path: file_path.to_string(),
        }
    }

    pub fn lint_source_and_ast(&mut self, source: &str, module: &Module) {
        self.check_file_lines(source);
        self.check_comments(source);
        self.lint_module(module);
    }

    pub fn violations(&self) -> &[LintViolation] {
        &self.violations
    }

    pub fn has_errors(&self) -> bool {
        self.violations.iter().any(|v| v.severity == LintSeverity::Error)
    }

    pub fn print_violations(&self) {
        if self.violations.is_empty() {
            return;
        }
        let file_basename = Path::new(&self.file_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| self.file_path.clone());

        println!("{}", "╔══════════════════════════════════════════════════════════════╗".bright_red());
        println!("{}", "║         ⚙️  END COMPILER CONFIG VIOLATIONS                  ║".bright_red());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_red());

        for v in &self.violations {
            let severity_icon = match v.severity {
                LintSeverity::Error => "❌ ERROR".bright_red(),
                LintSeverity::Warning => "⚠ WARN ".bright_yellow(),
            };
            println!(
                "  {} [{}] {}:{} — {}",
                severity_icon,
                v.rule.bright_cyan(),
                file_basename.bright_white(),
                v.line.to_string().bright_yellow(),
                v.message.white()
            );
            if !v.suggestion.is_empty() {
                println!(
                    "         {} {}",
                    "💡 Fix:".bright_green(),
                    v.suggestion.bright_green()
                );
            }
        }

        let errors = self.violations.iter().filter(|v| v.severity == LintSeverity::Error).count();
        let warnings = self.violations.iter().filter(|v| v.severity == LintSeverity::Warning).count();
        println!();
        println!(
            "  📊 Total: {} error(s), {} warning(s)",
            errors.to_string().bright_red(),
            warnings.to_string().bright_yellow()
        );
        println!();
    }

    /// Output lint violations in JSON format for machine consumption (IDE/CI integration).
    #[allow(dead_code)]
    pub fn print_violations_json(&self) {
        let violations_json: Vec<String> = self.violations.iter().map(|v| {
            format!(
                r#"{{"file":"{}","line":{},"rule":"{}","severity":"{}","message":"{}","suggestion":"{}"}}"#,
                v.file.replace('\\', "\\\\").replace('"', "\\\""),
                v.line,
                v.rule,
                match v.severity { LintSeverity::Error => "error", LintSeverity::Warning => "warning" },
                v.message.replace('"', "\\\""),
                v.suggestion.replace('"', "\\\""),
            )
        }).collect();
        println!("[{}]", violations_json.join(","));
    }

    // ── File-Level Checks ───────────────────────────────────────────

    fn check_file_lines(&mut self, source: &str) {
        let line_count = source.lines().count();
        if line_count > self.config.files.max_lines {
            self.violations.push(LintViolation {
                file: self.file_path.clone(),
                line: 0,
                rule: "files.max_lines".to_string(),
                message: format!(
                    "File has {} lines, exceeding maximum of {}.",
                    line_count, self.config.files.max_lines
                ),
                suggestion: format!(
                    "Split this file into smaller modules. Each module should have ≤{} lines.",
                    self.config.files.max_lines
                ),
                severity: LintSeverity::Error,
            });
        }
    }

    fn check_comments(&mut self, source: &str) {
        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Check single-line comments
            if let Some(comment_start) = trimmed.find("//") {
                let comment_body = trimmed[comment_start + 2..].trim();

                // If comments not allowed at all
                if !self.config.comments.allowed {
                    self.violations.push(LintViolation {
                        file: self.file_path.clone(),
                        line: i + 1,
                        rule: "comments.allowed".to_string(),
                        message: "Comments are disabled by project configuration.".to_string(),
                        suggestion: "Remove this comment or update end.config.toml [comments] allowed = true".to_string(),
                        severity: LintSeverity::Error,
                    });
                    continue;
                }

                // Check comment language
                if !comment_body.is_empty() {
                    match self.config.comments.language.as_str() {
                        "en" => {
                            if contains_persian(comment_body) {
                                self.violations.push(LintViolation {
                                    file: self.file_path.clone(),
                                    line: i + 1,
                                    rule: "comments.language".to_string(),
                                    message: "Comment contains Persian/Arabic characters, but config enforces language=\"en\".".to_string(),
                                    suggestion: "Rewrite comment in English or change [comments] language = \"any\".".to_string(),
                                    severity: LintSeverity::Error,
                                });
                            }
                        }
                        "fa" => {
                            if contains_english_words(comment_body) && !contains_persian(comment_body) {
                                self.violations.push(LintViolation {
                                    file: self.file_path.clone(),
                                    line: i + 1,
                                    rule: "comments.language".to_string(),
                                    message: "Comment is in English, but config enforces language=\"fa\".".to_string(),
                                    suggestion: "Rewrite comment in Persian/Farsi or change [comments] language = \"any\".".to_string(),
                                    severity: LintSeverity::Warning,
                                });
                            }
                        }
                        _ => {} // "any" — no restriction
                    }
                }
            }
        }
    }

    // ── AST-Level Checks ────────────────────────────────────────────

    fn lint_module(&mut self, module: &Module) {
        let fn_count = module.functions.len();

        for func in &module.functions {
            self.lint_function(func);
        }

        for st in &module.structs {
            self.check_struct_naming(&st.name, st.span.line);
        }

        // Check functions inside sub-modules
        for sub_mod in &module.modules {
            for func in &sub_mod.functions {
                self.lint_function(func);
            }
            for st in &sub_mod.structs {
                self.check_struct_naming(&st.name, st.span.line);
            }
        }

        // Check functions inside impl blocks
        for imp in &module.impls {
            for func in &imp.methods {
                self.lint_function(func);
            }
        }

        // Check max functions per file
        if fn_count > self.config.files.max_functions_per_file {
            self.violations.push(LintViolation {
                file: self.file_path.clone(),
                line: 0,
                rule: "files.max_functions_per_file".to_string(),
                message: format!(
                    "File contains {} functions, exceeding maximum of {}.",
                    fn_count, self.config.files.max_functions_per_file
                ),
                suggestion: "Split functions into separate modules for better modularity.".to_string(),
                severity: LintSeverity::Error,
            });
        }
    }

    fn lint_function(&mut self, func: &FunctionDef) {
        self.check_function_size(&func.name, &func.body, func.span.line);
        self.check_param_count(&func.name, func.params.len(), func.span.line);
        self.check_function_naming(&func.name, func.span.line);
        for param in &func.params {
            self.check_variable_naming(&param.name, param.span.line);
        }
        self.check_variables_in_block(&func.body);
        self.check_cyclomatic_complexity(&func.name, &func.body, func.span.line);

        if self.config.comments.require_doc_comments && func.is_pub && func.name != "main" {
            self.violations.push(LintViolation {
                file: self.file_path.clone(),
                line: func.span.line,
                rule: "comments.require_doc_comments".to_string(),
                message: format!("Public function '{}' should have a documentation comment.", func.name),
                suggestion: format!("Add a `// doc: ...` comment above function '{}'.", func.name),
                severity: LintSeverity::Warning,
            });
        }
    }

    fn check_function_size(&mut self, name: &str, body: &Block, line: usize) {
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

    fn check_param_count(&mut self, name: &str, count: usize, line: usize) {
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

    fn check_function_naming(&mut self, name: &str, line: usize) {
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

    fn check_struct_naming(&mut self, name: &str, line: usize) {
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

    fn check_variable_naming(&mut self, name: &str, line: usize) {
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

    fn check_variables_in_block(&mut self, block: &Block) {
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

    fn check_cyclomatic_complexity(&mut self, name: &str, body: &Block, line: usize) {
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

// ── Helpers ─────────────────────────────────────────────────────────

fn contains_persian(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        (0x0600..=0x06FF).contains(&cp) ||  // Arabic block (includes Persian)
        (0xFB50..=0xFDFF).contains(&cp) ||  // Arabic Presentation Forms A
        (0xFE70..=0xFEFF).contains(&cp)     // Arabic Presentation Forms B
    })
}

fn contains_english_words(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_alphabetic())
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
