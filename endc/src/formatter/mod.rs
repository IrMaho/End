use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatReport {
    pub files_formatted: usize,
    pub lines_realigned: usize,
    pub is_clean: bool,
}

pub struct EndFormatter;

impl EndFormatter {
    pub fn format_source(source: &str) -> String {
        let mut formatted = String::new();
        let mut indent_level: usize = 0;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }

            if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
                indent_level = indent_level.saturating_sub(1);
            }

            let indent = "    ".repeat(indent_level);
            formatted.push_str(&indent);
            formatted.push_str(trimmed);
            formatted.push('\n');

            if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
                indent_level += 1;
            }
        }

        formatted
    }

    pub fn check_formatting(source: &str) -> bool {
        let formatted = Self::format_source(source);
        formatted == source
    }
}
