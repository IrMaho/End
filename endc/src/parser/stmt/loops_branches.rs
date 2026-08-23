use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_loops_branches_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Val
            | TokenKind::Mut
            | TokenKind::Return
            | TokenKind::If
            | TokenKind::While
            | TokenKind::Parallel
            | TokenKind::For
            | TokenKind::Match
            | TokenKind::Region
            | TokenKind::InlineC
            | TokenKind::Asm
            | TokenKind::Target
            | TokenKind::Defer
            | TokenKind::Spawn
            | TokenKind::Skip => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_loops_branches_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_loops_branches_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Val | TokenKind::Mut => {
                let is_mut = self.peek_kind() == &TokenKind::Mut;
                self.advance();

                let name = self.parse_identifier_or_keyword()?;

                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }

                let mut initializer = None;
                if self.match_token(&TokenKind::Equal) {
                    initializer = Some(self.parse_expression()?);
                }

                self.match_token(&TokenKind::SemiColon);

                Ok(Statement::VarDecl {
                    name,
                    var_type,
                    is_mut,
                    is_lease: false,
                    initializer,
                    span,
                })
            }
            TokenKind::Return => {
                self.advance();
                let value = if !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::RBrace) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Return { value, span })
            }
            TokenKind::If => {
                self.advance();
                let condition = self.parse_expression()?;
                self.match_token(&TokenKind::Colon);
                let then_block = self.parse_block()?;
                let mut else_block = None;
                if self.match_token(&TokenKind::Else) {
                    self.match_token(&TokenKind::Colon);
                    if self.check(&TokenKind::If) {
                        let if_stmt = self.parse_statement()?;
                        else_block = Some(Block {
                            statements: vec![if_stmt],
                            span: self.current_span(),
                        });
                    } else {
                        else_block = Some(self.parse_block()?);
                    }
                }
                Ok(Statement::If {
                    condition,
                    then_block,
                    else_block,
                    span,
                })
            }
            TokenKind::While => {
                self.advance();
                let condition = self.parse_expression()?;
                self.match_token(&TokenKind::Colon);
                let body = self.parse_block()?;
                Ok(Statement::While {
                    condition,
                    body,
                    span,
                })
            }
            TokenKind::Parallel => {
                self.advance();
                if self.match_token(&TokenKind::Choose) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "choose" } else { false }) {
                    if let TokenKind::Ident(_) = self.peek_kind() { self.advance(); }
                    self.match_token(&TokenKind::Colon);
                    self.expect(TokenKind::LBrace)?;
                    let mut branches = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.peek_kind() == &TokenKind::Ident("branch".to_string()) {
                            self.advance();
                        }
                        let name = self.parse_identifier_or_keyword()?;
                        if self.match_token(&TokenKind::FatArrow) {}
                        let blk = self.parse_block()?;
                        branches.push((name, blk));
                        self.match_token(&TokenKind::Comma);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::ParallelChoose { branches, span });
                }
                self.expect(TokenKind::For)?;
                let item_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected item name after 'parallel for', found {:?}", other)),
                };
                self.expect(TokenKind::In)?;
                let iterable = self.parse_expression()?;
                self.match_token(&TokenKind::Colon);
                let body = self.parse_block()?;
                Ok(Statement::ParallelFor {
                    item_name,
                    iterable,
                    body,
                    span,
                })
            }
            TokenKind::For => {
                self.advance();
                let item_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected item name after 'for', found {:?}", other)),
                };
                self.expect(TokenKind::In)?;
                let iterable = self.parse_expression()?;
                self.match_token(&TokenKind::Colon);
                let body = self.parse_block()?;
                Ok(Statement::ForIn {
                    item_name,
                    iterable,
                    body,
                    span,
                })
            }
            TokenKind::Match => {
                self.advance();
                let expr = self.parse_expression()?;
                self.match_token(&TokenKind::Colon);
                self.expect(TokenKind::LBrace)?;
                let mut arms = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    arms.push(self.parse_match_arm()?);
                    self.match_token(&TokenKind::Comma);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::Match { expr, arms, span })
            }
            TokenKind::Region => {
                self.advance();
                let reg_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected region name, found {:?}", other)),
                };
                let body = self.parse_block()?;
                Ok(Statement::RegionBlock {
                    name: reg_name,
                    body,
                    span,
                })
            }
            TokenKind::InlineC => {
                let span = self.current_span();
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut code = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    match self.peek_kind() {
                        TokenKind::StringLit(s) => {
                            code.push_str(s);
                            code.push('\n');
                            self.advance();
                        }
                        _ => {
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::InlineC { code, span })
            }
            TokenKind::Asm => {
                self.advance();
                let arch = match self.advance().kind {
                    TokenKind::Ident(n) | TokenKind::StringLit(n) => n,
                    other => return Err(format!("Expected target architecture for asm, found {:?}", other)),
                };
                self.expect(TokenKind::LBrace)?;
                let mut asm_code = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    match self.peek_kind() {
                        TokenKind::StringLit(s) => {
                            asm_code.push_str(s);
                            asm_code.push('\n');
                            self.advance();
                        }
                        _ => {
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AsmBlock {
                    arch,
                    code: asm_code,
                    span,
                })
            }
            TokenKind::Target => {
                self.advance();
                let target_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected target name after 'target', found {:?}", other)),
                };
                let body = self.parse_block()?;
                Ok(Statement::TargetBlock {
                    target: target_name,
                    body,
                    span,
                })
            }
            TokenKind::Defer => {
                self.advance();
                let expr = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Defer { expr, span })
            }
            TokenKind::Spawn => {
                self.advance();
                let call = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Spawn { call, span })
            }
            TokenKind::Skip => {
                self.advance();
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Skip { span })
            }
            _ => unreachable!(),
        }
    }
}
