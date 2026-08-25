use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_syntax_composition_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Syntax
            | TokenKind::Use
            | TokenKind::Partial
            | TokenKind::Augment
            | TokenKind::Override
            | TokenKind::ExtensionPoint
            | TokenKind::Overlay
            | TokenKind::Open
            | TokenKind::Closed
            | TokenKind::Project => {}
            TokenKind::Ident(s) if s == "parallel_choose" || s == "project" => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_syntax_composition_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_syntax_composition_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Syntax => {
                self.advance();
                let mut full_name = self.parse_identifier_or_keyword()?;
                let mut namespace = None;
                let mut name = full_name.clone();
                while self.check(&TokenKind::Colon) {
                    self.advance();
                    if self.match_token(&TokenKind::Colon) {
                        let next_part = self.parse_identifier_or_keyword()?;
                        namespace = Some(name.clone());
                        name = next_part.clone();
                        full_name.push_str("::");
                        full_name.push_str(&next_part);
                    } else {
                        break;
                    }
                }
                let mut params = Vec::new();
                if self.match_token(&TokenKind::LParen) {
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        let f_span = self.current_span();
                        let is_mut = self.match_token(&TokenKind::Mut);
                        let p_name = self.parse_identifier_or_keyword()?;
                        let mut p_type = Type::Void;
                        if self.match_token(&TokenKind::Colon) {
                            p_type = self.parse_type()?;
                        }
                        params.push(FunctionParam { name: p_name, param_type: p_type, is_mut, span: f_span });
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                    self.expect(TokenKind::RParen)?;
                }
                let mut return_type = None;
                if self.match_token(&TokenKind::Arrow) {
                    return_type = Some(self.parse_type()?);
                }
                let mut body = None;
                if self.check(&TokenKind::LBrace) {
                    let is_schema = if let Some(TokenKind::Ident(_)) = self.peek_ahead(1) {
                        self.peek_ahead(2) == Some(&TokenKind::Colon) && self.peek_ahead(3) != Some(&TokenKind::Colon)
                    } else {
                        false
                    };
                    if is_schema {
                        self.expect(TokenKind::LBrace)?;
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let f_span = self.current_span();
                            let f_name = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            let f_type = self.parse_type().unwrap_or(Type::Void);
                            params.push(FunctionParam { name: f_name, param_type: f_type, is_mut: false, span: f_span });
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                    } else {
                        let blk = self.parse_block()?;
                        body = Some(blk);
                    }
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::SyntaxDecl {
                    name,
                    pattern: None,
                    namespace,
                    version: None,
                    params,
                    return_type,
                    body,
                    span,
                })
            }
            TokenKind::Use => {
                self.advance();
                if self.match_token(&TokenKind::Feature) || self.match_token(&TokenKind::Directive("@feature".to_string())) || (if let TokenKind::Ident(s) = self.peek_kind() { if s == "feature" { self.advance(); true } else { false } } else { false }) {
                    self.expect(TokenKind::LParen)?;
                    let feature_intent = self.parse_identifier_or_string()?;
                    self.expect(TokenKind::RParen)?;
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::SemanticImportDecl { feature_intent, alias: None, span });
                }
                let _is_syntax = if self.match_token(&TokenKind::Syntax) {
                    true
                } else if let TokenKind::Ident(s) = self.peek_kind() {
                    if s == "syntax" {
                        self.advance();
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                let mut namespace = self.parse_identifier_or_keyword()?;
                while self.check(&TokenKind::Colon) {
                    self.advance();
                    if self.match_token(&TokenKind::Colon) {
                        namespace.push_str("::");
                        namespace.push_str(&self.parse_identifier_or_keyword()?);
                    } else {
                        break;
                    }
                }
                let mut version = None;
                if self.match_token(&TokenKind::At) || (if let TokenKind::Directive(d) = self.peek_kind() { d.starts_with('@') } else { false }) {
                    if let TokenKind::IntLit(v) = self.peek_kind() {
                        version = Some(*v as usize);
                        self.advance();
                    } else if let TokenKind::Directive(d) = self.advance().kind {
                        let num = d.trim_start_matches('@').parse::<usize>().unwrap_or(0);
                        if num > 0 { version = Some(num); }
                    }
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::UseSyntaxDecl { namespace, version, span })
            }
            TokenKind::Ident(id) if id == "parallel_choose" => {
                self.advance();
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
                Ok(Statement::ParallelChoose { branches, span })
            }
            _ if *peek_k == TokenKind::Project || matches!(peek_k, TokenKind::Ident(s) if s == "project") => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut profile = std::collections::HashMap::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let k = self.parse_identifier_or_keyword()?;
                    if k == "skills" || k == "skill" {
                        self.expect(TokenKind::LBrace)?;
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let sk = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            let sv = self.parse_identifier_or_string()?;
                            profile.insert(sk, sv);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ProjectSkillsDecl { profile, span })
            }

            // Layer 1: DNA of Code Itself
            TokenKind::Partial => {
                self.advance();
                let kind = if self.match_token(&TokenKind::Struct) {
                    "struct".to_string()
                } else if self.match_token(&TokenKind::Mod) {
                    "module".to_string()
                } else {
                    self.parse_identifier_or_keyword()?
                };
                let name = self.parse_identifier_or_keyword()?;
                let mut body_struct = None;
                let mut body_module = None;
                if self.check(&TokenKind::LBrace) {
                    if kind == "module" || kind == "mod" {
                        let mut mod_def = ModuleDef::default();
                        mod_def.name = name.clone();
                        mod_def.is_partial = true;
                        self.expect(TokenKind::LBrace)?;
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            match self.parse_statement() {
                                Ok(stmt) => mod_def.statements.push(stmt),
                                Err(_) => {
                                    self.synchronize();
                                    if self.check(&TokenKind::RBrace) || self.check(&TokenKind::EOF) {
                                        break;
                                    }
                                }
                            }
                        }
                        self.expect(TokenKind::RBrace)?;
                        body_module = Some(mod_def);
                    } else {
                        self.expect(TokenKind::LBrace)?;
                        let mut fields = Vec::new();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let f_span = self.current_span();
                            let is_field_pub = self.match_token(&TokenKind::Pub);
                            let field_name = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            let field_type = self.parse_type().unwrap_or(Type::Void);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                            fields.push(StructField { name: field_name, field_type, is_pub: is_field_pub, span: f_span });
                        }
                        self.expect(TokenKind::RBrace)?;
                        body_struct = Some(StructDef {
                            name: name.clone(),
                            generic_params: vec![],
                            is_pub: true,
                            is_partial: true,
                            is_sealed: false,
                            is_extension_only: false,
                            is_open: false,
                            is_closed: false,
                            friend_modules: vec![],
                            extension_points: vec![],
                            fields,
                            directives: vec![],
                            span: span.clone(),
                        });
                    }
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::PartialDecl { kind, name, body_struct, body_module, span })
            }
            TokenKind::Augment => {
                let ext = self.parse_extension_block()?;
                Ok(Statement::AugmentDecl(ext))
            }
            TokenKind::Override => {
                self.advance();
                let mut target = self.parse_identifier_or_keyword()?;
                if self.match_token(&TokenKind::Dot) {
                    let method_name = self.parse_identifier_or_keyword()?;
                    target = format!("{}.{}", target, method_name);
                }
                let mut fn_name = target.clone();
                if fn_name.contains('.') {
                    fn_name = fn_name.split('.').last().unwrap_or(&fn_name).to_string();
                }
                self.expect(TokenKind::LParen)?;
                let mut params = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let p_span = self.current_span();
                    let is_mut = self.match_token(&TokenKind::Mut);
                    let p_name = self.parse_identifier_or_keyword()?;
                    let mut param_type = Type::Void;
                    if self.match_token(&TokenKind::Colon) {
                        param_type = self.parse_type()?;
                    }
                    params.push(FunctionParam { name: p_name, param_type, is_mut, span: p_span });
                    if !self.match_token(&TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::RParen)?;
                let mut return_type = Type::Void;
                if self.match_token(&TokenKind::Arrow) {
                    return_type = self.parse_type()?;
                }
                let body = self.parse_block()?;
                let method = FunctionDef {
                    name: fn_name,
                    generic_params: vec![],
                    is_pub: true,
                    params,
                    return_type,
                    body,
                    directives: vec![],
                    morphic_param: None,
                    span: span.clone(),
                };
                Ok(Statement::OverrideDecl { target, method, span })
            }
            TokenKind::ExtensionPoint => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut points = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    points.push(self.parse_identifier_or_keyword()?);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ExtensionPointDecl { target, points, span })
            }

            TokenKind::Overlay => {
                self.advance();
                let target_env = self.parse_identifier_or_keyword()?;
                let name = target_env.clone();
                let body = self.parse_block()?;
                Ok(Statement::ModuleOverlayDecl { name, target_env, body, span })
            }

            // Layer 3: Type System for Extensibility
            TokenKind::Open => {
                self.advance();
                let _ = self.parse_identifier_or_keyword().ok();
                let name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::OpenClosedTypeDecl { is_open: true, name, span })
            }
            TokenKind::Closed => {
                self.advance();
                let _ = self.parse_identifier_or_keyword().ok();
                let name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::OpenClosedTypeDecl { is_open: false, name, span })
            }

            _ => unreachable!(),
        }
    }
}
