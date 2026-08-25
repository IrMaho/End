use crate::ast::Span;
use crate::diagnostics::{Diagnostic, DiagnosticAccumulator, DiagCode, Severity, SourceSpan};
use crate::lexer::{Token, TokenKind};
use std::collections::HashSet;

#[allow(dead_code)]
pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) cursor: usize,
    pub filename: String,
    pub enum_names: HashSet<String>,
    pub diagnostics: DiagnosticAccumulator,
}

#[derive(Clone, Debug, Copy)]
pub struct ParserCheckpoint {
    pub cursor: usize,
    pub diag_len: usize,
}

impl Parser {
    pub fn new(filename: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            filename: filename.into(),
            enum_names: HashSet::new(),
            diagnostics: DiagnosticAccumulator::new(),
        }
    }

    pub fn checkpoint(&self) -> ParserCheckpoint {
        ParserCheckpoint {
            cursor: self.cursor,
            diag_len: self.diagnostics.len(),
        }
    }

    pub fn restore_checkpoint(&mut self, cp: ParserCheckpoint) {
        self.cursor = cp.cursor;
        self.diagnostics.truncate(cp.diag_len);
    }

    pub(crate) fn peek(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else if !self.tokens.is_empty() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            static EOF_TOKEN: std::sync::OnceLock<Token> = std::sync::OnceLock::new();
            EOF_TOKEN.get_or_init(|| Token {
                kind: TokenKind::EOF,
                span: Span::new("<unknown>", 1, 1),
            })
        }
    }

    pub(crate) fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    pub(crate) fn peek_next_kind(&self) -> Option<&TokenKind> {
        if self.cursor + 1 < self.tokens.len() {
            Some(&self.tokens[self.cursor + 1].kind)
        } else {
            None
        }
    }

    pub(crate) fn peek_ahead(&self, offset: usize) -> Option<&TokenKind> {
        if self.cursor + offset < self.tokens.len() {
            Some(&self.tokens[self.cursor + offset].kind)
        } else {
            None
        }
    }

    pub(crate) fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else if !self.tokens.is_empty() {
            self.tokens[self.tokens.len() - 1].clone()
        } else {
            Token {
                kind: TokenKind::EOF,
                span: Span::new(self.filename.clone(), 1, 1),
            }
        }
    }

    pub(crate) fn previous(&self) -> Option<&Token> {
        if self.cursor > 0 && self.cursor - 1 < self.tokens.len() {
            Some(&self.tokens[self.cursor - 1])
        } else {
            None
        }
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    pub(crate) fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn emit_e005(&mut self, span: &Span, expected: &str, actual: &str, message: &str) -> String {
        let msg = if message.starts_with("Error[E005]:") {
            message.to_string()
        } else {
            format!("Error[E005]: {}", message)
        };
        let src_span = SourceSpan::new(&self.filename, span.line, span.col, span.line, span.col + 1);
        let diag = Diagnostic {
            code: DiagCode::E005_PARSE_FAILURE,
            severity: Severity::Error,
            location: src_span,
            message: msg.clone(),
            context: Vec::new(),
            expected: Some(expected.to_string()),
            actual: Some(actual.to_string()),
            suggestion: None,
            related: Vec::new(),
        };
        self.diagnostics.add(diag);
        msg
    }

    pub(crate) fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let current = self.peek();
        if std::mem::discriminant(&current.kind) == std::mem::discriminant(&kind) {
            Ok(self.advance())
        } else {
            let span = current.span.clone();
            let actual = format!("{:?}", current.kind);
            let expected = format!("{:?}", kind);
            let raw_msg = format!(
                "Expected token {:?}, found {:?} at line {}, col {}",
                kind, current.kind, span.line, span.col
            );
            let formatted = self.emit_e005(&span, &expected, &actual, &raw_msg);
            Err(formatted)
        }
    }

    pub fn current_span(&self) -> Span {
        self.peek().span.clone()
    }

    pub(crate) fn synchronize(&mut self) {
        if self.cursor < self.tokens.len() {
            self.advance();
        }
        while !self.check(&TokenKind::EOF) {
            if let Some(prev) = self.previous() {
                if prev.kind == TokenKind::SemiColon {
                    return;
                }
            }
            match self.peek_kind() {
                TokenKind::Fn
                | TokenKind::Val
                | TokenKind::Mut
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Class
                | TokenKind::Feature
                | TokenKind::Mod
                | TokenKind::Contract
                | TokenKind::Architecture
                | TokenKind::Migration
                | TokenKind::Trait
                | TokenKind::Extend
                | TokenKind::Augment
                | TokenKind::Return
                | TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Match
                | TokenKind::RBrace => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

