use super::types::{LintSeverity, LintViolation, Linter};
use colored::*;
use std::path::Path;

impl Linter {
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


}
