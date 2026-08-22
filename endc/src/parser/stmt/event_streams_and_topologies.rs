use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_event_streams_and_topologies_statement(
        &mut self,
        peek_k: &TokenKind,
        span: &Span,
    ) -> Result<Option<Statement>, String> {
        let span = span.clone();
        match peek_k {
            TokenKind::State => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut val_type = None;
                if self.match_token(&TokenKind::Colon) {
                    val_type = Some(self.parse_type()?);
                }
                self.expect(TokenKind::Equal)?;
                let initial_val = self.parse_expression()?;
                let mut with_attributes = Vec::new();
                if self.match_token(&TokenKind::With) {
                    while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                        with_attributes.push(self.parse_identifier_or_keyword()?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ReactiveStateStmt(ReactiveStateDef {
                    name,
                    val_type,
                    initial_val,
                    with_attributes,
                    span,
                })))
            }
            TokenKind::Derive => {
                self.advance();
                let target_var = self.parse_identifier_or_keyword()?;
                let mut source_vars = Vec::new();
                if self.match_token(&TokenKind::From) {
                    while !self.check(&TokenKind::FatArrow) && !self.check(&TokenKind::Equal) && !self.check(&TokenKind::EOF) {
                        source_vars.push(self.parse_identifier_or_keyword()?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                }
                if !self.match_token(&TokenKind::FatArrow) {
                    self.match_token(&TokenKind::Equal);
                }
                let expr = self.parse_expression()?;
                let mut with_attributes = Vec::new();
                if self.match_token(&TokenKind::With) {
                    while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                        with_attributes.push(self.parse_identifier_or_keyword()?);
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::DeriveStmt(DeriveDef {
                    target_var,
                    source_vars,
                    expr,
                    with_attributes,
                    span,
                })))
            }
            TokenKind::Topology => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut nodes = Vec::new();
                let mut edges = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let mut prev_node = self.parse_identifier_or_keyword()?;
                    if !nodes.contains(&prev_node) { nodes.push(prev_node.clone()); }
                    while self.match_token(&TokenKind::Arrow) || self.match_token(&TokenKind::BiArrow) || self.match_token(&TokenKind::TildeBiArrow) {
                        let next_node = self.parse_identifier_or_keyword()?;
                        edges.push((prev_node.clone(), next_node.clone()));
                        if !nodes.contains(&next_node) { nodes.push(next_node.clone()); }
                        prev_node = next_node;
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Some(Statement::TopologyStmt(TopologyDef {
                    name,
                    nodes,
                    edges,
                    body: Block { statements: vec![], span: span.clone() },
                    span,
                })))
            }
            TokenKind::Debounce
            | TokenKind::Throttle
            | TokenKind::Sample
            | TokenKind::Coalesce
            | TokenKind::Window => {
                let op_tok = self.advance();
                let op_kind = match op_tok.kind {
                    TokenKind::Debounce => "debounce",
                    TokenKind::Throttle => "throttle",
                    TokenKind::Sample => "sample",
                    TokenKind::Coalesce => "coalesce",
                    TokenKind::Window => "window",
                    _ => "stream_op",
                }.to_string();
                let mut params = Vec::new();
                while !self.check(&TokenKind::On) && !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                    let p = self.parse_identifier_or_keyword_or_int().unwrap_or_else(|_| "0".to_string());
                    params.push(p);
                    self.match_token(&TokenKind::Comma);
                }
                let mut target = "stream".to_string();
                if self.match_token(&TokenKind::On) {
                    target = self.parse_identifier_or_keyword()?;
                }
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    None
                };
                Ok(Some(Statement::EventStreamOpStmt(EventStreamOpDef {
                    op_kind,
                    target,
                    params,
                    body,
                    span,
                })))
            }
            TokenKind::EventTransaction => {
                self.advance();
                let block = self.parse_block()?;
                let mut on_rollback = None;
                if self.match_token(&TokenKind::Rollback) {
                    on_rollback = Some(self.parse_block()?);
                }
                Ok(Some(Statement::EventTransactionStmt(EventTransactionDef {
                    statements: block.statements,
                    on_rollback,
                    span,
                })))
            }
            TokenKind::Ack
            | TokenKind::RequireAck
            | TokenKind::Replayable
            | TokenKind::Durable
            | TokenKind::EventSourced
            | TokenKind::Quarantine
            | TokenKind::Publish
            | TokenKind::Drain
            | TokenKind::Pause
            | TokenKind::Resume
            | TokenKind::CircuitBreaker
            | TokenKind::RetryPolicy
            | TokenKind::DeadLetterQueue => {
                let tok = self.advance();
                let action = match tok.kind {
                    TokenKind::Ack => "ack",
                    TokenKind::RequireAck => "require_ack",
                    TokenKind::Replayable => "replayable",
                    TokenKind::Durable => "durable",
                    TokenKind::EventSourced => "event_sourced",
                    TokenKind::Quarantine => "quarantine",
                    TokenKind::Publish => "publish",
                    TokenKind::Drain => "drain",
                    TokenKind::Pause => "pause",
                    TokenKind::Resume => "resume",
                    TokenKind::CircuitBreaker => "circuit_breaker",
                    TokenKind::RetryPolicy => "retry_policy",
                    TokenKind::DeadLetterQueue => "dead_letter_queue",
                    _ => "control",
                }.to_string();
                let mut target = String::new();
                let mut args = Vec::new();

                while !self.check(&TokenKind::On) && !self.check(&TokenKind::To) && !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                    let a = self.parse_identifier_or_keyword_or_int().unwrap_or_else(|_| "1".to_string());
                    args.push(a);
                }

                if self.match_token(&TokenKind::On) || self.match_token(&TokenKind::To) {
                    target = self.parse_identifier_or_keyword()?;
                } else if !args.is_empty() {
                    target = args.remove(0);
                }

                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::EventControlStmt(EventControlDef {
                    action,
                    target,
                    args,
                    span,
                })))
            }
            _ => Ok(None),
        }
    }
}
