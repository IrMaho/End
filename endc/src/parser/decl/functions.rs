use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_function(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<FunctionDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Fn)?;

        let (name, morphic_param) = if let TokenKind::MorphicIdent(m) = self.peek_kind() {
            let m_clone = m.clone();
            self.advance();
            let p = if m_clone.starts_with('{') && m_clone.contains('}') {
                let end_brace = m_clone.find('}').unwrap();
                Some(m_clone[1..end_brace].to_string())
            } else {
                None
            };
            (m_clone, p)
        } else {
            (self.parse_identifier_or_keyword()?, None)
        };

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

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
            let p_span = self.current_span();
            let is_ref = self.match_token(&TokenKind::Ampersand);
            let is_mut = self.match_token(&TokenKind::Mut);
            let is_star_star = self.match_token(&TokenKind::StarStar);
            let is_star = !is_star_star && self.match_token(&TokenKind::Star);
            let mut param_name = self.parse_identifier_or_keyword()?;
            if param_name == "required" && !self.check(&TokenKind::Colon) && !self.check(&TokenKind::Comma) && !self.check(&TokenKind::RParen) {
                let actual = self.parse_identifier_or_keyword()?;
                param_name = format!("required_{}", actual);
            }
            if is_star_star {
                param_name = format!("**{}", param_name);
            } else if is_star {
                param_name = format!("*{}", param_name);
            } else if is_ref {
                param_name = format!("&{}", param_name);
            }

            let mut param_type = Type::Void;
            if self.match_token(&TokenKind::Colon) {
                param_type = self.parse_type()?;
            } else if param_name == "&self" || param_name == "self" {
                param_type = Type::Custom("Self".to_string());
            }

            if self.match_token(&TokenKind::Equal) {
                let _default_expr = self.parse_expression()?;
            }

            params.push(FunctionParam {
                name: param_name,
                param_type,
                is_mut,
                span: p_span,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;

        // Return type
        let return_type = if self.match_token(&TokenKind::Arrow) {
            self.parse_type()?
        } else if self.check(&TokenKind::Bang)
            || matches!(
                self.peek_kind(),
                TokenKind::Ident(_)
                    | TokenKind::Sealed
                    | TokenKind::Contract
                    | TokenKind::Security
                    | TokenKind::Boundary
                    | TokenKind::Purity
                    | TokenKind::Stable
                    | TokenKind::Compat
            )
            || self.check(&TokenKind::Star)
            || self.check(&TokenKind::LBracket)
        {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = if self.check(&TokenKind::LBrace) {
            self.parse_block()?
        } else {
            self.match_token(&TokenKind::SemiColon);
            Block {
                statements: vec![],
                span: span.clone(),
            }
        };

        Ok(FunctionDef {
            name,
            generic_params,
            is_pub,
            params,
            return_type,
            body,
            directives,
            morphic_param,
            span,
        })
    }


    pub(crate) fn parse_operation(&mut self, is_pub: bool) -> Result<OperationDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Operation)?;

        let mut name = String::new();
        if matches!(self.peek_kind(), TokenKind::Ident(_)) || self.check(&TokenKind::Operation) || self.check(&TokenKind::Event) {
            name = self.parse_identifier_or_keyword()?;
        }

        let mut params = Vec::new();
        if self.match_token(&TokenKind::LParen) {
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                let p_span = self.current_span();
                let is_mut = self.match_token(&TokenKind::Mut);
                let p_name = self.parse_identifier_or_keyword()?;
                let mut param_type = Type::Void;
                if self.match_token(&TokenKind::Colon) {
                    param_type = self.parse_type()?;
                }
                params.push(FunctionParam {
                    name: p_name,
                    param_type,
                    is_mut,
                    span: p_span,
                });
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
        }

        let mut return_type = Type::Void;
        if self.match_token(&TokenKind::Arrow) {
            return_type = self.parse_type()?;
        }

        let mut version = None;
        if self.match_token(&TokenKind::Version) {
            if let TokenKind::IntLit(v) = self.peek_kind() {
                version = Some(*v as usize);
                self.advance();
            }
        }

        self.expect(TokenKind::LBrace)?;
        let mut requires = Vec::new();
        let mut guarantees = Vec::new();
        let mut effects = Vec::new();
        let mut emits = Vec::new();
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.check(&TokenKind::Requires) {
                self.advance();
                self.match_token(&TokenKind::Colon);
                if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                    let mut list = self.parse_string_list()?;
                    requires.append(&mut list);
                } else {
                    requires.push(self.parse_identifier_or_string()?);
                }
                self.match_token(&TokenKind::SemiColon);
            } else if self.check(&TokenKind::Guarantee) {
                self.advance();
                self.match_token(&TokenKind::Colon);
                if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                    let mut list = self.parse_string_list()?;
                    guarantees.append(&mut list);
                } else {
                    guarantees.push(self.parse_identifier_or_string()?);
                }
                self.match_token(&TokenKind::SemiColon);
            } else if self.check(&TokenKind::Effects) {
                self.advance();
                self.match_token(&TokenKind::Colon);
                if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                    let mut list = self.parse_string_list()?;
                    effects.append(&mut list);
                } else {
                    effects.push(self.parse_identifier_or_string()?);
                }
                self.match_token(&TokenKind::SemiColon);
            } else if self.check(&TokenKind::Emit) {
                self.advance();
                self.match_token(&TokenKind::Colon);
                if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                    let mut list = self.parse_string_list()?;
                    emits.append(&mut list);
                } else {
                    let ev_name = self.parse_identifier_or_keyword()?;
                    emits.push(ev_name.clone());
                    let mut args = Vec::new();
                    if self.match_token(&TokenKind::LParen) {
                        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                            args.push(self.parse_expression()?);
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(TokenKind::RParen)?;
                    }
                    statements.push(Statement::EmitEvent {
                        event_name: ev_name,
                        args,
                        span: self.current_span(),
                    });
                }
                self.match_token(&TokenKind::SemiColon);
            } else if self.check(&TokenKind::Version) {
                self.advance();
                self.match_token(&TokenKind::Colon);
                if let TokenKind::IntLit(v) = self.peek_kind() {
                    version = Some(*v as usize);
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
            } else {
                statements.push(self.parse_statement()?);
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(OperationDef {
            name,
            params,
            return_type,
            is_pub,
            requires,
            guarantees,
            effects,
            emits,
            version,
            body: Block {
                statements,
                span: span.clone(),
            },
            span,
        })
    }


    pub(crate) fn parse_trait(&mut self, is_pub: bool) -> Result<TraitDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Trait)?;

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected trait name, found {:?} at line {}", other, span.line)),
        };

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

        self.expect(TokenKind::LBrace)?;
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let m_span = self.current_span();
            self.expect(TokenKind::Fn)?;
            let m_name = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected trait method name, found {:?}", other)),
            };

            let mut m_generic_params = Vec::new();
            if self.match_token(&TokenKind::Less) {
                while !self.check(&TokenKind::Greater) && !self.check(&TokenKind::EOF) {
                    if let TokenKind::Ident(g) = self.advance().kind {
                        m_generic_params.push(g);
                    }
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::Greater)?;
            }

            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                let p_span = self.current_span();
                let is_mut = self.match_token(&TokenKind::Mut);
                let p_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected param name, found {:?}", other)),
                };
                let mut p_ty = Type::Void;
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

            let ret_ty = if self.match_token(&TokenKind::Arrow) {
                self.parse_type()?
            } else if matches!(self.peek_kind(), TokenKind::Ident(_)) || self.check(&TokenKind::LBracket) || self.check(&TokenKind::Star) {
                self.parse_type()?
            } else {
                Type::Void
            };

            if self.check(&TokenKind::LBrace) {
                let _body = self.parse_block()?;
            } else {
                self.match_token(&TokenKind::SemiColon);
            }

            methods.push(TraitMethodDef {
                name: m_name,
                generic_params: m_generic_params,
                params,
                return_type: ret_ty,
                span: m_span,
            });
        }
        self.expect(TokenKind::RBrace)?;

        Ok(TraitDef {
            name,
            generic_params,
            is_pub,
            methods,
            span,
        })
    }


    pub(crate) fn parse_impl(&mut self) -> Result<ImplBlock, String> {
        let span = self.current_span();
        self.expect(TokenKind::Impl)?;

        let first_ty = self.parse_type()?;
        let (trait_name, target_type) = if self.match_token(&TokenKind::For) {
            let tr_name = match &first_ty {
                Type::Custom(n) => n.clone(),
                _ => "Trait".to_string(),
            };
            let tgt = self.parse_type()?;
            (Some(tr_name), tgt)
        } else {
            (None, first_ty)
        };

        self.expect(TokenKind::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            methods.push(self.parse_function(true, Vec::new())?);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(ImplBlock {
            trait_name,
            target_type,
            methods,
            span,
        })
    }

    // ── 50 Super Revolutionary Feature-Oriented Paradigm Parsers ──

}
