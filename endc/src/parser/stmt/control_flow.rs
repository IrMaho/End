use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_control_flow_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::ValBang
            | TokenKind::Guard
            | TokenKind::Replace
            | TokenKind::Decorate
            | TokenKind::Compose
            | TokenKind::Lease
            | TokenKind::Borrow => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_control_flow_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_control_flow_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Guard => {
                self.advance();
                let condition = self.parse_expression()?;
                self.expect(TokenKind::Else)?;
                let else_block = self.parse_block()?;
                Ok(Statement::Guard {
                    condition,
                    else_block,
                    span,
                })
            }
            TokenKind::ValBang => {
                self.advance();
                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected variable name after val!, found {:?}", other)),
                };
                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }
                self.expect(TokenKind::Equal)?;
                let expr = self.parse_expression()?;
                let fallback = if self.match_token(&TokenKind::QuestionQuestion) {
                    self.parse_expression()?
                } else {
                    Expression::Lit(Literal::Int(0), span.clone())
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::QuantumUnwrap {
                    name,
                    var_type,
                    expr,
                    fallback,
                    span,
                })
            }
            TokenKind::Replace => {
                self.advance();
                let target_kind = self.parse_identifier_or_keyword()?;
                if target_kind == "feature" {
                    let target = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::With);
                    let with_provider = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::ReplaceFeature { target, with_provider, span })
                } else if target_kind == "module" || target_kind == "mod" {
                    let target = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::With);
                    let replacement = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::ReplaceModuleDecl { target, replacement, span })
                } else {
                    let target = target_kind;
                    self.match_token(&TokenKind::With);
                    let replacement = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::ReplaceModuleDecl { target, replacement, span })
                }
            }
            TokenKind::Decorate => {
                self.advance();
                let _ = self.match_token(&TokenKind::Feature);
                let target = self.parse_identifier_or_keyword()?;
                let mut decorators = Vec::new();
                if self.match_token(&TokenKind::With) {
                    if self.check(&TokenKind::LBracket) {
                        decorators = self.parse_string_list()?;
                    } else {
                        decorators.push(self.parse_identifier_or_keyword()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                } else if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        decorators.push(self.parse_identifier_or_keyword()?);
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                Ok(Statement::DecorateFeature { target, decorators, span })
            }
            TokenKind::Compose => {
                self.advance();
                if self.check(&TokenKind::Feature) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "feature" } else { false }) {
                    self.advance();
                    let target = self.parse_identifier_or_keyword()?;
                    let mut components = Vec::new();
                    if self.match_token(&TokenKind::With) {
                        if self.check(&TokenKind::LBracket) {
                            components = self.parse_string_list()?;
                        } else {
                            components.push(self.parse_identifier_or_keyword()?);
                        }
                        self.match_token(&TokenKind::SemiColon);
                    } else if self.check(&TokenKind::LBrace) {
                        self.advance();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            components.push(self.parse_identifier_or_keyword()?);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                    }
                    Ok(Statement::ComposeFeature { target, components, span })
                } else {
                    let mut modules = Vec::new();
                    if self.check(&TokenKind::LBracket) {
                        modules = self.parse_string_list()?;
                    } else if self.check(&TokenKind::LBrace) {
                        self.advance();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            modules.push(self.parse_identifier_or_keyword()?);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                    }
                    Ok(Statement::ModuleComposeDecl { modules, span })
                }
            }
            TokenKind::Lease | TokenKind::Borrow => {
                self.advance();

                // lease cpu(cores, priority) { body }
                if let TokenKind::Ident(ref peek_id) = self.peek_kind().clone() {
                    if peek_id == "cpu" {
                        self.advance(); // consume "cpu"
                        self.expect(TokenKind::LParen)?;
                        let cores = self.parse_expression()?;
                        let priority = if self.match_token(&TokenKind::Comma) {
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };
                        self.expect(TokenKind::RParen)?;
                        let body = self.parse_block()?;
                        return Ok(Statement::LeaseCpu {
                            cores,
                            priority,
                            body,
                            span,
                        });
                    }

                    // lease listen(event_expr) while condition { body }
                    if peek_id == "listen" {
                        self.advance(); // consume "listen"
                        self.expect(TokenKind::LParen)?;
                        let event_expr = self.parse_expression()?;
                        self.expect(TokenKind::RParen)?;
                        let mut condition = None;
                        if self.match_token(&TokenKind::While) || self.match_token(&TokenKind::During) {
                            if !self.check(&TokenKind::LBrace) {
                                condition = Some(self.parse_expression()?);
                            }
                        }
                        let body = self.parse_block()?;
                        return Ok(Statement::LeaseEvent {
                            event_expr,
                            condition,
                            body,
                            span,
                        });
                    }

                    // lease loop(budget) for item in iterable { body }
                    if peek_id == "loop" {
                        self.advance(); // consume "loop"
                        self.expect(TokenKind::LParen)?;
                        let budget = self.parse_expression()?;
                        self.expect(TokenKind::RParen)?;
                        self.expect(TokenKind::For)?;
                        let item_name = self.parse_identifier_or_keyword()?;
                        if self.match_token(&TokenKind::Comma) {
                            let _ = self.parse_identifier_or_keyword()?;
                        }
                        self.expect(TokenKind::In)?;
                        let iterable = self.parse_expression()?;
                        let body = self.parse_block()?;
                        return Ok(Statement::LeaseLoop {
                            budget: Some(budget),
                            item_name,
                            iterable,
                            body,
                            span,
                        });
                    }
                }

                // lease for item in iterable { body }  (zero-allocation fused loop)
                if self.check(&TokenKind::For) {
                    self.advance(); // consume "for"
                    let item_name = self.parse_identifier_or_keyword()?;
                    if self.match_token(&TokenKind::Comma) {
                        let _ = self.parse_identifier_or_keyword()?;
                    }
                    self.expect(TokenKind::In)?;
                    let iterable = self.parse_expression()?;
                    let body = self.parse_block()?;
                    return Ok(Statement::LeaseLoop {
                        budget: None,
                        item_name,
                        iterable,
                        body,
                        span,
                    });
                }

                // Existing: lease val name = expr { body } / lease val name = expr;
                let is_mut = if self.match_token(&TokenKind::Mut) {
                    true
                } else {
                    self.match_token(&TokenKind::Val);
                    false
                };

                let name = self.parse_identifier_or_keyword()?;

                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut owner = String::new();
                    let mut duration = "task".to_string();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "owner" {
                            owner = self.parse_identifier_or_string()?;
                        } else if key == "duration" {
                            duration = self.parse_identifier_or_string()?;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::AgentLeaseDecl { module_name: name, owner, duration, span });
                }

                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }

                self.expect(TokenKind::Equal)?;
                let initializer = self.parse_expression()?;

                let mut condition = None;
                if self.match_token(&TokenKind::While) || self.match_token(&TokenKind::During) {
                    if !self.check(&TokenKind::LBrace) {
                        condition = Some(self.parse_expression()?);
                    }
                }

                if self.check(&TokenKind::LBrace) {
                    let body = self.parse_block()?;
                    Ok(Statement::LeaseBlock {
                        name,
                        var_type,
                        initializer,
                        condition,
                        body,
                        span,
                    })
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::VarDecl {
                        name,
                        var_type,
                        is_mut,
                        is_lease: true,
                        initializer: Some(initializer),
                        span,
                    })
                }
            }
            _ => unreachable!(),
        }
    }
}
