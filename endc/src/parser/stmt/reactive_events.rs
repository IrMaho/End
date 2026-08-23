use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_reactive_events_statement(
        &mut self,
        peek_k: &TokenKind,
        span: &Span,
    ) -> Result<Option<Statement>, String> {
        let span = span.clone();
        match peek_k {
            TokenKind::Operation => {
                Ok(Some(Statement::OperationDecl(self.parse_operation(false)?)))
            }
            TokenKind::Event => {
                Ok(Some(Statement::EventDecl(self.parse_event(false)?)))
            }
            TokenKind::Hub => {
                Ok(Some(Statement::EventHubDecl(self.parse_event_hub(false)?)))
            }
            TokenKind::Observe => {
                self.advance();
                if self.check(&TokenKind::LBracket) {
                    let metrics = self.parse_string_list()?;
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Some(Statement::Observe { metrics, span }));
                }
                let mut op_expr = self.parse_expression()?;
                let mut alias = "trace".to_string();
                if let Expression::Cast { expr, target_type, .. } = op_expr {
                    op_expr = *expr;
                    alias = target_type.to_string();
                } else if self.match_token(&TokenKind::As) {
                    alias = self.parse_identifier_or_keyword()?;
                }
                while self.match_token(&TokenKind::Comma) {
                    let _ = self.parse_expression()?;
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ObserveOp { op_expr, alias, span }))
            }
            TokenKind::On => {
                self.advance();
                let mut event_pattern = self.parse_identifier_or_keyword()?;
                while self.match_token(&TokenKind::Dot) {
                    event_pattern.push('.');
                    event_pattern.push_str(&self.parse_identifier_or_keyword()?);
                }
                let mut guard = None;
                let mut filter = None;
                let mut projection = None;

                if self.match_token(&TokenKind::When) {
                    guard = Some(self.parse_expression()?);
                }
                if self.match_token(&TokenKind::Where) {
                    filter = Some(self.parse_expression()?);
                }
                if self.match_token(&TokenKind::FatArrow) {
                    projection = Some(self.parse_identifier_or_keyword_or_int()?);
                }

                let body = self.parse_block()?;
                Ok(Some(Statement::OnEventStmt(OnEventDef {
                    event_pattern,
                    guard,
                    filter,
                    projection,
                    body,
                    directives: vec![],
                    span,
                })))
            }
            TokenKind::Emit => {
                self.advance();
                let event_name = self.parse_identifier_or_keyword()?;
                let mut args = Vec::new();
                if self.match_token(&TokenKind::LParen) {
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                } else if !self.check(&TokenKind::SemiColon) {
                    args.push(self.parse_expression()?);
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::EmitEvent {
                    event_name,
                    args,
                    span,
                }))
            }
            TokenKind::Once => {
                self.advance();
                let mut event_pattern = self.parse_identifier_or_keyword()?;
                while self.match_token(&TokenKind::Dot) {
                    event_pattern.push('.');
                    event_pattern.push_str(&self.parse_identifier_or_keyword()?);
                }
                let body = self.parse_block()?;
                Ok(Some(Statement::OnceEventStmt(OnceEventDef {
                    event_pattern,
                    body,
                    span,
                })))
            }
            TokenKind::Every => {
                self.advance();
                let mut interval_str = String::new();
                if let TokenKind::IntLit(n) = self.peek_kind() {
                    interval_str.push_str(&n.to_string());
                    self.advance();
                    if let TokenKind::Ident(unit) = self.peek_kind() {
                        interval_str.push_str(unit);
                        self.advance();
                    }
                } else {
                    interval_str = self.parse_identifier_or_keyword_or_int()?;
                }
                let body = self.parse_block()?;
                Ok(Some(Statement::EveryEventStmt(EveryEventDef {
                    interval_str,
                    body,
                    span,
                })))
            }
            TokenKind::After => {
                self.advance();
                let mut delay_str = String::new();
                if let TokenKind::IntLit(n) = self.peek_kind() {
                    delay_str.push_str(&n.to_string());
                    self.advance();
                    if let TokenKind::Ident(unit) = self.peek_kind() {
                        delay_str.push_str(unit);
                        self.advance();
                    }
                } else {
                    delay_str = self.parse_identifier_or_keyword_or_int()?;
                }
                let body = self.parse_block()?;
                Ok(Some(Statement::AfterEventStmt(AfterEventDef {
                    delay_str,
                    body,
                    span,
                })))
            }
            TokenKind::Before => {
                self.advance();
                let mut event_pattern = self.parse_identifier_or_keyword()?;
                while self.match_token(&TokenKind::Dot) {
                    event_pattern.push('.');
                    event_pattern.push_str(&self.parse_identifier_or_keyword()?);
                }
                let body = self.parse_block()?;
                Ok(Some(Statement::BeforeEventStmt(BeforeEventDef {
                    event_pattern,
                    body,
                    span,
                })))
            }
            TokenKind::Watch => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let mut event = "mutate".to_string();
                if self.match_token(&TokenKind::On) {
                    event = self.parse_identifier_or_keyword()?;
                }
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    if self.match_token(&TokenKind::On) {
                        event = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::FatArrow);
                        let handler = self.parse_block()?;
                        self.expect(TokenKind::RBrace)?;
                        Ok(Some(Statement::WatchBlock { target, event, handler, span }))
                    } else {
                        let mut stmts = Vec::new();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            stmts.push(self.parse_statement()?);
                        }
                        self.expect(TokenKind::RBrace)?;
                        let handler = Block { statements: stmts, span: span.clone() };
                        Ok(Some(Statement::WatchBlock { target, event, handler, span }))
                    }
                } else {
                    let handler = self.parse_block()?;
                    Ok(Some(Statement::WatchBlock { target, event, handler, span }))
                }
            }
            TokenKind::React => {
                self.advance();
                self.match_token(&TokenKind::To);
                let event = if self.check(&TokenKind::LBrace) {
                    Expression::Ident("default".to_string(), self.current_span())
                } else if self.check(&TokenKind::LParen) {
                    self.parse_expression()?
                } else if let TokenKind::Ident(id) = self.peek_kind() {
                    let id_str = id.clone();
                    let id_span = self.current_span();
                    self.advance();
                    if self.match_token(&TokenKind::EqualEqual) || self.match_token(&TokenKind::Equal) {
                        let rhs = self.parse_expression()?;
                        Expression::Binary {
                            left: Box::new(Expression::Ident(id_str, id_span.clone())),
                            op: BinaryOp::Equal,
                            right: Box::new(rhs),
                            span: id_span,
                        }
                    } else if self.match_token(&TokenKind::Greater) {
                        let rhs = self.parse_expression()?;
                        Expression::Binary {
                            left: Box::new(Expression::Ident(id_str, id_span.clone())),
                            op: BinaryOp::GreaterThan,
                            right: Box::new(rhs),
                            span: id_span,
                        }
                    } else if self.match_token(&TokenKind::Less) {
                        let rhs = self.parse_expression()?;
                        Expression::Binary {
                            left: Box::new(Expression::Ident(id_str, id_span.clone())),
                            op: BinaryOp::LessThan,
                            right: Box::new(rhs),
                            span: id_span,
                        }
                    } else if self.match_token(&TokenKind::GreaterEqual) {
                        let rhs = self.parse_expression()?;
                        Expression::Binary {
                            left: Box::new(Expression::Ident(id_str, id_span.clone())),
                            op: BinaryOp::GreaterEqual,
                            right: Box::new(rhs),
                            span: id_span,
                        }
                    } else if self.match_token(&TokenKind::LessEqual) {
                        let rhs = self.parse_expression()?;
                        Expression::Binary {
                            left: Box::new(Expression::Ident(id_str, id_span.clone())),
                            op: BinaryOp::LessEqual,
                            right: Box::new(rhs),
                            span: id_span,
                        }
                    } else if self.match_token(&TokenKind::BangEqual) {
                        let rhs = self.parse_expression()?;
                        Expression::Binary {
                            left: Box::new(Expression::Ident(id_str, id_span.clone())),
                            op: BinaryOp::NotEqual,
                            right: Box::new(rhs),
                            span: id_span,
                        }
                    } else {
                        Expression::Ident(id_str, id_span)
                    }
                } else {
                    self.parse_expression()?
                };
                let handler = self.parse_block()?;
                Ok(Some(Statement::ReactBlock { event, handler, span }))
            }
            TokenKind::Stream => {
                self.advance();
                let source = match self.peek_kind() {
                    TokenKind::IntLit(n) => {
                        let v = *n;
                        self.advance();
                        Expression::Lit(Literal::Int(v), self.current_span())
                    }
                    _ => {
                        let source_name = self.parse_identifier_or_keyword()?;
                        Expression::Ident(source_name, span.clone())
                    }
                };
                let mut operations = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Val) {
                            let stmt = self.parse_statement()?;
                            operations.push(Expression::Ident(format!("{:?}", stmt), stmt.span().clone()));
                        } else {
                            operations.push(self.parse_expression()?);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                        operations.push(self.parse_expression()?);
                        self.match_token(&TokenKind::Comma);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Some(Statement::StreamBlock { source, operations, span }))
            }
            TokenKind::Flow => {
                self.advance();
                let mut steps = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Val) {
                            let stmt = self.parse_statement()?;
                            steps.push(Expression::Ident(format!("{:?}", stmt), stmt.span().clone()));
                        } else {
                            steps.push(self.parse_expression()?);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                        steps.push(self.parse_expression()?);
                        self.match_token(&TokenKind::Comma);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Some(Statement::FlowBlock { steps, span }))
            }
            TokenKind::Race => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut branches = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    if self.check(&TokenKind::LBrace) {
                        branches.push(self.parse_block()?);
                    } else {
                        let stmt = self.parse_statement()?;
                        branches.push(Block { statements: vec![stmt], span: self.current_span() });
                    }
                    self.match_token(&TokenKind::Comma);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Some(Statement::RaceBlock { branches, span }))
            }
            TokenKind::Hedge => {
                self.advance();
                let mut delay_ms = Expression::Lit(Literal::Int(20), span.clone());
                if self.peek_kind() != &TokenKind::LBrace {
                    let _ = self.parse_identifier_or_keyword().ok();
                    delay_ms = self.parse_expression()?;
                    if matches!(self.peek_kind(), TokenKind::Ident(id) if id == "ms" || id == "s" || id == "us") {
                        self.advance();
                    }
                }
                let primary = self.parse_block()?;
                let mut fallback = Block { statements: vec![], span: span.clone() };
                if self.match_token(&TokenKind::Fallback) {
                    fallback = self.parse_block()?;
                }
                Ok(Some(Statement::HedgeBlock { delay_ms, primary, fallback, span }))
            }
            TokenKind::CancelSafe => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Some(Statement::CancelSafeBlock { body, span }))
            }
            _ => Ok(None),
        }
    }
}
