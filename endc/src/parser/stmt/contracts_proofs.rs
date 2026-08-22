use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_contracts_proofs_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Intent
            | TokenKind::Prove
            | TokenKind::Assume
            | TokenKind::Guarantee
            | TokenKind::Invariant
            | TokenKind::Verify
            | TokenKind::Because
            | TokenKind::Why
            | TokenKind::Protect
            | TokenKind::Frozen
            | TokenKind::MutableBy
            | TokenKind::Owned
            | TokenKind::Handoff
            | TokenKind::ReturnTo
            | TokenKind::Compute
            | TokenKind::RaceFree
            | TokenKind::Order
            | TokenKind::Deterministic
            | TokenKind::Replay
            | TokenKind::Checkpoint
            | TokenKind::Rollback
            | TokenKind::Transaction
            | TokenKind::Speculative
            | TokenKind::Fallback
            | TokenKind::Budget
            | TokenKind::Deadline
            | TokenKind::Priority
            | TokenKind::Quality
            | TokenKind::Tradeoff
            | TokenKind::Adapt => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_contracts_proofs_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_contracts_proofs_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Intent => {
                self.advance();
                if let TokenKind::Ident(ref id) = self.peek_kind().clone() {
                    if id == "diff" {
                        self.advance();
                        let mut preserve = Vec::new();
                        let mut change = Vec::new();
                        if self.match_token(&TokenKind::LBrace) {
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                let key = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::Colon);
                                let list = self.parse_string_list()?;
                                if key == "preserve" {
                                    preserve = list;
                                } else if key == "change" || key == "allow" {
                                    change = list;
                                }
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                                let key = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::Colon);
                                let list = self.parse_string_list()?;
                                if key == "preserve" {
                                    preserve = list;
                                } else if key == "change" || key == "allow" {
                                    change = list;
                                }
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.match_token(&TokenKind::SemiColon);
                        }
                        return Ok(Statement::IntentDiff { preserve, change, span });
                    }
                }

                let mut name = None;
                let mut goal = String::new();
                let mut preserve = Vec::new();
                let mut optimize = Vec::new();

                match self.peek_kind() {
                    TokenKind::StringLit(s) => {
                        goal = s.clone();
                        self.advance();
                    }
                    TokenKind::Ident(_) => {
                        name = Some(self.parse_identifier_or_keyword()?);
                    }
                    _ => {}
                }

                let mut body = None;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut stmts = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Goal) {
                            self.advance();
                            self.match_token(&TokenKind::Colon);
                            goal = self.parse_identifier_or_string()?;
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else if self.check(&TokenKind::Preserve) {
                            self.advance();
                            self.match_token(&TokenKind::Colon);
                            preserve = self.parse_string_list()?;
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else if self.check(&TokenKind::Optimize) {
                            self.advance();
                            self.match_token(&TokenKind::Colon);
                            optimize = self.parse_string_list()?;
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else {
                            if let Ok(stmt) = self.parse_statement() {
                                stmts.push(stmt);
                            } else {
                                let key = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::Colon);
                                if key == "goal" {
                                    goal = self.parse_identifier_or_string()?;
                                } else if key == "preserve" {
                                    preserve = self.parse_string_list()?;
                                } else if key == "optimize" {
                                    optimize = self.parse_string_list()?;
                                }
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    if !stmts.is_empty() {
                        body = Some(Block { statements: stmts, span: span.clone() });
                    }
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }

                if name.is_none() && body.is_none() && (!goal.is_empty() || !preserve.is_empty() || !optimize.is_empty()) {
                    Ok(Statement::IntentDecl { goal, preserve, optimize, span })
                } else {
                    Ok(Statement::Intent { name, goal, preserve, body, span })
                }
            }
            TokenKind::Prove => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Prove { condition, span })
            }
            TokenKind::Assume => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Assume { condition, span })
            }
            TokenKind::Guarantee => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Guarantee { condition, span })
            }
            TokenKind::Invariant => {
                self.advance();
                if let TokenKind::StringLit(s) = self.peek_kind() {
                    let s_val = s.clone();
                    self.advance();
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ArchInvariantDecl { rule: s_val, span });
                }
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Invariant { condition, span })
            }
            TokenKind::Verify => {
                self.advance();
                if self.check(&TokenKind::Adversarial) {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    let mut skill = None;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "skill" {
                            skill = Some(self.parse_identifier_or_string()?);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    Ok(Statement::VerifyTask { target: "adversarial".to_string(), is_adversarial: true, skill, span })
                } else if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut invariants = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        invariants.push(self.parse_expression()?);
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    Ok(Statement::VerifyBlock { invariants, span })
                } else {
                    let target = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::VerifyTask { target, is_adversarial: false, skill: None, span })
                }
            }
            TokenKind::Because => {
                self.advance();
                let rationale = match self.peek_kind() {
                    TokenKind::StringLit(s) => {
                        let r = s.clone();
                        self.advance();
                        r
                    }
                    _ => self.parse_identifier_or_keyword()?,
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Because { rationale, span })
            }
            TokenKind::Why => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let mut rationale = String::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if let TokenKind::StringLit(s) = self.peek_kind() {
                            rationale.push_str(s);
                            self.advance();
                        } else {
                            rationale.push_str(&self.parse_identifier_or_keyword()?);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else if self.match_token(&TokenKind::Colon) {
                    rationale = self.parse_identifier_or_string()?;
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::Why { target, rationale, span })
            }
            TokenKind::Protect => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::ProtectBlock { body, span })
            }
            TokenKind::Frozen => {
                self.advance();
                let symbol = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Frozen { symbol, span })
            }
            TokenKind::MutableBy => {
                self.advance();
                let mut roles = Vec::new();
                while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                    roles.push(self.parse_identifier_or_keyword()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::MutableBy { roles, span })
            }
            TokenKind::Owned => {
                self.advance();
                self.match_token(&TokenKind::Val);
                self.match_token(&TokenKind::Mut);
                let name = self.parse_identifier_or_keyword()?;
                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }
                self.expect(TokenKind::Equal)?;
                let initializer = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Owned { name, var_type, initializer, span })
            }
            TokenKind::Handoff => {
                self.advance();
                let resource = self.parse_identifier_or_keyword()?;
                if !self.match_token(&TokenKind::Arrow) {
                    self.match_token(&TokenKind::To);
                }
                let target_domain = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Handoff { resource, target_domain, span })
            }
            TokenKind::ReturnTo => {
                self.advance();
                let source_domain = self.parse_identifier_or_keyword()?;
                let resource = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ReturnTo { source_domain, resource, span })
            }
            TokenKind::Compute => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let body = self.parse_block()?;
                let mut fallback = None;
                if self.match_token(&TokenKind::Fallback) {
                    let _ = self.parse_identifier_or_keyword().ok();
                    fallback = Some(self.parse_block()?);
                }
                Ok(Statement::ComputeBlock { target, body, fallback, span })
            }
            TokenKind::RaceFree => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::RaceFreeBlock { body, span })
            }
            TokenKind::Order => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let mode = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Order { mode, span })
            }
            TokenKind::Deterministic => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::DeterministicBlock { body, span })
            }
            TokenKind::Replay => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::ReplayBlock { body, span })
            }
            TokenKind::Checkpoint => {
                self.advance();
                let state_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Checkpoint { state_name, span })
            }
            TokenKind::Rollback => {
                self.advance();
                self.match_token(&TokenKind::To);
                let checkpoint_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Rollback { checkpoint_name, span })
            }
            TokenKind::Transaction => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::TransactionBlock { body, span })
            }
            TokenKind::Speculative => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::SpeculativeBlock { body, span })
            }
            TokenKind::Fallback => {
                self.advance();
                let target = self.parse_identifier_or_keyword().unwrap_or_else(|_| "default".to_string());
                let body = self.parse_block()?;
                Ok(Statement::FallbackBlock { target, body, span })
            }
            TokenKind::Budget => {
                self.advance();
                let specs = self.parse_key_value_pairs()?;
                let mut body = None;
                if self.check(&TokenKind::LBrace) {
                    body = Some(self.parse_block()?);
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::BudgetBlock { specs, body, span })
            }
            TokenKind::Deadline => {
                self.advance();
                let duration = self.parse_identifier_or_string()?;
                let body = self.parse_block()?;
                Ok(Statement::DeadlineBlock { duration, body, span })
            }
            TokenKind::Priority => {
                self.advance();
                let level = self.parse_identifier_or_keyword()?;
                let body = self.parse_block()?;
                Ok(Statement::PriorityBlock { level, body, span })
            }
            TokenKind::Quality => {
                self.advance();
                let pairs = self.parse_key_value_pairs()?;
                let min_metric = pairs.iter().find(|(k, _)| k == "min" || k == "min_metric" || k == "min_accuracy").map(|(_, v)| v.clone()).unwrap_or_else(|| "1.0".to_string());
                let max_latency = pairs.iter().find(|(k, _)| k == "max_latency" || k == "latency").map(|(_, v)| v.clone()).unwrap_or_else(|| "16ms".to_string());
                let body = self.parse_block()?;
                Ok(Statement::QualityBlock { min_metric, max_latency, body, span })
            }
            TokenKind::Tradeoff => {
                self.advance();
                let pairs = self.parse_key_value_pairs()?;
                let prefer = pairs.iter().find(|(k, _)| k == "prefer").map(|(_, v)| v.clone()).unwrap_or_else(|| "latency".to_string());
                let sacrifice = pairs.iter().find(|(k, _)| k == "sacrifice").map(|(_, v)| v.clone()).unwrap_or_else(|| "memory".to_string());
                let body = self.parse_block()?;
                Ok(Statement::TradeoffBlock { prefer, sacrifice, body, span })
            }
            TokenKind::Adapt => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut branches = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    if self.match_token(&TokenKind::When) || self.match_token(&TokenKind::If) {
                        // consumed keyword
                    } else if let TokenKind::Ident(s) = self.peek_kind() {
                        if s == "when" || s == "if" {
                            self.advance();
                        }
                    }
                    let cond = self.parse_expression()?;
                    if self.match_token(&TokenKind::FatArrow) {}
                    let blk = self.parse_block()?;
                    branches.push((cond, blk));
                    self.match_token(&TokenKind::Comma);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AdaptBlock { branches, span })
            }
            _ => unreachable!(),
        }
    }
}
