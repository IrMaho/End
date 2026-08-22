use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_refactoring_ops_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Boundary
            | TokenKind::Responsibility
            | TokenKind::Owns
            | TokenKind::Exposes
            | TokenKind::DependsOnly
            | TokenKind::Depends
            | TokenKind::Forbid
            | TokenKind::Layer
            | TokenKind::Direction
            | TokenKind::Split
            | TokenKind::Partition
            | TokenKind::Extract
            | TokenKind::Cluster
            | TokenKind::Separate
            | TokenKind::Merge
            | TokenKind::Inline
            | TokenKind::Contract
            | TokenKind::Port
            | TokenKind::Adapter
            | TokenKind::Facade
            | TokenKind::Gateway => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_refactoring_ops_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_refactoring_ops_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Boundary => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut allows = Vec::new();
                let mut denies = Vec::new();
                let mut is_sealed = false;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        if key == "allow" {
                            allows.push(self.parse_identifier_or_string()?);
                        } else if key == "deny" {
                            denies.push(self.parse_identifier_or_string()?);
                        } else if key == "sealed" {
                            is_sealed = true;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else if self.match_token(&TokenKind::Sealed) {
                    is_sealed = true;
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::BoundaryDecl { name, allows, denies, is_sealed, span })
            }
            TokenKind::Responsibility => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let description = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ResponsibilityDecl { module_name: "".to_string(), description, span })
            }
            TokenKind::Owns => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let symbols = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::OwnsDecl { module_name: "".to_string(), symbols, span })
            }
            TokenKind::Exposes => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let symbols = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ExposesDecl { module_name: "".to_string(), symbols, span })
            }
            TokenKind::DependsOnly => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let target_module = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?.join(", ")
                } else {
                    self.parse_identifier_or_string()?
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DependsDecl { from_module: "".to_string(), target_module, is_only: true, span })
            }
            TokenKind::Depends => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let target_module = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DependsDecl { from_module: "".to_string(), target_module, is_only: false, span })
            }
            TokenKind::Forbid => {
                self.advance();
                let from = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Arrow);
                let to = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ForbidDecl { from, to, span })
            }
            TokenKind::Layer => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut forbid_depends = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.match_token(&TokenKind::Forbid) {
                            if self.match_token(&TokenKind::Depends) {
                                // consumed depends
                            }
                            forbid_depends.push(self.parse_identifier_or_keyword()?);
                        } else {
                            let _ = self.advance();
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::LayerDecl { name, forbid_depends, span })
            }
            TokenKind::Direction => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let from = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Arrow);
                let to = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DirectionDecl { from, to, span })
            }
            TokenKind::Split => {
                self.advance();
                let mut is_op = false;
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                    is_op = true;
                }
                let entity = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Into);
                let parts = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                if is_op {
                    Ok(Statement::SplitOpDecl { op_name: entity, sub_ops: parts, span })
                } else {
                    Ok(Statement::SplitDecl { entity, parts, span })
                }
            }
            TokenKind::Partition => {
                self.advance();
                let entity = self.parse_identifier_or_keyword()?;
                let mut by = "responsibility".to_string();
                if self.match_token(&TokenKind::By) {
                    by = self.parse_identifier_or_keyword()?;
                }
                let parts = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::PartitionDecl { entity, by, parts, span })
            }
            TokenKind::Extract => {
                self.advance();
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                    let op_name = self.parse_identifier_or_keyword()?;
                    let mut from_mod = String::new();
                    let mut condition = String::new();
                    if self.match_token(&TokenKind::From) {
                        from_mod = self.parse_identifier_or_keyword()?;
                    }
                    if self.match_token(&TokenKind::Into) || self.match_token(&TokenKind::To) {
                        let _into_mod = self.parse_identifier_or_keyword()?;
                    }
                    if self.match_token(&TokenKind::Where) || self.match_token(&TokenKind::When) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "where" } else { false }) {
                        if self.peek_kind() == &TokenKind::Ident("where".to_string()) {
                            self.advance();
                        }
                        condition = self.parse_identifier_or_string()?;
                        if self.match_token(&TokenKind::EqualEqual) || self.match_token(&TokenKind::Equal) {
                            condition.push_str(" == ");
                            condition.push_str(&self.parse_identifier_or_string()?);
                        }
                    }
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ExtractOpDecl { op_name, from_mod, condition, span });
                }
                if self.check(&TokenKind::LBracket) {
                    let symbols = self.parse_string_list()?;
                    if !self.match_token(&TokenKind::To) {
                        self.match_token(&TokenKind::Into);
                    }
                    let into_module = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ExtractDecl { symbols, into_module, span });
                }
                self.expect(TokenKind::LBrace)?;
                let mut symbols = Vec::new();
                let mut into_module = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "symbols" || key == "symbol" {
                        symbols = self.parse_string_list()?;
                    } else if key == "into" || key == "to" {
                        into_module = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ExtractDecl { symbols, into_module, span })
            }
            TokenKind::Cluster => {
                self.advance();
                let mut by = "semantic".to_string();
                if self.match_token(&TokenKind::By) {
                    by = self.parse_identifier_or_keyword()?;
                }
                if !self.check(&TokenKind::LBrace) {
                    let predicate = self.parse_identifier_or_string()?;
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ClusterDecl { by, predicate, span });
                }
                self.expect(TokenKind::LBrace)?;
                let mut predicate = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let s = self.parse_identifier_or_string()?;
                    if !predicate.is_empty() { predicate.push(' '); }
                    predicate.push_str(&s);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ClusterDecl { by, predicate, span })
            }
            TokenKind::Separate => {
                self.advance();
                let left = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::From);
                let right = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::SeparateDecl { left, right, span })
            }
            TokenKind::Merge => {
                self.advance();
                let source_ops = self.parse_string_list()?;
                let mut as_name = String::new();
                if self.match_token(&TokenKind::As) {
                    as_name = self.parse_identifier_or_keyword()?;
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::MergeOpDecl { source_ops, as_name, span })
            }
            TokenKind::Inline => {
                self.advance();
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                }
                let op_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::InlineOpDecl { op_name, span })
            }
            TokenKind::Contract => {
                let ctr = self.parse_contract_def(false)?;
                Ok(Statement::ContractDefinition(ctr))
            }
            TokenKind::Port => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut methods = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    if self.check(&TokenKind::Fn) { self.advance(); }
                    let m = self.parse_identifier_or_keyword()?;
                    if self.match_token(&TokenKind::LParen) {
                        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                            let _ = self.parse_identifier_or_keyword();
                            if self.match_token(&TokenKind::Colon) {
                                let _ = self.parse_type();
                            }
                            self.match_token(&TokenKind::Comma);
                        }
                        self.expect(TokenKind::RParen)?;
                    }
                    if self.match_token(&TokenKind::Arrow) {
                        let _ = self.parse_type();
                    }
                    methods.push(m);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::PortDecl { name, methods, span })
            }
            TokenKind::Adapter => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut port = String::new();
                if self.match_token(&TokenKind::For) || self.match_token(&TokenKind::Impl) || self.peek_kind() != &TokenKind::LBrace {
                    port = self.parse_identifier_or_keyword().unwrap_or_default();
                }
                let body = self.parse_block()?;
                Ok(Statement::AdapterDecl { name, port, body, span })
            }
            TokenKind::Facade => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut exposes = Vec::new();
                if self.match_token(&TokenKind::Exposes) || self.match_token(&TokenKind::Expose) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "exposes" || s == "expose" } else { false }) {
                    if let TokenKind::Ident(_) = self.peek_kind() { self.advance(); }
                    if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                        exposes = self.parse_string_list()?;
                    } else {
                        exposes.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                } else if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "expose" || key == "exposes" {
                            if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                                let mut list = self.parse_string_list()?;
                                exposes.append(&mut list);
                            } else {
                                exposes.push(self.parse_identifier_or_string()?);
                            }
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                Ok(Statement::FacadeDecl { name, exposes, span })
            }
            TokenKind::Gateway => {
                self.advance();
                let from_mod = self.parse_identifier_or_keyword()?;
                if self.match_token(&TokenKind::From) {
                    let _ = self.parse_identifier_or_keyword();
                }
                self.match_token(&TokenKind::To);
                self.match_token(&TokenKind::Arrow);
                let to_mod = self.parse_identifier_or_keyword()?;
                let mut allowed_calls = Vec::new();
                if self.peek_kind() == &TokenKind::Ident("allowed_calls".to_string()) || self.match_token(&TokenKind::Allow) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "allows" } else { false }) {
                    if let TokenKind::Ident(_) = self.peek_kind() {
                        self.advance();
                    }
                    if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                        allowed_calls = self.parse_string_list()?;
                    } else {
                        allowed_calls.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                } else if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "allow" || key == "allows" || key == "allowed_calls" {
                            if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                                let mut list = self.parse_string_list()?;
                                allowed_calls.append(&mut list);
                            } else {
                                allowed_calls.push(self.parse_identifier_or_string()?);
                            }
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                Ok(Statement::GatewayDecl { from_mod, to_mod, allowed_calls, span })
            }
            _ => unreachable!(),
        }
    }
}
