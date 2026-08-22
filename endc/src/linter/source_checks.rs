use super::types::{LintSeverity, LintViolation, Linter};
use std::path::Path;

impl Linter {
pub(crate) fn check_file_lines(&mut self, source: &str) {
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

    pub(crate) fn check_comments(&mut self, source: &str) {
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


}

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

