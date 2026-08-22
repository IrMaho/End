use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_extension_block(&mut self) -> Result<ExtensionBlock, String> {
        let span = self.current_span();
        let is_augment = self.match_token(&TokenKind::Augment);
        if !is_augment {
            self.expect(TokenKind::Extend)?;
        }
        let is_struct = if self.match_token(&TokenKind::Struct) {
            true
        } else if self.match_token(&TokenKind::Mod) {
            false
        } else {
            true
        };
        let target = self.parse_identifier_or_keyword()?;
        
        let mut generic_params = Vec::new();
        if self.match_token(&TokenKind::Less) {
            while !self.check(&TokenKind::Greater) && !self.check(&TokenKind::EOF) {
                if let TokenKind::Ident(g) = self.advance().kind {
                    generic_params.push(g);
                }
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::Greater)?;
        }

        let mut at_hook = None;
        if self.match_token(&TokenKind::At) || self.match_token(&TokenKind::Dot) {
            if self.check(&TokenKind::At) { self.advance(); }
            at_hook = Some(self.parse_identifier_or_keyword()?);
        }

        let mut required_capability = None;
        if self.match_token(&TokenKind::Requires) {
            self.match_token(&TokenKind::Colon);
            let _ = self.parse_identifier_or_keyword().ok();
            if self.match_token(&TokenKind::LParen) {
                required_capability = Some(self.parse_identifier_or_string()?);
                self.expect(TokenKind::RParen)?;
            }
        }

        let mut when_feature = None;
        if self.match_token(&TokenKind::When) {
            let _ = self.parse_identifier_or_keyword().ok();
            if self.match_token(&TokenKind::LParen) {
                when_feature = Some(self.parse_identifier_or_string()?);
                self.expect(TokenKind::RParen)?;
            }
        }

        let mut version_req = None;
        let mut owned_by = None;
        let mut lifecycle = None;

        if self.match_token(&TokenKind::OwnedBy) {
            owned_by = Some(self.parse_identifier_or_string()?);
        }

        while let TokenKind::Directive(d) = self.peek_kind() {
            let mut full_d = d.clone();
            self.advance();
            if self.match_token(&TokenKind::LParen) {
                let mut inner = String::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::Ident(s) | TokenKind::StringLit(s) => inner.push_str(&s),
                        TokenKind::IntLit(i) => inner.push_str(&i.to_string()),
                        TokenKind::GreaterEqual => inner.push_str(">="),
                        TokenKind::LessEqual => inner.push_str("<="),
                        TokenKind::Greater => inner.push_str(">"),
                        TokenKind::Less => inner.push_str("<"),
                        TokenKind::Equal => inner.push_str("="),
                        _ => {}
                    }
                }
                self.expect(TokenKind::RParen)?;
                full_d = format!("{}({})", full_d, inner);
            }
            if full_d.starts_with("@api") {
                version_req = Some(full_d);
            }
        }

        self.expect(TokenKind::LBrace)?;
        let mut functions = Vec::new();
        let mut overrides = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let mut pending_directives = Vec::new();
            while let TokenKind::Directive(d) = self.peek_kind() {
                let dir_name = d.clone();
                let dir_span = self.current_span();
                self.advance();
                pending_directives.push(Directive {
                    name: dir_name,
                    args: Vec::new(),
                    span: dir_span,
                });
            }
            if self.match_token(&TokenKind::Override) {
                if self.check(&TokenKind::Fn) {
                    overrides.push(self.parse_function(false, pending_directives)?);
                } else {
                    let full_target = self.parse_identifier_or_keyword()?;
                    let mut fn_name = full_target;
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
                    overrides.push(FunctionDef {
                        name: fn_name,
                        generic_params: vec![],
                        is_pub: true,
                        params,
                        return_type,
                        body,
                        directives: pending_directives,
                        morphic_param: None,
                        span: self.current_span(),
                    });
                }
            } else if self.check(&TokenKind::Fn) {
                functions.push(self.parse_function(false, pending_directives)?);
            } else if self.match_token(&TokenKind::Pub) {
                if self.check(&TokenKind::Fn) {
                    functions.push(self.parse_function(true, pending_directives)?);
                }
            } else if self.match_token(&TokenKind::Capability) {
                let cap = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                if required_capability.is_none() {
                    required_capability = Some(cap);
                }
            } else {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(ExtensionBlock {
            target,
            is_struct,
            is_augment,
            trait_name: None,
            at_hook,
            required_capability,
            when_feature,
            generic_params,
            version_req,
            owned_by,
            lifecycle,
            functions,
            overrides,
            span,
        })
    }

}
