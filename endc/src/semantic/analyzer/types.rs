#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum OwnershipState {
    Uninitialized,
    Owned,
    Moved { to: String, at_line: usize },
    BorrowedShared(usize), // count
    BorrowedMut(usize),    // line where &mut was taken
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoanKind {
    Shared,
    Mutable,
}

#[derive(Debug, Clone)]
pub struct ActiveLoan {
    pub place: String,
    pub kind: LoanKind,
    pub borrowed_at: usize,
    pub holder: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticError {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub kind: String,
    pub repair_suggestion: Option<String>,
}
