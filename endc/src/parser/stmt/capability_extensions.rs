use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_capability_extensions_statement(
        &mut self,
        peek_k: &TokenKind,
        span: &Span,
    ) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::View => {
                let checkpoint = self.checkpoint();
                self.advance();
                if let Ok(entity) = self.parse_dotted_path() {
                    if self.match_token(&TokenKind::As) {
                        let view_shape = self.parse_dotted_path()?;
                        self.match_token(&TokenKind::SemiColon);
                        return Ok(Some(Statement::ViewProjection {
                            entity,
                            view_shape,
                            span: span.clone(),
                        }));
                    }
                }
                self.restore_checkpoint(checkpoint);
                Ok(None)
            }
            TokenKind::Project => {
                let checkpoint = self.checkpoint();
                self.advance();
                if !self.check(&TokenKind::LBrace) {
                    if let Ok(entity) = self.parse_dotted_path() {
                        if self.check(&TokenKind::LBrace) && self.peek_ahead(2) != Some(&TokenKind::Colon) {
                            if let Ok(fields) = self.parse_brace_ident_list() {
                                self.match_token(&TokenKind::SemiColon);
                                return Ok(Some(Statement::ProjectSurface {
                                    entity,
                                    fields,
                                    span: span.clone(),
                                }));
                            }
                        }
                    }
                }
                self.restore_checkpoint(checkpoint);
                Ok(None)
            }
            TokenKind::Delegate => {
                self.advance();
                let full = self.parse_dotted_path()?;
                let (entity, method) = if let Some(idx) = full.rfind('.') {
                    (full[..idx].to_string(), full[idx + 1..].to_string())
                } else {
                    (full.clone(), String::new())
                };
                self.expect(TokenKind::To)?;
                let target = self.parse_dotted_path()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::DelegateBehavior {
                    entity,
                    method,
                    target,
                    span: span.clone(),
                }))
            }
            TokenKind::Proxy => {
                self.advance();
                let target = self.parse_dotted_path()?;
                self.expect(TokenKind::Through)?;
                let interceptor = self.parse_dotted_path()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ProxyCapability {
                    target,
                    interceptor,
                    span: span.clone(),
                }))
            }
            TokenKind::Decorate => {
                let checkpoint = self.checkpoint();
                self.advance();
                if let Ok(full) = self.parse_dotted_path() {
                    if self.match_token(&TokenKind::With) {
                        if self.check(&TokenKind::LBrace) {
                            let with_caps = self.parse_brace_ident_list()?;
                            self.match_token(&TokenKind::SemiColon);
                            return Ok(Some(Statement::DecorateEntity {
                                target: full,
                                method: None,
                                with_caps,
                                span: span.clone(),
                            }));
                        } else if let Ok(cap) = self.parse_dotted_path() {
                            let (target, method) = if let Some(idx) = full.rfind('.') {
                                (full[..idx].to_string(), Some(full[idx + 1..].to_string()))
                            } else {
                                (full.clone(), None)
                            };
                            self.match_token(&TokenKind::SemiColon);
                            return Ok(Some(Statement::DecorateEntity {
                                target,
                                method,
                                with_caps: vec![cap],
                                span: span.clone(),
                            }));
                        }
                    }
                }
                self.restore_checkpoint(checkpoint);
                Ok(None)
            }
            TokenKind::Intercept => {
                self.advance();
                let full = self.parse_dotted_path()?;
                let (entity, method) = if let Some(idx) = full.rfind('.') {
                    (full[..idx].to_string(), full[idx + 1..].to_string())
                } else {
                    (full.clone(), String::new())
                };
                self.expect(TokenKind::LBrace)?;
                let mut before_block = None;
                let mut after_block = None;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    if self.match_token(&TokenKind::Before) {
                        before_block = Some(self.parse_block()?);
                    } else if self.match_token(&TokenKind::After) {
                        after_block = Some(self.parse_block()?);
                    } else {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Some(Statement::InterceptMethod(InterceptDef {
                    entity,
                    method,
                    before_block,
                    after_block,
                    span: span.clone(),
                })))
            }
            TokenKind::Hook => {
                self.advance();
                let full = self.parse_dotted_path()?;
                let (entity, event_point) = if let Some(idx) = full.rfind('.') {
                    (full[..idx].to_string(), full[idx + 1..].to_string())
                } else {
                    (full.clone(), String::new())
                };
                let body = self.parse_block()?;
                Ok(Some(Statement::HookEvent(HookDef {
                    entity,
                    event_point,
                    body,
                    span: span.clone(),
                })))
            }
            TokenKind::Enable | TokenKind::Disable => {
                let enabled = *peek_k == TokenKind::Enable;
                self.advance();
                let capability = self.parse_dotted_path()?;
                self.expect(TokenKind::For)?;
                let entity = self.parse_dotted_path()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::EnableCapability {
                    capability,
                    entity,
                    enabled,
                    span: span.clone(),
                }))
            }
            TokenKind::Scope => {
                let checkpoint = self.checkpoint();
                self.advance();
                if let Ok(name) = self.parse_identifier_or_keyword() {
                    if self.check(&TokenKind::LBrace) && self.peek_ahead(2) != Some(&TokenKind::Colon) {
                        if let Ok(block) = self.parse_block() {
                            return Ok(Some(Statement::ScopeBoundary(ScopeDef {
                                name,
                                statements: block.statements,
                                span: span.clone(),
                            })));
                        }
                    }
                }
                self.restore_checkpoint(checkpoint);
                Ok(None)
            }
            TokenKind::Context => {
                let checkpoint = self.checkpoint();
                self.advance();
                if let Ok(environment) = self.parse_identifier_or_keyword() {
                    if self.check(&TokenKind::LBrace) && self.peek_ahead(2) != Some(&TokenKind::Colon) {
                        if let Ok(block) = self.parse_block() {
                            return Ok(Some(Statement::ContextEnv(ContextDef {
                                environment,
                                statements: block.statements,
                                span: span.clone(),
                            })));
                        }
                    }
                }
                self.restore_checkpoint(checkpoint);
                Ok(None)
            }
            TokenKind::FeatureSwitch => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut enabled_env = String::new();
                if self.check(&TokenKind::Ident("enabled".to_string())) || self.check(&TokenKind::Enable) {
                    self.advance();
                    enabled_env = self.parse_identifier_or_keyword()?;
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Some(Statement::FeatureSwitchDecl {
                    name,
                    enabled_env,
                    span: span.clone(),
                }))
            }
            TokenKind::Augment => {
                let checkpoint = self.checkpoint();
                self.advance();
                if let Ok(target) = self.parse_dotted_path() {
                    if self.match_token(&TokenKind::LBrace) {
                        if self.match_token(&TokenKind::Capability) {
                            let mut capabilities = vec![self.parse_dotted_path()?];
                            self.match_token(&TokenKind::SemiColon);
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                if self.match_token(&TokenKind::Capability) {
                                    capabilities.push(self.parse_dotted_path()?);
                                } else {
                                    capabilities.push(self.parse_dotted_path()?);
                                }
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                            return Ok(Some(Statement::AttachCapability {
                                capabilities,
                                target,
                                when_cond: None,
                                if_pred: None,
                                span: span.clone(),
                            }));
                        }
                    }
                }
                self.restore_checkpoint(checkpoint);
                Ok(None)
            }
            TokenKind::Traitify => {
                self.advance();
                let entity = self.parse_dotted_path()?;
                self.expect(TokenKind::As)?;
                let trait_name = self.parse_dotted_path()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::TraitifyCheck {
                    entity,
                    trait_name,
                    span: span.clone(),
                }))
            }
            TokenKind::Equip => {
                self.advance();
                let entity = self.parse_dotted_path()?;
                let mut condition = None;
                if self.match_token(&TokenKind::When) {
                    condition = Some(self.parse_condition_str()?);
                }
                self.expect(TokenKind::With)?;
                let capabilities = if self.check(&TokenKind::LBrace) {
                    self.parse_brace_ident_list()?
                } else {
                    vec![self.parse_dotted_path()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::EquipEntity {
                    entity,
                    capabilities,
                    condition,
                    span: span.clone(),
                }))
            }
            TokenKind::Fuse => {
                self.advance();
                let mut features = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    features = self.parse_brace_ident_list()?;
                } else {
                    while !self.check(&TokenKind::As) && !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                        features.push(self.parse_identifier_or_keyword()?);
                        self.match_token(&TokenKind::Comma);
                    }
                }
                self.expect(TokenKind::As)?;
                let alias = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::FuseFeatures {
                    features,
                    alias,
                    span: span.clone(),
                }))
            }
            TokenKind::Shape => {
                self.advance();
                let full = self.parse_dotted_path()?;
                let (entity, name) = if let Some(idx) = full.rfind('.') {
                    (full[..idx].to_string(), full[idx + 1..].to_string())
                } else {
                    (full.clone(), String::new())
                };
                let fields = self.parse_brace_ident_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ShapeDefinition(ShapeDef {
                    entity,
                    name,
                    fields,
                    span: span.clone(),
                })))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn parse_dotted_path(&mut self) -> Result<String, String> {
        let mut path = self.parse_identifier_or_keyword()?;
        while self.match_token(&TokenKind::Dot) {
            let seg = self.parse_identifier_or_keyword()?;
            path.push('.');
            path.push_str(&seg);
        }
        Ok(path)
    }

    pub(crate) fn parse_brace_ident_list(&mut self) -> Result<Vec<String>, String> {
        self.expect(TokenKind::LBrace)?;
        let mut list = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if let Ok(id) = self.parse_identifier_or_keyword() {
                list.push(id);
            } else {
                let s = format!("{:?}", self.peek_kind());
                self.advance();
                list.push(s);
            }
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(list)
    }

    pub(crate) fn parse_function_block_methods(&mut self) -> Result<Vec<FunctionDef>, String> {
        self.expect(TokenKind::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.check(&TokenKind::Fn) {
                methods.push(self.parse_function(false, vec![])?);
            } else {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(methods)
    }

    pub(crate) fn parse_condition_str(&mut self) -> Result<String, String> {
        let mut parts = Vec::new();
        while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::With) && !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
            parts.push(self.parse_token_literal_string());
            self.advance();
        }
        Ok(parts.join(""))
    }

    fn parse_token_literal_string(&self) -> String {
        match self.peek_kind() {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::StringLit(s) => format!("\"{}\"", s),
            TokenKind::IntLit(n) => n.to_string(),
            TokenKind::FloatLit(f) => f.to_string(),
            TokenKind::Dot => ".".to_string(),
            TokenKind::EqualEqual => " == ".to_string(),
            TokenKind::BangEqual => " != ".to_string(),
            TokenKind::Greater => " > ".to_string(),
            TokenKind::Less => " < ".to_string(),
            TokenKind::GreaterEqual => " >= ".to_string(),
            TokenKind::LessEqual => " <= ".to_string(),
            _ => format!("{:?}", self.peek_kind()),
        }
    }
}
