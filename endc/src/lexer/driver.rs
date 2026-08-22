use super::cursor::Lexer;
use super::keywords::{match_keyword_or_ident, match_macro_call};
use super::number::parse_number;
use super::operator::parse_operator;
use super::string::parse_string;
use super::tokens::{Token, TokenKind};
use crate::ast::Span;

impl<'a> Lexer<'a> {
    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments();

        let start_line = self.line;
        let start_col = self.col;
        let span = Span::new(&self.filename, start_line, start_col);

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::EOF,
                    span,
                })
            }
        };

        // Directive
        if ch == '@' {
            self.advance();
            let mut name = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    name.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            return Ok(Token {
                kind: TokenKind::Directive(format!("@{}", name)),
                span,
            });
        }

        if ch == '_' && !self.peek_next().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Underscore,
                span,
            });
        }

        // Morphic Template Identifiers: e.g. {platform}_send or {target}_Client
        if ch == '{' {
            let mut i = 1;
            let mut is_morphic = false;
            let mut has_close = false;
            let mut only_ident = true;
            while let Some(c) = self.peek_offset(i) {
                if c == '}' {
                    has_close = true;
                    if let Some(after) = self.peek_offset(i + 1) {
                        if after == '_' || after.is_alphanumeric() {
                            is_morphic = true;
                        }
                    }
                    break;
                } else if !c.is_alphanumeric() && c != '_' {
                    only_ident = false;
                    break;
                }
                i += 1;
            }
            if has_close && only_ident && is_morphic {
                self.advance(); // consume '{'
                let mut morphic_str = String::from("{");
                while let Some(c) = self.peek() {
                    morphic_str.push(c);
                    self.advance();
                    if c == '}' {
                        break;
                    }
                }
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        morphic_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                return Ok(Token {
                    kind: TokenKind::MorphicIdent(morphic_str),
                    span,
                });
            }
        }

        // Identifiers or keywords
        if ch.is_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = if self.peek() == Some('!') && self.peek_next() != Some('=') {
                self.advance();
                match_macro_call(&ident)
            } else {
                match_keyword_or_ident(&ident)
            };

            return Ok(Token { kind, span });
        }

        // Numeric literals
        if ch.is_ascii_digit() {
            return parse_number(self, ch, span);
        }

        // String literals
        if ch == '"' {
            return parse_string(self, span);
        }

        // Operators & Single character tokens
        parse_operator(self, ch, span, start_line, start_col)
    }

    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            if tok.kind == TokenKind::EOF {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        Ok(tokens)
    }
}
