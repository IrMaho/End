use super::types::{LintSeverity, LintViolation, Linter};
use crate::ast::{FunctionDef, Module};
use crate::config::CompilerConfig;

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


}
