use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_governance_proposals_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Satisfies
            | TokenKind::Requirement
            | TokenKind::Implements
            | TokenKind::Verifies
            | TokenKind::Claim
            | TokenKind::Complete
            | TokenKind::Todo
            | TokenKind::Change
            | TokenKind::Review
            | TokenKind::Approval
            | TokenKind::Proposal
            | TokenKind::Knowledge
            | TokenKind::Decision
            | TokenKind::AgentBoundary
            | TokenKind::AgentContext
            | TokenKind::ContextFirewall
            | TokenKind::AgentApi
            | TokenKind::Agentability
            | TokenKind::RegressionGuard => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_governance_proposals_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_governance_proposals_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Satisfies => {
                self.advance();
                let entity = self.parse_identifier_or_keyword()?;
                let skills = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::SatisfiesDecl { entity, skills, span })
            }
            TokenKind::Requirement => {
                self.advance();
                let req_id = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::LBrace);
                let description = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::RBrace);
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::RequirementDecl { req_id, description, span })
            }
            TokenKind::Implements => {
                self.advance();
                let req_id = self.parse_identifier_or_keyword()?;
                let entities = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ImplementsDecl { req_id, entities, span })
            }
            TokenKind::Verifies => {
                self.advance();
                let req_id = self.parse_identifier_or_keyword()?;
                let entities = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::VerifiesDecl { req_id, entities, span })
            }
            TokenKind::Claim => {
                self.advance();
                if self.check(&TokenKind::Task) {
                    self.advance();
                }
                let task_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ClaimTask { task_name, span })
            }
            TokenKind::Complete => {
                self.advance();
                if self.check(&TokenKind::Task) {
                    self.advance();
                }
                let task_name = self.parse_identifier_or_keyword()?;
                let mut result = "success".to_string();
                let mut confidence = None;
                let mut summary = None;
                let mut evidence = Vec::new();
                let mut risks = None;
                let mut recommendation = None;
                let mut notes = None;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "result" {
                            result = self.parse_identifier_or_string()?;
                        } else if key == "confidence" {
                            if let TokenKind::FloatLit(f) = self.peek_kind() {
                                confidence = Some(*f);
                                self.advance();
                            } else if let TokenKind::IntLit(i) = self.peek_kind() {
                                confidence = Some(*i as f64);
                                self.advance();
                            }
                        } else if key == "summary" {
                            summary = Some(self.parse_identifier_or_string()?);
                        } else if key == "evidence" {
                            evidence = self.parse_string_list()?;
                        } else if key == "risks" || key == "risk" {
                            risks = Some(self.parse_identifier_or_string()?);
                        } else if key == "recommendation" {
                            recommendation = Some(self.parse_identifier_or_string()?);
                        } else if key == "notes" || key == "note" {
                            notes = Some(self.parse_identifier_or_string()?);
                        } else {
                            let _ = self.parse_string_list();
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::CompleteTask {
                    task_name, result, confidence, summary, evidence, risks, recommendation, notes, span
                })
            }
            TokenKind::Todo => {
                self.advance();
                let id = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut implement = String::new();
                let mut requires = Vec::new();
                let mut verify = Vec::new();
                let mut status = "planned".to_string();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "implement" {
                        implement = self.parse_identifier_or_string()?;
                    } else if key == "requires" || key == "require" {
                        requires = self.parse_string_list()?;
                    } else if key == "verify" {
                        verify = self.parse_string_list()?;
                    } else if key == "status" {
                        status = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::TodoDecl { id, implement, requires, verify, status, span })
            }
            TokenKind::Change => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut task = String::new();
                let mut intent = String::new();
                let mut satisfies = Vec::new();
                let mut evidence = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "task" {
                        task = self.parse_identifier_or_string()?;
                    } else if key == "intent" || key == "message" {
                        intent = self.parse_identifier_or_string()?;
                    } else if key == "satisfies" {
                        satisfies = self.parse_string_list()?;
                    } else if key == "evidence" {
                        evidence = self.parse_string_list()?;
                    } else {
                        let _ = self.parse_identifier_or_string().ok();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::SemanticCommitDecl { task, intent, satisfies, evidence, span })
            }
            TokenKind::Review => {
                self.advance();
                let task_id = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut summary = String::new();
                let mut completed = 0;
                let mut unresolved = 0;
                let mut risks = 0;
                let mut confidence = 1.0;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "summary" {
                        summary = self.parse_identifier_or_string()?;
                    } else if key == "completed" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            completed = *i as usize;
                            self.advance();
                        }
                    } else if key == "unresolved" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            unresolved = *i as usize;
                            self.advance();
                        }
                    } else if key == "risks" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            risks = *i as usize;
                            self.advance();
                        }
                    } else if key == "confidence" {
                        if let TokenKind::FloatLit(f) = self.peek_kind() {
                            confidence = *f;
                            self.advance();
                        } else if let TokenKind::IntLit(i) = self.peek_kind() {
                            confidence = *i as f64;
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentReviewDecl { task_id, summary, completed, unresolved, risks, confidence, span })
            }
            TokenKind::Approval => {
                self.advance();
                if self.peek_kind() == &TokenKind::Ident("required".to_string()) {
                    self.advance();
                }
                let required_items = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ApprovalDecl { required_items, span })
            }
            TokenKind::Proposal => {
                self.advance();
                let mut title = if !self.check(&TokenKind::LBrace) {
                    self.parse_identifier_or_string()?
                } else {
                    "proposal".to_string()
                };
                self.expect(TokenKind::LBrace)?;
                let mut files = Vec::new();
                let mut symbols = Vec::new();
                let mut dependencies = Vec::new();
                let mut risks = Vec::new();
                let mut migration = None;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "title" {
                        title = self.parse_identifier_or_string()?;
                    } else if key == "files" || key == "file" {
                        files = self.parse_string_list()?;
                    } else if key == "symbols" || key == "symbol" || key == "target" {
                        symbols = self.parse_string_list()?;
                    } else if key == "dependencies" || key == "deps" {
                        dependencies = self.parse_string_list()?;
                    } else if key == "risks" || key == "risk" {
                        risks = self.parse_string_list()?;
                    } else if key == "migration" || key == "proof" {
                        migration = Some(self.parse_identifier_or_string()?);
                    } else {
                        let _ = self.parse_identifier_or_string().ok();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentChangeProposalDecl {
                    title,
                    files,
                    symbols,
                    dependencies,
                    risks,
                    migration,
                    span,
                })
            }
            TokenKind::Knowledge => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut decisions = Vec::new();
                let mut constraints = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "decisions" || key == "decision" {
                        decisions = self.parse_string_list()?;
                    } else if key == "constraints" || key == "constraint" {
                        constraints = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::KnowledgeDecl { name, decisions, constraints, span })
            }
            TokenKind::Decision => {
                self.advance();
                let id = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut choose = String::new();
                let mut because = String::new();
                let mut reject = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "choose" {
                        choose = self.parse_identifier_or_string()?;
                    } else if key == "because" {
                        because = self.parse_identifier_or_string()?;
                    } else if key == "reject" {
                        reject = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::DecisionDecl { id, choose, because, reject, span })
            }
            TokenKind::AgentBoundary => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::AgentBoundaryDecl { module_name, span })
            }
            TokenKind::AgentContext => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut expose = Vec::new();
                let mut hide = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "expose" {
                        expose = self.parse_string_list()?;
                    } else if key == "hide" {
                        hide = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentContextDecl { module_name, expose, hide, span })
            }
            TokenKind::ContextFirewall => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut deny = Vec::new();
                let mut expose = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "deny" {
                        deny = self.parse_string_list()?;
                    } else if key == "expose" {
                        expose = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ContextFirewallDecl { module_name, deny, expose, span })
            }
            TokenKind::AgentApi => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut expose = Vec::new();
                let mut hide = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "expose" {
                        expose = self.parse_string_list()?;
                    } else if key == "hide" {
                        hide = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentApiDecl { module_name, expose, hide, span })
            }
            TokenKind::Agentability => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut max_context_tokens = 12000;
                let mut max_operation_complexity = "medium".to_string();
                let mut max_dependency_fanout = 8;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "max_context_tokens" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            max_context_tokens = *i as usize;
                            self.advance();
                        }
                    } else if key == "max_operation_complexity" {
                        max_operation_complexity = self.parse_identifier_or_string()?;
                    } else if key == "max_dependency_fanout" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            max_dependency_fanout = *i as usize;
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentabilityDecl { max_context_tokens, max_operation_complexity, max_dependency_fanout, span })
            }
            TokenKind::RegressionGuard => {
                self.advance();
                let items = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::RegressionGuardDecl { items, span })
            }
            _ => unreachable!(),
        }
    }
}
