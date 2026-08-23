use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_import(&mut self) -> Result<ImportStmt, String> {
        let span = self.current_span();
        if self.check(&TokenKind::Import) {
            self.advance();
        } else if self.check(&TokenKind::Use) {
            self.advance();
        } else {
            return Err(format!("Expected 'use' or 'import', found {:?}", self.peek_kind()));
        }

        let (kind, path) = match self.peek_kind() {
            TokenKind::Directive(d) => {
                let dir = d.clone();
                self.advance();
                self.expect(TokenKind::LParen)?;
                let p = match self.advance().kind {
                    TokenKind::StringLit(s) => s,
                    other => return Err(format!("Expected string path in import directive, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;

                match dir.as_str() {
                    "@c" => (ImportKind::C(p.clone()), p),
                    "@zig" => (ImportKind::Zig(p.clone()), p),
                    "@rust" => (ImportKind::Rust(p.clone()), p),
                    "@go" => (ImportKind::Go(p.clone()), p),
                    _ => (ImportKind::Standard, p),
                }
            }
            TokenKind::StringLit(s) => {
                let p = s.clone();
                self.advance();
                (ImportKind::Standard, p)
            }
            TokenKind::Ident(_) => {
                let mut full_path = String::new();
                while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::As) && !self.check(&TokenKind::EOF) {
                    match self.peek_kind() {
                        TokenKind::Ident(id) => {
                            full_path.push_str(id);
                            self.advance();
                        }
                        TokenKind::Dot => {
                            full_path.push('.');
                            self.advance();
                        }
                        TokenKind::Colon => {
                            self.advance();
                            if self.match_token(&TokenKind::Colon) {
                                full_path.push_str("::");
                            } else {
                                full_path.push(':');
                            }
                        }
                        TokenKind::Star => {
                            full_path.push('*');
                            self.advance();
                        }
                        TokenKind::Slash => {
                            full_path.push('/');
                            self.advance();
                        }
                        _ => break,
                    }
                }
                (ImportKind::Standard, full_path.clone())
            }
            other => return Err(format!("Invalid import/use syntax: {:?} at line {}", other, span.line)),
        };

        let mut alias = None;
        if self.match_token(&TokenKind::As) {
            match self.advance().kind {
                TokenKind::Ident(a) => alias = Some(a),
                other => return Err(format!("Expected alias identifier after 'as', found {:?}", other)),
            }
        }

        self.match_token(&TokenKind::SemiColon);

        Ok(ImportStmt {
            kind,
            path,
            alias,
            span,
        })
    }
}
