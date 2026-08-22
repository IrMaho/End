use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        let span = self.current_span();
        let pattern = self.parse_pattern()?;

        let mut guard = None;
        if self.match_token(&TokenKind::If) {
            guard = Some(self.parse_expression()?);
        }

        self.expect(TokenKind::FatArrow)?;

        let body = if self.check(&TokenKind::LBrace) {
            self.parse_block()?
        } else {
            let stmt = self.parse_statement()?;
            Block {
                statements: vec![stmt],
                span: span.clone(),
            }
        };

        Ok(MatchArm {
            pattern,
            guard,
            body,
            span,
        })
    }


    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern, String> {
        if self.match_token(&TokenKind::Underscore) {
            return Ok(Pattern::Wildcard);
        }

        if self.match_token(&TokenKind::Dot) {
            let variant_name = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected variant name after '.', found {:?}", other)),
            };

            let mut binding = None;
            if self.match_token(&TokenKind::LParen) {
                if let TokenKind::Ident(b) = self.advance().kind {
                    binding = Some(b);
                }
                self.expect(TokenKind::RParen)?;
            }

            return Ok(Pattern::Variant {
                enum_name: None,
                variant_name,
                binding,
            });
        }

        match self.peek_kind() {
            TokenKind::IntLit(n) => {
                let val = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(val)))
            }
            TokenKind::StringLit(s) => {
                let val = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(val)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            TokenKind::Ident(name) => {
                let id = name.clone();
                self.advance();
                let is_enum_variant = if self.match_token(&TokenKind::Dot) {
                    true
                } else if self.check(&TokenKind::Colon) {
                    self.advance();
                    self.match_token(&TokenKind::Colon)
                } else {
                    false
                };

                if is_enum_variant {
                    let vname = match self.advance().kind {
                        TokenKind::Ident(n) => n,
                        other => return Err(format!("Expected variant name, found {:?}", other)),
                    };
                    let mut binding = None;
                    if self.match_token(&TokenKind::LParen) {
                        if let TokenKind::Ident(b) = self.advance().kind {
                            binding = Some(b);
                        }
                        self.expect(TokenKind::RParen)?;
                    }
                    Ok(Pattern::Variant {
                        enum_name: Some(id),
                        variant_name: vname,
                        binding,
                    })
                } else {
                    Ok(Pattern::Ident(id))
                }
            }
            other => Err(format!("Invalid pattern syntax: {:?} at line {}", other, self.current_span().line)),
        }
    }

}
