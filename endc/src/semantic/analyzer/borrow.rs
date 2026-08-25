use super::types::{ActiveLoan, DiagnosticError, LoanKind, OwnershipState};
use crate::ast::span::Span;

#[derive(Debug, Clone, Default)]
pub struct BorrowChecker {
    pub active_loans: Vec<ActiveLoan>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            active_loans: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.active_loans.clear();
    }

    pub fn release_loans_for_holder(&mut self, holder: &str) {
        self.active_loans.retain(|loan| loan.holder != holder);
    }

    pub fn check_use_after_move(
        &self,
        name: &str,
        state: Option<&OwnershipState>,
        span: &Span,
        errors: &mut Vec<DiagnosticError>,
    ) {
        if let Some(OwnershipState::Moved { to, at_line }) = state {
            errors.push(
                DiagnosticError::new(
                    "E018",
                    format!(
                        "BorrowCheckFailure: use of moved value '{}' at line {} (previously moved to '{}' at line {})",
                        name, span.line, to, at_line
                    ),
                    span.line,
                    span.col,
                    "BorrowCheckFailure",
                )
                .with_suggestion(format!("clone '{}' or reinitialize before transferring ownership", name))
                .with_context(format!("value moved to '{}' at line {}", to, at_line)),
            );
        } else if let Some(OwnershipState::Freed { at_line }) = state {
            errors.push(
                DiagnosticError::new(
                    "E018",
                    format!(
                        "BorrowCheckFailure: use of freed value '{}' at line {} (previously freed at line {})",
                        name, span.line, at_line
                    ),
                    span.line,
                    span.col,
                    "BorrowCheckFailure",
                )
                .with_suggestion(format!("do not access '{}' after calling free()", name))
                .with_context(format!("memory freed at line {}", at_line)),
            );
        }
    }

    pub fn check_free(
        &mut self,
        name: &str,
        state: Option<&OwnershipState>,
        span: &Span,
        errors: &mut Vec<DiagnosticError>,
    ) -> bool {
        if let Some(OwnershipState::Freed { at_line }) = state {
            errors.push(
                DiagnosticError::new(
                    "E018",
                    format!(
                        "BorrowCheckFailure: double-free detected for '{}' at line {} (previously freed at line {})",
                        name, span.line, at_line
                    ),
                    span.line,
                    span.col,
                    "BorrowCheckFailure",
                )
                .with_suggestion(format!("remove redundant free() call for '{}'", name))
                .with_context(format!("first free occurred at line {}", at_line)),
            );
            return false;
        }

        if let Some(OwnershipState::Moved { to, at_line }) = state {
            errors.push(
                DiagnosticError::new(
                    "E018",
                    format!(
                        "BorrowCheckFailure: cannot free moved value '{}' at line {} (previously moved to '{}' at line {})",
                        name, span.line, to, at_line
                    ),
                    span.line,
                    span.col,
                    "BorrowCheckFailure",
                )
                .with_suggestion(format!("free the owner '{}' instead", to)),
            );
            return false;
        }

        if let Some(existing) = self.active_loans.iter().find(|l| l.place == name) {
            errors.push(
                DiagnosticError::new(
                    "E018",
                    format!(
                        "BorrowCheckFailure: cannot free '{}' at line {} while actively borrowed by '{}' (borrowed at line {})",
                        name, span.line, existing.holder, existing.borrowed_at
                    ),
                    span.line,
                    span.col,
                    "BorrowCheckFailure",
                )
                .with_suggestion(format!("ensure reference '{}' goes out of scope before freeing '{}'", existing.holder, name)),
            );
            return false;
        }

        true
    }

    pub fn check_borrow_creation(
        &mut self,
        place_name: &str,
        holder: &str,
        kind: LoanKind,
        span: &Span,
        errors: &mut Vec<DiagnosticError>,
    ) {
        if kind == LoanKind::Mutable {
            // Cannot mutably borrow if already borrowed (shared or mutable)
            if let Some(existing) = self.active_loans.iter().find(|l| l.place == place_name) {
                errors.push(
                    DiagnosticError::new(
                        "E018",
                        format!(
                            "BorrowCheckFailure: cannot borrow '{}' mutably at line {} because it is already borrowed by '{}' at line {}",
                            place_name, span.line, existing.holder, existing.borrowed_at
                        ),
                        span.line,
                        span.col,
                        "BorrowCheckFailure",
                    )
                    .with_suggestion("release previous reference before taking a mutable reference"),
                );
                return;
            }
        } else {
            // Shared borrow: cannot borrow if already mutably borrowed
            if let Some(existing) = self
                .active_loans
                .iter()
                .find(|l| l.place == place_name && l.kind == LoanKind::Mutable)
            {
                errors.push(
                    DiagnosticError::new(
                        "E018",
                        format!(
                            "BorrowCheckFailure: cannot borrow '{}' shared at line {} because it is already mutably borrowed by '{}' at line {}",
                            place_name, span.line, existing.holder, existing.borrowed_at
                        ),
                        span.line,
                        span.col,
                        "BorrowCheckFailure",
                    )
                    .with_suggestion("release previous mutable reference before borrowing again"),
                );
                return;
            }
        }

        self.active_loans.push(ActiveLoan {
            place: place_name.to_string(),
            kind,
            borrowed_at: span.line,
            holder: holder.to_string(),
        });
    }

    pub fn check_mutation_while_borrowed(
        &self,
        place_name: &str,
        span: &Span,
        errors: &mut Vec<DiagnosticError>,
    ) {
        if let Some(loan) = self
            .active_loans
            .iter()
            .find(|l| l.place == place_name && l.holder != place_name)
        {
            errors.push(
                DiagnosticError::new(
                    "E018",
                    format!(
                        "BorrowCheckFailure: cannot mutate '{}' at line {} because it is currently borrowed by '{}' (borrowed at line {})",
                        place_name, span.line, loan.holder, loan.borrowed_at
                    ),
                    span.line,
                    span.col,
                    "BorrowCheckFailure",
                )
                .with_suggestion(format!(
                    "ensure borrow '{}' goes out of scope before modifying '{}'",
                    loan.holder, place_name
                )),
            );
        }
    }
}
