use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_class(&mut self, is_pub: bool, pending_directives: Vec<Directive>) -> Result<ClassDef, String> {
        let span = self.current_span();
        let mut is_abstract = false;
        let mut is_sealed = false;
        let mut is_open = false;

        if self.match_token(&TokenKind::Abstract) {
            is_abstract = true;
        }
        if self.match_token(&TokenKind::Sealed) {
            is_sealed = true;
        }
        if self.match_token(&TokenKind::Open) {
            is_open = true;
        }

        self.expect(TokenKind::Class)?;
        let name = self.parse_identifier_or_keyword()?;

        let mut extends = Vec::new();
        if self.match_token(&TokenKind::Extends) || self.match_token(&TokenKind::Inherits) || self.match_token(&TokenKind::Colon) {
            while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::With) && !self.check(&TokenKind::Implements) && !self.check(&TokenKind::Lock) && !self.check(&TokenKind::EOF) {
                extends.push(self.parse_identifier_or_keyword()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        let mut mixins = Vec::new();
        if self.match_token(&TokenKind::With) {
            while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::Implements) && !self.check(&TokenKind::Lock) && !self.check(&TokenKind::EOF) {
                mixins.push(self.parse_identifier_or_keyword()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        let mut implements = Vec::new();
        if self.match_token(&TokenKind::Implements) || self.match_token(&TokenKind::Implement) {
            if self.check(&TokenKind::LBrace) {
                // Could be implements { A, B } or start of class body
                // Peek next to disambiguate
                let is_block_list = if let Some(TokenKind::Ident(_)) = self.peek_next_kind() {
                    true
                } else {
                    false
                };
                if is_block_list {
                    self.advance(); // consume {
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        implements.push(self.parse_identifier_or_keyword()?);
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                }
            } else {
                while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::Lock) && !self.check(&TokenKind::EOF) {
                    implements.push(self.parse_identifier_or_keyword()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
            }
        }

        let mut locked_contracts = Vec::new();
        if self.match_token(&TokenKind::Lock) {
            while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::EOF) {
                let mut path = self.parse_identifier_or_keyword()?;
                while self.match_token(&TokenKind::Dot) {
                    path.push('.');
                    path.push_str(&self.parse_identifier_or_keyword()?);
                }
                locked_contracts.push(path);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut shared_parents = Vec::new();
        let mut virtual_parents = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.match_token(&TokenKind::SemiColon) {
                continue;
            }

            if self.match_token(&TokenKind::Share) {
                shared_parents.push(self.parse_identifier_or_keyword()?);
                self.match_token(&TokenKind::SemiColon);
                continue;
            }

            if self.match_token(&TokenKind::Virtual) {
                virtual_parents.push(self.parse_identifier_or_keyword()?);
                self.match_token(&TokenKind::SemiColon);
                continue;
            }

            let member_span = self.current_span();
            let mut member_directives = Vec::new();
            while let TokenKind::Directive(d) = self.peek_kind() {
                let d_str = d.clone();
                self.advance();
                member_directives.push(Directive {
                    name: d_str,
                    args: vec![],
                    span: member_span.clone(),
                });
            }

            let mut m_is_abstract = false;
            let mut m_is_override = false;
            let mut m_is_requires_override = false;

            if self.match_token(&TokenKind::Abstract) {
                m_is_abstract = true;
            }
            if self.match_token(&TokenKind::Override) {
                m_is_override = true;
            }
            if self.match_token(&TokenKind::Requires) {
                if self.match_token(&TokenKind::Override) {
                    m_is_requires_override = true;
                }
            }

            let is_pub_member = self.match_token(&TokenKind::Pub);

            if self.check(&TokenKind::Fn) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "fn" || s == "def" } else { false }) {
                self.advance(); // consume fn or def
                let f_name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LParen)?;
                let mut params = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let p_span = self.current_span();
                    let is_mut = self.match_token(&TokenKind::Mut);
                    let p_name = self.parse_identifier_or_keyword()?;
                    let mut p_ty = if p_name == "self" || p_name == "&self" {
                        Type::Custom(name.clone())
                    } else {
                        Type::Void
                    };
                    if self.match_token(&TokenKind::Colon) {
                        p_ty = self.parse_type()?;
                    }
                    params.push(FunctionParam {
                        name: p_name,
                        param_type: p_ty,
                        is_mut,
                        span: p_span,
                    });
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                let return_type = if self.match_token(&TokenKind::Arrow) {
                    self.parse_type()?
                } else if self.check(&TokenKind::Colon) {
                    self.advance();
                    self.parse_type()?
                } else {
                    Type::Void
                };

                let body = if self.check(&TokenKind::LBrace) {
                    self.parse_block()?
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Block { statements: vec![], span: member_span.clone() }
                };

                if m_is_abstract {
                    member_directives.push(Directive { name: "@abstract".to_string(), args: vec![], span: member_span.clone() });
                }
                if m_is_override {
                    member_directives.push(Directive { name: "@override".to_string(), args: vec![], span: member_span.clone() });
                }
                if m_is_requires_override {
                    member_directives.push(Directive { name: "@requires_override".to_string(), args: vec![], span: member_span.clone() });
                }

                methods.push(FunctionDef {
                    name: f_name,
                    generic_params: vec![],
                    is_pub: is_pub_member || is_pub,
                    params,
                    return_type,
                    body,
                    directives: member_directives,
                    morphic_param: None,
                    span: member_span,
                });
            } else {
                // Field declaration: [pub] [val|mut] name: Type, or val name = expr;
                let is_pub_field = is_pub_member || self.match_token(&TokenKind::Pub);
                let is_val = self.match_token(&TokenKind::Val);
                let is_mut = self.match_token(&TokenKind::Mut);
                let f_name = self.parse_identifier_or_keyword()?;
                let mut field_type = Type::Custom("Any".to_string());
                if self.match_token(&TokenKind::Colon) {
                    field_type = self.parse_type()?;
                } else if self.match_token(&TokenKind::Equal) {
                    let _init = self.parse_expression()?;
                } else if !is_val && !is_mut {
                    return Err(format!("Expected ':' or '=' after field name '{}' in class at line {}", f_name, member_span.line));
                }
                self.match_token(&TokenKind::SemiColon);
                self.match_token(&TokenKind::Comma);

                fields.push(StructField {
                    name: f_name,
                    field_type,
                    is_pub: is_pub_field || is_pub,
                    span: member_span,
                });
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(ClassDef {
            name,
            is_pub,
            is_abstract,
            is_sealed,
            is_open,
            extends,
            mixins,
            implements,
            shared_parents,
            virtual_parents,
            locked_contracts,
            fields,
            methods,
            directives: pending_directives,
            span,
        })
    }
}
