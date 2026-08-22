use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_compiler_extensions_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::CompilerPlugin
            | TokenKind::Lint
            | TokenKind::Analyzer
            | TokenKind::TypeRule
            | TokenKind::Optimizer
            | TokenKind::BuildPlugin
            | TokenKind::Generator
            | TokenKind::Reflect
            | TokenKind::ArchitectureTest
            | TokenKind::Lock
            | TokenKind::AgentExtension
            | TokenKind::Proposal
            | TokenKind::Begin
            | TokenKind::Evolvable
            | TokenKind::Impact => {}
            TokenKind::Ident(s) if s == "change_limit" || s == "change_budget" => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_compiler_extensions_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_compiler_extensions_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::CompilerPlugin => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let kind = if self.check(&TokenKind::LBrace) {
                    let _ = self.parse_block();
                    "optimizer".to_string()
                } else {
                    self.parse_identifier_or_keyword().unwrap_or_else(|_| "optimizer".to_string())
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::CompilerPluginDecl { name, kind, span })
            }
            TokenKind::Lint => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut rules = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        rules.push(self.parse_identifier_or_string()?);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    rules.push(self.parse_identifier_or_string().unwrap_or_default());
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::CustomLinterDecl { name, rules, span })
            }
            TokenKind::Analyzer => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut checks = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        checks.push(self.parse_identifier_or_string()?);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    checks.push(self.parse_identifier_or_string().unwrap_or_default());
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::CustomAnalyzerDecl { name, checks, span })
            }
            TokenKind::TypeRule => {
                self.advance();
                let target_type = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut rules = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    rules.push(self.parse_identifier_or_string()?);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::CustomTypeRuleDecl { target_type, rules, span })
            }
            TokenKind::Optimizer => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut pass = "vectorize".to_string();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        pass = self.parse_identifier_or_string().unwrap_or(pass);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else if !self.check(&TokenKind::SemiColon) {
                    pass = self.parse_identifier_or_string().unwrap_or(pass);
                    self.match_token(&TokenKind::SemiColon);
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::CustomOptimizerDecl { name, pass, span })
            }
            TokenKind::BuildPlugin => {
                self.advance();
                let name = self.parse_identifier_or_keyword().unwrap_or_else(|_| "build_ext".to_string());
                if self.check(&TokenKind::LBrace) {
                    let _ = self.parse_block();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::BuildPluginDecl { name, hooks: vec!["pre_build".to_string(), "post_build".to_string()], span })
            }
            TokenKind::Generator => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut target_format = name.clone();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        target_format = self.parse_identifier_or_string().unwrap_or(target_format);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else if !self.check(&TokenKind::SemiColon) {
                    target_format = self.parse_identifier_or_string().unwrap_or(target_format);
                    self.match_token(&TokenKind::SemiColon);
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::GeneratorDecl { name, target_format, span })
            }
            TokenKind::Reflect => {
                self.advance();
                let target_type = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut queries = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    queries.push(self.parse_identifier_or_keyword()?);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ReflectDecl { target_type, queries, span })
            }
            TokenKind::Ident(s) if s == "change_limit" || s == "change_budget" => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut max_files = None;
                let mut max_modules = None;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if let TokenKind::IntLit(i) = self.peek_kind() {
                        let val = *i as usize;
                        self.advance();
                        if key == "files" || key == "max_files" {
                            max_files = Some(val);
                        } else if key == "modules" || key == "max_modules" {
                            max_modules = Some(val);
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ChangeBudgetDecl { max_files, max_modules, public_api_allowed: Some(false), span })
            }

            // Layer 6: Architecture as Code
            TokenKind::ArchitectureTest => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut assertions = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    assertions.push(self.parse_identifier_or_string()?);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ArchitectureTestDecl { assertions, span })
            }
            TokenKind::Lock => {
                self.advance();
                let _ = self.parse_identifier_or_keyword().ok();
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DependencyLockDecl { locked: true, span })
            }

            // Layer 9: Agent-Native Extensibility
            TokenKind::AgentExtension => {
                self.advance();
                let name = self.parse_identifier_or_keyword().unwrap_or_else(|_| "ext".to_string());
                self.expect(TokenKind::LBrace)?;
                let mut purpose = String::new();
                let mut inputs = Vec::new();
                let mut outputs = Vec::new();
                let mut constraints = Vec::new();
                let mut tests = Vec::new();
                let mut permissions = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "purpose" {
                        purpose = self.parse_identifier_or_string()?;
                    } else if key == "inputs" {
                        inputs = self.parse_string_list()?;
                    } else if key == "outputs" {
                        outputs = self.parse_string_list()?;
                    } else if key == "constraints" {
                        constraints = self.parse_string_list()?;
                    } else if key == "tests" {
                        tests = self.parse_string_list()?;
                    } else if key == "permissions" {
                        permissions = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentExtensionContractDecl { name, purpose, inputs, outputs, constraints, tests, permissions, span })
            }
            TokenKind::Proposal => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut title = "change proposal".to_string();
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
                    } else if key == "files" {
                        files = self.parse_string_list()?;
                    } else if key == "symbols" {
                        symbols = self.parse_string_list()?;
                    } else if key == "dependencies" {
                        dependencies = self.parse_string_list()?;
                    } else if key == "risks" {
                        risks = self.parse_string_list()?;
                    } else if key == "migration" {
                        migration = Some(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentChangeProposalDecl { title, files, symbols, dependencies, risks, migration, span })
            }
            TokenKind::Begin => {
                self.advance();
                let action = self.parse_identifier_or_keyword().unwrap_or_else(|_| "change".to_string());
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                self.match_token(&TokenKind::Commit);
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::AgentTransactionDecl { action, body, span })
            }
            TokenKind::Evolvable => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::EvolvableDecl { module_name, metrics_target: None, span })
            }

            TokenKind::Impact => {
                self.advance();
                let target = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ImpactQuery { target, span })
            }



            _ => unreachable!(),
        }
    }
}
