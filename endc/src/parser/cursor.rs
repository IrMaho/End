use crate::ast::Span;
use crate::lexer::{Token, TokenKind};
use std::collections::HashSet;

#[allow(dead_code)]
pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) cursor: usize,
    pub filename: String,
    pub enum_names: HashSet<String>,
}

impl Parser {
    pub fn new(filename: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            filename: filename.into(),
            enum_names: HashSet::new(),
        }
    }

    pub(crate) fn peek(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            &self.tokens[self.tokens.len() - 1]
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

    pub(crate) fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else {
            self.tokens[self.tokens.len() - 1].clone()
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

    pub(crate) fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let current = self.peek();
        if std::mem::discriminant(&current.kind) == std::mem::discriminant(&kind) {
            Ok(self.advance())
        } else {
            Err(format!(
                "Expected token {:?}, found {:?} at line {}, col {}",
                kind, current.kind, current.span.line, current.span.col
            ))
        }
    }

    pub fn current_span(&self) -> Span {
        self.peek().span.clone()
    }
}
