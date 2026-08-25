use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OwnershipState {
    Uninitialized,
    Owned,
    Moved { to: String, at_line: usize },
    Freed { at_line: usize },
    BorrowedShared(usize), // count
    BorrowedMut(usize),    // line where &mut was taken
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoanKind {
    Shared,
    Mutable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveLoan {
    pub place: String,
    pub kind: LoanKind,
    pub borrowed_at: usize,
    pub holder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticError {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_suggestion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<String>,
}

impl DiagnosticError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, line: usize, col: usize, kind: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            line,
            col,
            kind: kind.into(),
            repair_suggestion: None,
            expected: None,
            actual: None,
            context: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.repair_suggestion = Some(suggestion.into());
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

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }
}
