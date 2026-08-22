use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_capability_composition_statement(
        &mut self,
        peek_k: &TokenKind,
        span: &Span,
    ) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Use => {
                let checkpoint = self.cursor;
                self.advance(); // consume 'use'
                if self.check(&TokenKind::Feature)
                    || self.check(&TokenKind::Syntax)
                    || (if let TokenKind::Directive(d) = self.peek_kind() { d == "@feature" } else { false })
                {
                    self.cursor = checkpoint;
                    return Ok(None);
                }
                if let Ok(stmt) = self.parse_use_surface_statement(span) {
                    return Ok(Some(stmt));
                }
                self.cursor = checkpoint;
                Ok(None)
            }
            TokenKind::Borrow => {
                let checkpoint = self.cursor;
                self.advance(); // consume 'borrow'
                let is_mut = self.match_token(&TokenKind::Mut);
                if self.check(&TokenKind::Val) || self.check(&TokenKind::For) {
                    self.cursor = checkpoint;
                    return Ok(None);
                }
                if let TokenKind::Ident(ref s) = self.peek_kind().clone() {
                    if s == "cpu" || s == "memory" || s == "listen" {
                        self.cursor = checkpoint;
                        return Ok(None);
                    }
                }
                if let Ok(target) = self.parse_dotted_path() {
                    if !self.check(&TokenKind::Equal) && !self.check(&TokenKind::LParen) && !self.check(&TokenKind::Colon) {
                        self.match_token(&TokenKind::SemiColon);
                        return Ok(Some(Statement::UseSurface {
                            target,
                            section: None,
                            symbols: Vec::new(),
                            alias: None,
                            shape_fields: Vec::new(),
                            is_borrow: true,
                            is_mut,
                            is_generic: None,
                            span: span.clone(),
                        }));
                    }
                }
                self.cursor = checkpoint;
                Ok(None)
            }
            TokenKind::Access => {
                self.advance();
                let full = self.parse_dotted_path()?;
                let (entity, capability) = if let Some(idx) = full.rfind('.') {
                    (full[..idx].to_string(), full[idx + 1..].to_string())
                } else {
                    (full.clone(), String::new())
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::AccessCapability {
                    entity,
                    capability,
                    span: span.clone(),
                }))
            }
            TokenKind::Grant => {
                self.advance();
                let target = self.parse_dotted_path()?;
                let capabilities = self.parse_brace_ident_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::GrantCapability {
                    target,
                    capabilities,
                    span: span.clone(),
                }))
            }
            TokenKind::Deny => {
                let checkpoint = self.cursor;
                self.advance();
                if let Ok(target) = self.parse_dotted_path() {
                    if self.check(&TokenKind::LBrace) {
                        let capabilities = self.parse_brace_ident_list()?;
                        self.match_token(&TokenKind::SemiColon);
                        return Ok(Some(Statement::DenyCapability {
                            target,
                            capabilities,
                            span: span.clone(),
                        }));
                    }
                }
                self.cursor = checkpoint;
                Ok(None)
            }
            TokenKind::Expose => {
                self.advance();
                let target = self.parse_dotted_path()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::UseSurface {
                    target,
                    section: None,
                    symbols: vec!["__expose__".to_string()],
                    alias: None,
                    shape_fields: Vec::new(),
                    is_borrow: false,
                    is_mut: false,
                    is_generic: None,
                    span: span.clone(),
                }))
            }
            TokenKind::Hide => {
                let checkpoint = self.cursor;
                self.advance();
                if let Ok(target) = self.parse_dotted_path() {
                    if target.contains('.') {
                        let (entity, cap) = if let Some(idx) = target.rfind('.') {
                            (target[..idx].to_string(), target[idx + 1..].to_string())
                        } else {
                            (target.clone(), String::new())
                        };
                        self.match_token(&TokenKind::SemiColon);
                        return Ok(Some(Statement::DenyCapability {
                            target: entity,
                            capabilities: vec![cap],
                            span: span.clone(),
                        }));
                    }
                }
                self.cursor = checkpoint;
                Ok(None)
            }
            TokenKind::Surface => {
                let checkpoint = self.cursor;
                self.advance();
                if let Ok(full) = self.parse_dotted_path() {
                    if full.contains('.') || self.check(&TokenKind::When) {
                        let (entity, name) = if let Some(idx) = full.rfind('.') {
                            (full[..idx].to_string(), full[idx + 1..].to_string())
                        } else {
                            (full.clone(), String::new())
                        };
                        let mut condition = None;
                        if self.match_token(&TokenKind::When) {
                            condition = Some(self.parse_condition_str()?);
                        }
                        if self.check(&TokenKind::LBrace) {
                            let symbols = self.parse_brace_ident_list()?;
                            self.match_token(&TokenKind::SemiColon);
                            return Ok(Some(Statement::SurfaceDefinition(SurfaceDef {
                                entity,
                                name,
                                condition,
                                symbols,
                                span: span.clone(),
                            })));
                        }
                    }
                }
                self.cursor = checkpoint;
                Ok(None)
            }
            TokenKind::Adopt => {
                self.advance();
                let target = self.parse_dotted_path()?;
                let alias = if self.match_token(&TokenKind::As) {
                    Some(self.parse_dotted_path()?)
                } else {
                    None
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::AdoptContract {
                    target,
                    alias,
                    span: span.clone(),
                }))
            }
            TokenKind::Implement => {
                self.advance();
                let contract = self.parse_dotted_path()?;
                let target = if self.match_token(&TokenKind::For) {
                    Some(self.parse_dotted_path()?)
                } else {
                    None
                };
                let methods = self.parse_function_block_methods()?;
                Ok(Some(Statement::ImplementContract {
                    contract,
                    target,
                    methods,
                    span: span.clone(),
                }))
            }
            TokenKind::Extend => {
                let checkpoint = self.cursor;
                self.advance();
                if let Ok(target) = self.parse_dotted_path() {
                    if self.check(&TokenKind::LBrace) {
                        let methods = self.parse_function_block_methods()?;
                        return Ok(Some(Statement::ImplementContract {
                            contract: target,
                            target: None,
                            methods,
                            span: span.clone(),
                        }));
                    }
                }
                self.cursor = checkpoint;
                Ok(None)
            }
            TokenKind::Attach => {
                self.advance();
                let capabilities = if self.check(&TokenKind::LBrace) {
                    self.parse_brace_ident_list()?
                } else {
                    vec![self.parse_dotted_path()?]
                };
                self.expect(TokenKind::To)?;
                let target = self.parse_dotted_path()?;
                let mut when_cond = None;
                let mut if_pred = None;
                if self.match_token(&TokenKind::When) {
                    when_cond = Some(self.parse_condition_str()?);
                } else if self.match_token(&TokenKind::If) {
                    if_pred = Some(self.parse_condition_str()?);
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::AttachCapability {
                    capabilities,
                    target,
                    when_cond,
                    if_pred,
                    span: span.clone(),
                }))
            }
            TokenKind::Detach => {
                self.advance();
                let capability = self.parse_dotted_path()?;
                self.expect(TokenKind::From)?;
                let target = self.parse_dotted_path()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::DetachCapability {
                    capability,
                    target,
                    span: span.clone(),
                }))
            }
            TokenKind::Compose => {
                let checkpoint = self.cursor;
                self.advance();
                if let Ok(name) = self.parse_identifier_or_keyword() {
                    if name != "module" && name != "mod" && self.check(&TokenKind::LBrace) {
                        let capabilities = self.parse_brace_ident_list()?;
                        return Ok(Some(Statement::ComposeCapability {
                            name,
                            capabilities,
                            span: span.clone(),
                        }));
                    }
                }
                self.cursor = checkpoint;
                Ok(None)
            }
            TokenKind::Mixin => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let methods = if self.check(&TokenKind::LBrace) {
                    self.parse_function_block_methods()?
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Vec::new()
                };
                Ok(Some(Statement::MixinDecl(MixinDef {
                    name,
                    methods,
                    span: span.clone(),
                })))
            }
            TokenKind::Capability => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let methods = if self.check(&TokenKind::LBrace) {
                    self.parse_brace_ident_list()?
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Vec::new()
                };
                Ok(Some(Statement::CapabilityDecl(CapabilityDef {
                    name,
                    methods,
                    span: span.clone(),
                })))
            }
            TokenKind::Provide => {
                self.advance();
                let capability = self.parse_dotted_path()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ProvideCapability {
                    capability,
                    span: span.clone(),
                }))
            }
            TokenKind::Require => {
                self.advance();
                let contract = self.parse_dotted_path()?;
                let alias = if self.match_token(&TokenKind::As) {
                    Some(self.parse_identifier_or_keyword()?)
                } else {
                    None
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::RequireCapability {
                    contract,
                    alias,
                    span: span.clone(),
                }))
            }
            TokenKind::Resolve => {
                self.advance();
                let contract = self.parse_dotted_path()?;
                self.expect(TokenKind::Arrow)?;
                let implementation = self.parse_dotted_path()?;
                let condition = if self.match_token(&TokenKind::When) {
                    Some(self.parse_condition_str()?)
                } else {
                    None
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ResolveContract {
                    contract,
                    implementation,
                    condition,
                    span: span.clone(),
                }))
            }
            TokenKind::Select => {
                self.advance();
                let contract = self.parse_dotted_path()?;
                let candidates = self.parse_brace_ident_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::SelectContract {
                    contract,
                    candidates,
                    span: span.clone(),
                }))
            }
            _ => self.parse_capability_extensions_statement(peek_k, span),
        }
    }

    fn parse_use_surface_statement(&mut self, span: &Span) -> Result<Statement, String> {
        let mut target = self.parse_identifier_or_keyword()?;
        let mut section = None;
        let mut symbols = Vec::new();
        let mut alias = None;
        let mut shape_fields = Vec::new();
        let mut is_generic = None;

        while self.match_token(&TokenKind::Dot) {
            let is_sec = (self.check(&TokenKind::Section)
                || (if let TokenKind::Ident(s) = self.peek_kind() { s == "section" } else { false }))
                && self.peek_next_kind() == Some(&TokenKind::LParen);
            if is_sec {
                self.advance();
                self.expect(TokenKind::LParen)?;
                section = Some(self.parse_identifier_or_string()?);
                self.expect(TokenKind::RParen)?;
                break;
            } else {
                let seg = self.parse_identifier_or_keyword()?;
                target.push('.');
                target.push_str(&seg);
            }
        }

        // Generic use <Contract><Implementation>
        if self.match_token(&TokenKind::Less) {
            is_generic = Some(self.parse_dotted_path()?);
            self.expect(TokenKind::Greater)?;
        }

        // use <Entity> only { a, b }
        if self.match_token(&TokenKind::Only) {
            symbols = self.parse_brace_ident_list()?;
        }
        // use <Entity> as <Alias> or as { a, b }
        else if self.match_token(&TokenKind::As) {
            if self.check(&TokenKind::LBrace) {
                shape_fields = self.parse_brace_ident_list()?;
            } else {
                alias = Some(self.parse_dotted_path()?);
            }
        }

        self.match_token(&TokenKind::SemiColon);

        Ok(Statement::UseSurface {
            target,
            section,
            symbols,
            alias,
            shape_fields,
            is_borrow: false,
            is_mut: false,
            is_generic,
            span: span.clone(),
        })
    }
}
