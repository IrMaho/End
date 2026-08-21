use colored::*;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub filename: String,
    pub line: usize,
    pub column: usize,
    pub span_len: usize,
    pub help: Option<String>,
    pub suggestion: Option<(String, String)>, // (old_snippet, new_snippet)
}

impl Diagnostic {
    pub fn error(code: &str, message: &str, filename: &str, line: usize, column: usize) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            filename: filename.to_string(),
            line,
            column,
            span_len: 1,
            help: None,
            suggestion: None,
        }
    }

    pub fn with_span(mut self, len: usize) -> Self {
        self.span_len = len.max(1);
        self
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    pub fn with_suggestion(mut self, original: &str, replacement: &str) -> Self {
        self.suggestion = Some((original.to_string(), replacement.to_string()));
        self
    }

    pub fn render(&self, source: &str) -> String {
        let mut out = String::new();

        // Header: Error[E0412]: cannot find type `UserSession` in this scope
        out.push_str(&format!(
            "{}[{}]: {}\n",
            "Error".red().bold(),
            self.code.bright_red().bold(),
            self.message.bold()
        ));

        // Location:  --> src/auth.end:24:18
        out.push_str(&format!(
            "  {} {}:{}:{}\n",
            "-->".blue().bold(),
            self.filename,
            self.line,
            self.column
        ));

        let lines: Vec<&str> = source.lines().collect();
        let gutter_width = self.line.to_string().len().max(2);

        out.push_str(&format!("{:width$} {}\n", "", "|".blue().bold(), width = gutter_width));

        if self.line > 0 && self.line <= lines.len() {
            let line_idx = self.line - 1;
            let src_line = lines[line_idx];

            // Source line: 24 |     val session: UserSession = get_session()
            out.push_str(&format!(
                "{:width$} {} {}\n",
                self.line.to_string().blue().bold(),
                "|".blue().bold(),
                src_line,
                width = gutter_width
            ));

            // Underline:     |                  ^^^^^^^^^^^ not found in this scope
            let col = if self.column > 0 { self.column - 1 } else { 0 };
            let pad = " ".repeat(col);
            let underline = "^".repeat(self.span_len).red().bold();

            out.push_str(&format!(
                "{:width$} {} {}{}\n",
                "",
                "|".blue().bold(),
                pad,
                underline,
                width = gutter_width
            ));
        }

        // Help footer:  = help: a struct with a similar name exists: `User`
        if let Some(ref h) = self.help {
            out.push_str(&format!(
                "{:width$} {} {} {}\n",
                "",
                "=".blue().bold(),
                "help:".bright_yellow().bold(),
                h,
                width = gutter_width
            ));
        }

        out
    }
}
