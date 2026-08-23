use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_agent_workflow_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Agent
            | TokenKind::Task
            | TokenKind::Accept
            | TokenKind::Reject
            | TokenKind::Baseline
            | TokenKind::Regression
            | TokenKind::Explain
            | TokenKind::Context
            | TokenKind::Slice
            | TokenKind::Patch
            | TokenKind::Evolve => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_agent_workflow_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_agent_workflow_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Agent => {
                self.advance();
                if self.check(&TokenKind::Lease) {
                    self.advance();
                    let module_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::LBrace)?;
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
                    return Ok(Statement::AgentLeaseDecl { module_name, owner, duration, span });
                }
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut capabilities = Vec::new();
                    let mut cannot = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        let list = self.parse_string_list()?;
                        if key == "capabilities" || key == "capability" {
                            capabilities.extend(list);
                        } else if key == "cannot" {
                            cannot.extend(list);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::AgentCapabilityDecl { capabilities, cannot, span });
                }
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut scope = String::new();
                let mut goal = String::new();
                let mut constraints = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "scope" {
                        scope = self.parse_identifier_or_string()?;
                    } else if key == "goal" {
                        goal = self.parse_identifier_or_string()?;
                    } else if key == "constraints" {
                        constraints = self.parse_string_list()?;
                    } else {
                        let _ = self.parse_identifier_or_string();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Statement::AgentContract { name, scope, goal, constraints, body, span })
            }
            TokenKind::Task => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut owner = None;
                    let mut status = None;
                    let mut requirement = None;
                    let mut implementation = None;
                    let mut skills = Vec::new();
                    let mut change_budget = Vec::new();
                    let mut evidence = Vec::new();
                    let mut body_stmts = Vec::new();

                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Evidence) {
                            self.advance();
                            self.expect(TokenKind::LBrace)?;
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                let ek = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::Colon);
                                let ev = self.parse_identifier_or_string()?;
                                evidence.push((ek, ev));
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                            continue;
                        }

                        if self.check(&TokenKind::Target) {
                            self.advance();
                            let target_val = self.parse_identifier_or_string()?;
                            self.match_token(&TokenKind::SemiColon);
                            implementation = Some(target_val);
                            continue;
                        }

                        if self.check(&TokenKind::Requires) || self.check(&TokenKind::Require) {
                            self.advance();
                            if self.check(&TokenKind::LBrace) {
                                self.advance();
                                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                    let req_name = self.parse_identifier_or_string()?;
                                    skills.push(req_name);
                                    self.match_token(&TokenKind::Comma);
                                    self.match_token(&TokenKind::SemiColon);
                                }
                                self.expect(TokenKind::RBrace)?;
                            } else {
                                let req_name = self.parse_identifier_or_string()?;
                                self.match_token(&TokenKind::SemiColon);
                                skills.push(req_name);
                            }
                            continue;
                        }

                        let is_spec = self.check(&TokenKind::Ident("owner".to_string()))
                            || self.check(&TokenKind::Ident("status".to_string()))
                            || self.check(&TokenKind::Requirement)
                            || self.check(&TokenKind::Ident("implementation".to_string()))
                            || self.check(&TokenKind::Skills)
                            || self.check(&TokenKind::Ident("change_budget".to_string()));

                        if is_spec {
                            let key = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            if key == "owner" {
                                owner = Some(self.parse_identifier_or_string()?);
                            } else if key == "status" {
                                status = Some(self.parse_identifier_or_string()?);
                            } else if key == "requirement" || key == "requirements" {
                                requirement = Some(self.parse_identifier_or_string()?);
                            } else if key == "implementation" {
                                implementation = Some(self.parse_identifier_or_string()?);
                            } else if key == "skills" || key == "skill" {
                                skills = self.parse_string_list()?;
                            } else if key == "change_budget" {
                                change_budget = self.parse_string_list()?;
                            }
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else {
                            if let Ok(stmt) = self.parse_statement() {
                                body_stmts.push(stmt);
                            } else {
                                self.advance();
                            }
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    if owner.is_some() || status.is_some() || requirement.is_some() || !skills.is_empty() || !evidence.is_empty() {
                        Ok(Statement::AgentTaskContractDecl {
                            name, owner, status, requirement, implementation, skills, change_budget, evidence, span
                        })
                    } else {
                        Ok(Statement::TaskDecl { name, body: Block { statements: body_stmts, span: span.clone() }, span })
                    }
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::TaskDecl { name, body: Block { statements: Vec::new(), span: span.clone() }, span })
                }
            }
            TokenKind::Accept => {
                self.advance();
                let conditions = self.parse_string_list()?;
                Ok(Statement::AcceptBlock { conditions, span })
            }
            TokenKind::Reject => {
                self.advance();
                if self.check(&TokenKind::If) {
                    self.advance();
                }
                let conditions = self.parse_string_list()?;
                Ok(Statement::RejectBlock { conditions, span })
            }
            TokenKind::Baseline => {
                self.advance();
                let metrics = self.parse_key_value_pairs()?;
                Ok(Statement::BaselineBlock { metrics, span })
            }
            TokenKind::Regression => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let cond = self.parse_identifier_or_string()?;
                    self.expect(TokenKind::RBrace)?;
                    cond
                } else {
                    let cond = self.parse_identifier_or_string()?;
                    self.match_token(&TokenKind::SemiColon);
                    cond
                };
                Ok(Statement::RegressionCheck { condition, span })
            }
            TokenKind::Explain => {
                self.advance();
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                    let op_name = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ExplainOpDecl { op_name, span });
                }
                let mut topic = "general".to_string();
                let mut rationale = String::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let s = self.parse_identifier_or_string()?;
                        if self.match_token(&TokenKind::Colon) {
                            topic = s;
                            rationale = self.parse_identifier_or_string()?;
                        } else {
                            rationale = s;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    rationale = self.parse_identifier_or_string()?;
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::ExplainBlock { topic, rationale, span })
            }
            TokenKind::Context => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut includes = Vec::new();
                let mut excludes = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    let list = self.parse_string_list()?;
                    if key == "include" || key == "includes" {
                        includes = list;
                    } else if key == "exclude" || key == "excludes" {
                        excludes = list;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Statement::ContextBlock { name, includes, excludes, body, span })
            }
            TokenKind::Slice => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut from_target = String::new();
                let mut includes = Vec::new();
                let mut excludes = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "from" {
                        from_target = self.parse_identifier_or_string()?;
                    } else if key == "include" || key == "includes" {
                        includes = self.parse_string_list()?;
                    } else if key == "exclude" || key == "excludes" {
                        excludes = self.parse_string_list()?;
                    } else {
                        let _ = self.parse_identifier_or_string();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::SliceDecl { name, from_target, includes, excludes, span })
            }
            TokenKind::Patch => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let body = self.parse_block()?;
                Ok(Statement::PatchDecl { target, body, span })
            }
            TokenKind::Evolve => {
                self.advance();
                let target = if self.check(&TokenKind::LBrace) {
                    "self".to_string()
                } else {
                    self.parse_identifier_or_keyword()?
                };
                if target == "event_topology" {
                    let top_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::LBrace)?;
                    let mut preserve = Vec::new();
                    let mut optimize = Vec::new();
                    let mut allow = Vec::new();
                    let mut reject = Vec::new();

                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        let val = self.parse_identifier_or_keyword_or_int().unwrap_or_default();
                        self.match_token(&TokenKind::SemiColon);
                        if key == "add" {
                            allow.push(val);
                        } else if key == "remove" {
                            reject.push(val);
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::EvolveOpDecl {
                        op_name: top_name,
                        preserve,
                        optimize,
                        allow,
                        reject,
                        span,
                    });
                }
                if target == "operation" {
                    let op_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::LBrace)?;
                    let mut preserve = Vec::new();
                    let mut optimize = Vec::new();
                    let mut allow = Vec::new();
                    let mut reject = Vec::new();

                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "preserve" {
                            let mut p = self.parse_string_list()?;
                            preserve.append(&mut p);
                        } else if key == "optimize" {
                            let mut opt = self.parse_string_list()?;
                            optimize.append(&mut opt);
                        } else if key == "allow" {
                            let mut a = self.parse_string_list()?;
                            allow.append(&mut a);
                        } else if key == "reject" || key == "reject_if" {
                            let mut r = self.parse_string_list()?;
                            reject.append(&mut r);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::EvolveOpDecl { op_name, preserve, optimize, allow, reject, span });
                }
                if target == "architecture" {
                    self.expect(TokenKind::LBrace)?;
                    let mut from = String::new();
                    let mut toward = String::new();
                    let mut target_modules = 25;
                    let mut preserve = Vec::new();
                    let mut optimize = Vec::new();
                    let mut reject_if = Vec::new();
                    let mut verify = Vec::new();

                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "from" {
                            from = self.parse_identifier_or_string()?;
                            if self.match_token(&TokenKind::Toward) {
                                toward = self.parse_identifier_or_string()?;
                            }
                        } else if key == "toward" {
                            toward = self.parse_identifier_or_string()?;
                        } else if key == "target" || key == "target_modules" || key == "modules" {
                            if let TokenKind::IntLit(i) = self.peek_kind() {
                                target_modules = *i as usize;
                                self.advance();
                            }
                        } else if key == "preserve" {
                            let mut p = self.parse_string_list()?;
                            preserve.append(&mut p);
                        } else if key == "optimize" {
                            let mut opt = self.parse_string_list()?;
                            optimize.append(&mut opt);
                        } else if key == "reject_if" || key == "reject" {
                            let mut r = self.parse_string_list()?;
                            reject_if.append(&mut r);
                        } else if key == "verify" {
                            let mut v = self.parse_string_list()?;
                            verify.append(&mut v);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::EvolveArchDecl { from, toward, target_modules, preserve, optimize, reject_if, verify, span });
                }

                self.expect(TokenKind::LBrace)?;
                let mut intent = String::new();
                let mut preserve = Vec::new();
                let mut budget = None;
                let mut allow = Vec::new();
                let mut reject = Vec::new();
                let mut verify = Vec::new();
                let mut accept = Vec::new();

                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    if self.match_token(&TokenKind::Colon) {
                        if key == "intent" {
                            intent = self.parse_identifier_or_string()?;
                        } else if key == "preserve" {
                            preserve = self.parse_string_list()?;
                        } else if key == "budget" {
                            budget = Some(self.parse_identifier_or_string()?);
                        } else if key == "allow" {
                            allow = self.parse_string_list()?;
                        } else if key == "reject" {
                            reject = self.parse_string_list()?;
                        } else if key == "verify" {
                            verify = self.parse_string_list()?;
                        } else if key == "accept" {
                            accept = self.parse_string_list()?;
                        } else {
                            let _ = self.parse_string_list();
                        }
                    } else {
                        let _ = self.parse_identifier_or_string();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Statement::EvolveBlock { target, intent, preserve, budget, allow, reject, verify, accept, body, span })
            }

            _ => unreachable!(),
        }
    }
}
