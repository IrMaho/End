// End Compiler Diagnostics Architecture
// Defines structured compiler diagnostics, severity levels, source spans,
// error codes (E001-E020), and diagnostic accumulation.

use colored::*;
use serde::{Deserialize, Serialize};

/// Reserved error codes for the End compiler pipeline
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagCode {
    #[serde(rename = "E001")]
    E001_UNSUPPORTED_EXPRESSION,
    #[serde(rename = "E002")]
    E002_TYPE_MISMATCH,
    #[serde(rename = "E003")]
    E003_UNDEFINED_NAME,
    #[serde(rename = "E004")]
    E004_INVALID_C_GENERATED,
    #[serde(rename = "E005")]
    E005_PARSE_FAILURE,
    #[serde(rename = "E006")]
    E006_SEMANTIC_FAILURE,
    #[serde(rename = "E007")]
    E007_CODEGEN_FAILURE,
    #[serde(rename = "E008")]
    E008_RANGE_BOUND_INVALID,
    #[serde(rename = "E009")]
    E009_ENUM_VARIANT_INVALID,
    #[serde(rename = "E010")]
    E010_PATTERN_UNHANDLED,
    #[serde(rename = "E011")]
    E011_CLOSURE_CAPTURE_INVALID,
    #[serde(rename = "E012")]
    E012_GENERICS_UNSUPPORTED,
    #[serde(rename = "E013")]
    E013_TRAIT_RESOLUTION_FAILURE,
    #[serde(rename = "E014")]
    E014_MODULE_NOT_FOUND,
    #[serde(rename = "E015")]
    E015_IMPORT_CYCLE,
    #[serde(rename = "E016")]
    E016_INVALID_LVALUE,
    #[serde(rename = "E017")]
    E017_MEMORY_OWNERSHIP_VIOLATION,
    #[serde(rename = "E018")]
    E018_BORROW_CHECK_FAILURE,
    #[serde(rename = "E019")]
    E019_CONCURRENCY_VIOLATION,
    #[serde(rename = "E020")]
    E020_ATTRIBUTE_INVALID,
    #[serde(untagged)]
    Custom(String),
}

impl DiagCode {
    /// Return the standardized short code (e.g. "E001")
    pub fn as_code_str(&self) -> &str {
        match self {
            DiagCode::E001_UNSUPPORTED_EXPRESSION => "E001",
            DiagCode::E002_TYPE_MISMATCH => "E002",
            DiagCode::E003_UNDEFINED_NAME => "E003",
            DiagCode::E004_INVALID_C_GENERATED => "E004",
            DiagCode::E005_PARSE_FAILURE => "E005",
            DiagCode::E006_SEMANTIC_FAILURE => "E006",
            DiagCode::E007_CODEGEN_FAILURE => "E007",
            DiagCode::E008_RANGE_BOUND_INVALID => "E008",
            DiagCode::E009_ENUM_VARIANT_INVALID => "E009",
            DiagCode::E010_PATTERN_UNHANDLED => "E010",
            DiagCode::E011_CLOSURE_CAPTURE_INVALID => "E011",
            DiagCode::E012_GENERICS_UNSUPPORTED => "E012",
            DiagCode::E013_TRAIT_RESOLUTION_FAILURE => "E013",
            DiagCode::E014_MODULE_NOT_FOUND => "E014",
            DiagCode::E015_IMPORT_CYCLE => "E015",
            DiagCode::E016_INVALID_LVALUE => "E016",
            DiagCode::E017_MEMORY_OWNERSHIP_VIOLATION => "E017",
            DiagCode::E018_BORROW_CHECK_FAILURE => "E018",
            DiagCode::E019_CONCURRENCY_VIOLATION => "E019",
            DiagCode::E020_ATTRIBUTE_INVALID => "E020",
            DiagCode::Custom(s) => s.as_str(),
        }
    }

    /// Return the full identifier name
    pub fn name(&self) -> &str {
        match self {
            DiagCode::E001_UNSUPPORTED_EXPRESSION => "E001_UNSUPPORTED_EXPRESSION",
            DiagCode::E002_TYPE_MISMATCH => "E002_TYPE_MISMATCH",
            DiagCode::E003_UNDEFINED_NAME => "E003_UNDEFINED_NAME",
            DiagCode::E004_INVALID_C_GENERATED => "E004_INVALID_C_GENERATED",
            DiagCode::E005_PARSE_FAILURE => "E005_PARSE_FAILURE",
            DiagCode::E006_SEMANTIC_FAILURE => "E006_SEMANTIC_FAILURE",
            DiagCode::E007_CODEGEN_FAILURE => "E007_CODEGEN_FAILURE",
            DiagCode::E008_RANGE_BOUND_INVALID => "E008_RANGE_BOUND_INVALID",
            DiagCode::E009_ENUM_VARIANT_INVALID => "E009_ENUM_VARIANT_INVALID",
            DiagCode::E010_PATTERN_UNHANDLED => "E010_PATTERN_UNHANDLED",
            DiagCode::E011_CLOSURE_CAPTURE_INVALID => "E011_CLOSURE_CAPTURE_INVALID",
            DiagCode::E012_GENERICS_UNSUPPORTED => "E012_GENERICS_UNSUPPORTED",
            DiagCode::E013_TRAIT_RESOLUTION_FAILURE => "E013_TRAIT_RESOLUTION_FAILURE",
            DiagCode::E014_MODULE_NOT_FOUND => "E014_MODULE_NOT_FOUND",
            DiagCode::E015_IMPORT_CYCLE => "E015_IMPORT_CYCLE",
            DiagCode::E016_INVALID_LVALUE => "E016_INVALID_LVALUE",
            DiagCode::E017_MEMORY_OWNERSHIP_VIOLATION => "E017_MEMORY_OWNERSHIP_VIOLATION",
            DiagCode::E018_BORROW_CHECK_FAILURE => "E018_BORROW_CHECK_FAILURE",
            DiagCode::E019_CONCURRENCY_VIOLATION => "E019_CONCURRENCY_VIOLATION",
            DiagCode::E020_ATTRIBUTE_INVALID => "E020_ATTRIBUTE_INVALID",
            DiagCode::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str_code(s: &str) -> Self {
        match s {
            "E001" | "E001_UNSUPPORTED_EXPRESSION" => DiagCode::E001_UNSUPPORTED_EXPRESSION,
            "E002" | "E002_TYPE_MISMATCH" => DiagCode::E002_TYPE_MISMATCH,
            "E003" | "E003_UNDEFINED_NAME" => DiagCode::E003_UNDEFINED_NAME,
            "E004" | "E004_INVALID_C_GENERATED" => DiagCode::E004_INVALID_C_GENERATED,
            "E005" | "E005_PARSE_FAILURE" => DiagCode::E005_PARSE_FAILURE,
            "E006" | "E006_SEMANTIC_FAILURE" => DiagCode::E006_SEMANTIC_FAILURE,
            "E007" | "E007_CODEGEN_FAILURE" => DiagCode::E007_CODEGEN_FAILURE,
            "E008" | "E008_RANGE_BOUND_INVALID" => DiagCode::E008_RANGE_BOUND_INVALID,
            "E009" | "E009_ENUM_VARIANT_INVALID" => DiagCode::E009_ENUM_VARIANT_INVALID,
            "E010" | "E010_PATTERN_UNHANDLED" => DiagCode::E010_PATTERN_UNHANDLED,
            "E011" | "E011_CLOSURE_CAPTURE_INVALID" => DiagCode::E011_CLOSURE_CAPTURE_INVALID,
            "E012" | "E012_GENERICS_UNSUPPORTED" => DiagCode::E012_GENERICS_UNSUPPORTED,
            "E013" | "E013_TRAIT_RESOLUTION_FAILURE" => DiagCode::E013_TRAIT_RESOLUTION_FAILURE,
            "E014" | "E014_MODULE_NOT_FOUND" => DiagCode::E014_MODULE_NOT_FOUND,
            "E015" | "E015_IMPORT_CYCLE" => DiagCode::E015_IMPORT_CYCLE,
            "E016" | "E016_INVALID_LVALUE" => DiagCode::E016_INVALID_LVALUE,
            "E017" | "E017_MEMORY_OWNERSHIP_VIOLATION" => DiagCode::E017_MEMORY_OWNERSHIP_VIOLATION,
            "E018" | "E018_BORROW_CHECK_FAILURE" => DiagCode::E018_BORROW_CHECK_FAILURE,
            "E019" | "E019_CONCURRENCY_VIOLATION" => DiagCode::E019_CONCURRENCY_VIOLATION,
            "E020" | "E020_ATTRIBUTE_INVALID" => DiagCode::E020_ATTRIBUTE_INVALID,
            other => DiagCode::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for DiagCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_code_str())
    }
}

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Precise source span information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceSpan {
    pub file: String,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl SourceSpan {
    pub fn new(file: impl Into<String>, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            file: file.into(),
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    pub fn point(file: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            file: file.into(),
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
        }
    }
}

impl From<&crate::ast::span::Span> for SourceSpan {
    fn from(s: &crate::ast::span::Span) -> Self {
        SourceSpan {
            file: s.file.clone(),
            start_line: s.line,
            start_col: s.col,
            end_line: s.line,
            end_col: s.col,
        }
    }
}

impl From<crate::ast::span::Span> for SourceSpan {
    fn from(s: crate::ast::span::Span) -> Self {
        SourceSpan {
            file: s.file,
            start_line: s.line,
            start_col: s.col,
            end_line: s.line,
            end_col: s.col,
        }
    }
}

/// A structured compiler diagnostic
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagCode,
    pub severity: Severity,
    pub location: SourceSpan,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<SourceSpan>,
}

impl Diagnostic {
    pub fn error(code: &str, message: &str, file: &str, line: usize, col: usize) -> Self {
        Self {
            code: DiagCode::from_str_code(code),
            severity: Severity::Error,
            location: SourceSpan::point(file, line, col),
            message: message.to_string(),
            context: Vec::new(),
            expected: None,
            actual: None,
            suggestion: None,
            related: Vec::new(),
        }
    }

    pub fn warning(code: &str, message: &str, file: &str, line: usize, col: usize) -> Self {
        Self {
            code: DiagCode::from_str_code(code),
            severity: Severity::Warning,
            location: SourceSpan::point(file, line, col),
            message: message.to_string(),
            context: Vec::new(),
            expected: None,
            actual: None,
            suggestion: None,
            related: Vec::new(),
        }
    }

    pub fn info(code: &str, message: &str, file: &str, line: usize, col: usize) -> Self {
        Self {
            code: DiagCode::from_str_code(code),
            severity: Severity::Info,
            location: SourceSpan::point(file, line, col),
            message: message.to_string(),
            context: Vec::new(),
            expected: None,
            actual: None,
            suggestion: None,
            related: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.location = span;
        self
    }

    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    pub fn with_actual(mut self, actual: impl Into<String>) -> Self {
        self.actual = Some(actual.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.suggestion = Some(help.to_string());
        self
    }

    pub fn with_related(mut self, span: SourceSpan) -> Self {
        self.related.push(span);
        self
    }

    /// Render human-readable ANSI colored diagnostic with source snippet
    pub fn render(&self, source: &str) -> String {
        let mut out = String::new();

        let header_severity = match self.severity {
            Severity::Error => "Error".red().bold(),
            Severity::Warning => "Warning".yellow().bold(),
            Severity::Info => "Info".cyan().bold(),
        };

        // Header: Error[E001]: unsupported expression
        out.push_str(&format!(
            "{}[{}]: {}\n",
            header_severity,
            self.code.as_code_str().bright_red().bold(),
            self.message.bold()
        ));

        // Location:  --> src/file.end:24:18
        out.push_str(&format!(
            "  {} {}:{}:{}\n",
            "-->".blue().bold(),
            self.location.file,
            self.location.start_line,
            self.location.start_col
        ));

        let lines: Vec<&str> = source.lines().collect();
        let gutter_width = self.location.start_line.to_string().len().max(2);

        out.push_str(&format!("{:width$} {}\n", "", "|".blue().bold(), width = gutter_width));

        if self.location.start_line > 0 && self.location.start_line <= lines.len() {
            let line_idx = self.location.start_line - 1;
            let src_line = lines[line_idx];

            out.push_str(&format!(
                "{:width$} {} {}\n",
                self.location.start_line.to_string().blue().bold(),
                "|".blue().bold(),
                src_line,
                width = gutter_width
            ));

            let col = if self.location.start_col > 0 { self.location.start_col - 1 } else { 0 };
            let span_len = if self.location.end_col >= self.location.start_col && self.location.end_col > 0 {
                (self.location.end_col - self.location.start_col).max(1)
            } else {
                1
            };
            let pad = " ".repeat(col);
            let underline = match self.severity {
                Severity::Error => "^".repeat(span_len).red().bold(),
                Severity::Warning => "^".repeat(span_len).yellow().bold(),
                Severity::Info => "^".repeat(span_len).cyan().bold(),
            };

            out.push_str(&format!(
                "{:width$} {} {}{}\n",
                "",
                "|".blue().bold(),
                pad,
                underline,
                width = gutter_width
            ));
        }

        if let Some(ref exp) = self.expected {
            out.push_str(&format!(
                "{:width$} {} {} {}\n",
                "",
                "=".blue().bold(),
                "expected:".bright_yellow().bold(),
                exp,
                width = gutter_width
            ));
        }

        if let Some(ref act) = self.actual {
            out.push_str(&format!(
                "{:width$} {} {} {}\n",
                "",
                "=".blue().bold(),
                "actual:".bright_red().bold(),
                act,
                width = gutter_width
            ));
        }

        if let Some(ref s) = self.suggestion {
            out.push_str(&format!(
                "{:width$} {} {} {}\n",
                "",
                "=".blue().bold(),
                "help:".bright_yellow().bold(),
                s,
                width = gutter_width
            ));
        }

        for ctx in &self.context {
            out.push_str(&format!(
                "{:width$} {} {} {}\n",
                "",
                "=".blue().bold(),
                "note:".bright_cyan().bold(),
                ctx,
                width = gutter_width
            ));
        }

        out
    }

    /// Serialize diagnostic to JSON Value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.code.as_code_str(),
            "severity": self.severity.as_str(),
            "file": self.location.file,
            "line": self.location.start_line,
            "column": self.location.start_col,
            "message": self.message,
            "expected": self.expected,
            "actual": self.actual,
            "suggestion": self.suggestion,
            "context": self.context,
            "related": self.related,
        })
    }
}

/// An accumulator for compiler diagnostics across parsing, semantics, and codegen
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticAccumulator {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticAccumulator {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count()
    }

    pub fn info_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Info).count()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn emit_all(&self, source_map: Option<&std::collections::HashMap<String, String>>) {
        for diag in &self.diagnostics {
            let empty = String::new();
            let src = source_map
                .and_then(|sm| sm.get(&diag.location.file))
                .unwrap_or(&empty);
            eprintln!("{}", diag.render(src));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_construction_and_fields() {
        let span = SourceSpan::new("test.end", 10, 5, 10, 15);
        let diag = Diagnostic::error("E001", "unsupported expression: ListComprehension", "test.end", 10, 5)
            .with_span(span.clone())
            .with_expected("explicit implementation or simple expression")
            .with_actual("ListComprehension")
            .with_suggestion("use a regular for loop instead")
            .with_context("list comprehensions are not yet lowered to C backend");

        assert_eq!(diag.code, DiagCode::E001_UNSUPPORTED_EXPRESSION);
        assert_eq!(diag.code.as_code_str(), "E001");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.location.file, "test.end");
        assert_eq!(diag.location.start_line, 10);
        assert_eq!(diag.location.start_col, 5);
        assert_eq!(diag.location.end_line, 10);
        assert_eq!(diag.location.end_col, 15);
        assert_eq!(diag.expected, Some("explicit implementation or simple expression".to_string()));
        assert_eq!(diag.actual, Some("ListComprehension".to_string()));
        assert_eq!(diag.suggestion, Some("use a regular for loop instead".to_string()));
        assert_eq!(diag.context.len(), 1);
    }

    #[test]
    fn test_diagnostic_json_serialization() {
        let diag = Diagnostic::error("E001", "unsupported expression", "hello.end", 3, 5);
        let json = diag.to_json();

        assert_eq!(json["code"], "E001");
        assert_eq!(json["severity"], "error");
        assert_eq!(json["file"], "hello.end");
        assert_eq!(json["line"], 3);
        assert_eq!(json["column"], 5);
        assert_eq!(json["message"], "unsupported expression");
    }

    #[test]
    fn test_accumulator_error_determination() {
        let mut acc = DiagnosticAccumulator::new();
        assert!(!acc.has_errors());
        assert_eq!(acc.error_count(), 0);

        // Add warning only
        acc.add(Diagnostic::warning("W001", "unused variable", "test.end", 1, 1));
        assert!(!acc.has_errors());
        assert_eq!(acc.error_count(), 0);
        assert_eq!(acc.warning_count(), 1);

        // Add error
        acc.add(Diagnostic::error("E001", "unsupported expression", "test.end", 2, 4));
        assert!(acc.has_errors());
        assert_eq!(acc.error_count(), 1);
        assert_eq!(acc.len(), 2);
    }

    #[test]
    fn test_reserved_codes_available() {
        let codes = [
            ("E001", DiagCode::E001_UNSUPPORTED_EXPRESSION),
            ("E002", DiagCode::E002_TYPE_MISMATCH),
            ("E003", DiagCode::E003_UNDEFINED_NAME),
            ("E004", DiagCode::E004_INVALID_C_GENERATED),
            ("E005", DiagCode::E005_PARSE_FAILURE),
            ("E006", DiagCode::E006_SEMANTIC_FAILURE),
            ("E007", DiagCode::E007_CODEGEN_FAILURE),
            ("E008", DiagCode::E008_RANGE_BOUND_INVALID),
            ("E009", DiagCode::E009_ENUM_VARIANT_INVALID),
            ("E010", DiagCode::E010_PATTERN_UNHANDLED),
            ("E011", DiagCode::E011_CLOSURE_CAPTURE_INVALID),
            ("E012", DiagCode::E012_GENERICS_UNSUPPORTED),
            ("E013", DiagCode::E013_TRAIT_RESOLUTION_FAILURE),
            ("E014", DiagCode::E014_MODULE_NOT_FOUND),
            ("E015", DiagCode::E015_IMPORT_CYCLE),
            ("E016", DiagCode::E016_INVALID_LVALUE),
            ("E017", DiagCode::E017_MEMORY_OWNERSHIP_VIOLATION),
            ("E018", DiagCode::E018_BORROW_CHECK_FAILURE),
            ("E019", DiagCode::E019_CONCURRENCY_VIOLATION),
            ("E020", DiagCode::E020_ATTRIBUTE_INVALID),
        ];

        for (str_code, enum_code) in codes {
            assert_eq!(DiagCode::from_str_code(str_code), enum_code);
            assert_eq!(enum_code.as_code_str(), str_code);
        }
    }

    #[test]
    fn test_gate_fresh_clone_app_template_exists() {
        let template_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/ui/app_template.html");
        assert!(
            template_path.exists(),
            "app_template.html must be present in fresh clones"
        );
        let content = std::fs::read_to_string(&template_path).expect("failed to read app_template.html");
        assert!(content.contains("<!DOCTYPE html>"), "app_template.html must contain HTML header");
    }

    #[test]
    fn test_gate_unsupported_expression_emits_e001_diagnostic() {
        let mut backend = crate::codegen::c_backend::CBackend::new();
        let span = crate::ast::span::Span {
            line: 42,
            col: 15,
            file: "test_unsupported.end".to_string(),
        };

        let dummy_expr = crate::ast::Expression::Cascade {
            target: Box::new(crate::ast::Expression::Lit(crate::ast::Literal::Int(1), span.clone())),
            operations: Vec::new(),
            is_null_aware: false,
            span: span.clone(),
        };

        let mut module = crate::ast::Module::empty("test_mod");
        module.statements.push(crate::ast::Statement::Expression(dummy_expr));

        let _ = backend.generate_with_options(&module, false);

        assert!(backend.has_errors(), "CBackend must report errors for unsupported expression");
        let diags = backend.diagnostics();
        assert_eq!(diags.error_count(), 1, "Must have exactly 1 error diagnostic");
        let first = &diags.diagnostics()[0];
        assert_eq!(first.code, DiagCode::E001_UNSUPPORTED_EXPRESSION);
        assert_eq!(first.severity, Severity::Error);
        assert_eq!(first.location.start_line, 42);
        assert_eq!(first.location.start_col, 15);
        assert_eq!(first.location.file, "test_unsupported.end");

        let rendered = first.render("let x = a..b;");
        assert!(rendered.contains("[E001]"));
        assert!(rendered.contains("unsupported expression: Cascade"));
    }

    #[test]
    fn test_gate_zero_expressive_expr_fallbacks() {
        let src_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let forbidden_token = ["expressive", "expr"].join("_");
        for entry in walkdir(src_dir) {
            if entry.extension().map_or(false, |ext| ext == "rs") {
                // Skip the test file itself
                if entry.file_name().map_or(false, |n| n == "mod.rs" && entry.parent().unwrap().ends_with("diagnostics")) {
                    continue;
                }
                let content = std::fs::read_to_string(&entry).unwrap();
                assert!(
                    !content.contains(&forbidden_token),
                    "File {:?} contains banned fallback token {:?}!",
                    entry,
                    forbidden_token
                );
            }
        }
    }

    #[test]
    fn test_gate_cli_release_flag_plumbing() {
        use clap::Parser;
        // Global flag before subcommand
        let cli1 = crate::cli::Cli::try_parse_from(["endc", "--release", "build", "main.end"]).expect("parse failed");
        assert!(cli1.release, "Global --release flag must be true");

        // Subcommand flag
        let cli2 = crate::cli::Cli::try_parse_from(["endc", "build", "--release", "main.end"]).expect("parse failed");
        if let crate::cli::Commands::Build(build_args) = cli2.command {
            assert!(build_args.release, "Subcommand --release flag must be true");
        } else {
            panic!("Expected Build command");
        }
    }

    fn walkdir(dir: std::path::PathBuf) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(walkdir(path));
                } else {
                    files.push(path);
                }
            }
        }
        files
    }
}
