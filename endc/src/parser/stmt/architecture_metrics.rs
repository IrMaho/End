use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_architecture_metrics_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::CycleFree
            | TokenKind::MaxFanout
            | TokenKind::MaxFanin
            | TokenKind::MaxDepth
            | TokenKind::Cohesion
            | TokenKind::Modularize
            | TokenKind::Decompose
            | TokenKind::Architecture
            | TokenKind::Repair
            | TokenKind::Gravity
            | TokenKind::Bridge
            | TokenKind::Analyze
            | TokenKind::Feature
            | TokenKind::Skill => {}
            TokenKind::Ident(s) if s == "bridge" => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_architecture_metrics_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_architecture_metrics_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::CycleFree => {
                self.advance();
                self.match_token(&TokenKind::Equal);
                self.match_token(&TokenKind::Colon);
                let scope = if self.match_token(&TokenKind::True) || self.match_token(&TokenKind::False) {
                    "modules".to_string()
                } else {
                    self.parse_identifier_or_keyword().unwrap_or_else(|_| "modules".to_string())
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::CycleFreeDecl { scope, span })
            }
            TokenKind::MaxFanout => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::Colon);
                let mut limit = 5;
                if let TokenKind::IntLit(i) = self.peek_kind() {
                    limit = *i as usize;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::FanoutDecl { module_name, limit, span })
            }
            TokenKind::MaxFanin => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::Colon);
                let mut limit = 20;
                if let TokenKind::IntLit(i) = self.peek_kind() {
                    limit = *i as usize;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::FaninDecl { module_name, limit, span })
            }
            TokenKind::MaxDepth => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let mut limit = 6;
                if let TokenKind::IntLit(i) = self.peek_kind() {
                    limit = *i as usize;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DepthDecl { limit, span })
            }
            TokenKind::Cohesion => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::GreaterEqual);
                self.match_token(&TokenKind::Equal);
                self.match_token(&TokenKind::Colon);
                let mut min_threshold = 0.8;
                if let TokenKind::FloatLit(f) = self.peek_kind() {
                    min_threshold = *f;
                    self.advance();
                } else if let TokenKind::IntLit(i) = self.peek_kind() {
                    min_threshold = *i as f64;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::CohesionDecl { module_name, min_threshold, span })
            }
            TokenKind::Modularize => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut target_files_min = 5;
                let mut target_files_max = 20;
                let mut preserve = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "target_files" || key == "target" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            target_files_min = *i as usize;
                            target_files_max = *i as usize;
                            self.advance();
                            if self.match_token(&TokenKind::Dot) && self.match_token(&TokenKind::Dot) {
                                if let TokenKind::IntLit(i2) = self.peek_kind() {
                                    target_files_max = *i2 as usize;
                                    self.advance();
                                }
                            }
                        }
                    } else if key == "preserve" {
                        let mut p = self.parse_string_list()?;
                        preserve.append(&mut p);
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ModularizeDecl { target, target_files_min, target_files_max, preserve, span })
            }
            TokenKind::Decompose => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut target_modules = None;
                let mut optimize = Vec::new();
                let mut preserve = Vec::new();
                let mut gravity = None;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "target" || key == "target_modules" || key == "modules" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            target_modules = Some(*i as usize);
                            self.advance();
                            let _ = self.parse_identifier_or_keyword().ok();
                        }
                    } else if key == "optimize" {
                        let mut opt = self.parse_string_list()?;
                        optimize.append(&mut opt);
                    } else if key == "preserve" {
                        let mut p = self.parse_string_list()?;
                        preserve.append(&mut p);
                    } else if key == "gravity" {
                        gravity = Some(self.parse_identifier_or_string()?);
                    } else {
                        let _ = self.parse_string_list();
                    }
                self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::DecomposeDecl { target, target_modules, optimize, preserve, gravity, span })
            }
            TokenKind::Architecture => {
                self.parse_architecture_rule_or_template()
            }
            TokenKind::Repair => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::RepairDecl { target, span })
            }
            TokenKind::Gravity => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut weights = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let k = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    let mut w = 1.0;
                    if let TokenKind::IntLit(i) = self.peek_kind() {
                        w = *i as f64 / 100.0;
                        self.advance();
                        self.match_token(&TokenKind::Percent);
                    } else if let TokenKind::FloatLit(f) = self.peek_kind() {
                        w = *f;
                        self.advance();
                        self.match_token(&TokenKind::Percent);
                    }
                    weights.push((k, w));
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::GravityDecl { weights, span })
            }

            TokenKind::Bridge | TokenKind::Ident(_) if self.peek_kind() == &TokenKind::Bridge || (if let TokenKind::Ident(id) = self.peek_kind() { id == "bridge" } else { false }) => {
                self.advance();
                let from_mod = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Arrow);
                let to_mod = self.parse_identifier_or_keyword()?;
                let mut via = String::new();
                if self.peek_kind() == &TokenKind::Ident("via".to_string()) {
                    self.advance();
                    via = self.parse_identifier_or_keyword()?;
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::BridgeDecl { from_mod, to_mod, via, span })
            }
            TokenKind::Analyze => {
                self.advance();
                let op_expr = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::AnalyzeOp { op_expr, span })
            }
            TokenKind::Feature => {
                let feat = self.parse_feature_def(false, vec![])?;
                Ok(Statement::FeatureStatement(feat))
            }
            TokenKind::Skill => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut for_scope = None;
                if self.peek_kind() == &TokenKind::Ident("for".to_string()) || self.peek_kind() == &TokenKind::For {
                    self.advance();
                    for_scope = Some(self.parse_identifier_or_keyword()?);
                }
                let mut rules = Vec::new();
                let mut constraints = Vec::new();
                let mut structural = Vec::new();
                let mut semantic = Vec::new();
                let mut behavioral = Vec::new();
                let mut architectural = Vec::new();
                let mut performance = Vec::new();
                let mut security = Vec::new();
                let mut testing = Vec::new();
                let mut agent = Vec::new();
                let mut requires = Vec::new();
                let mut hard = Vec::new();
                let mut soft = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        let list = self.parse_string_list()?;
                        if key == "rules" || key == "rule" {
                            rules.extend(list);
                        } else if key == "constraints" || key == "constraint" {
                            constraints.extend(list);
                        } else if key == "structural" {
                            structural.extend(list);
                        } else if key == "semantic" {
                            semantic.extend(list);
                        } else if key == "behavioral" {
                            behavioral.extend(list);
                        } else if key == "architectural" {
                            architectural.extend(list);
                        } else if key == "performance" {
                            performance.extend(list);
                        } else if key == "security" {
                            security.extend(list);
                        } else if key == "testing" {
                            testing.extend(list);
                        } else if key == "agent" {
                            agent.extend(list);
                        } else if key == "requires" || key == "require" {
                            requires.extend(list);
                        } else if key == "hard" {
                            hard.extend(list);
                        } else if key == "soft" {
                            soft.extend(list);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::SkillDecl {
                    name, rules, constraints, structural, semantic, behavioral, architectural,
                    performance, security, testing, agent, requires, hard, soft, for_scope, span
                })
            }
            _ => unreachable!(),
        }
    }
}
