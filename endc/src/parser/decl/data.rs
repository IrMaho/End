use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_enum(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<EnumDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Enum)?;

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected enum name, found {:?} at line {}", other, span.line)),
        };
        self.enum_names.insert(name.clone());

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
        let mut variants = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let vspan = self.current_span();
            let vname = self.parse_identifier_or_keyword()?;

            let mut payload = None;
            if self.match_token(&TokenKind::LParen) {
                let mut ptypes = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    ptypes.push(self.parse_type()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                if ptypes.len() == 1 {
                    payload = Some(ptypes.remove(0));
                } else if !ptypes.is_empty() {
                    payload = Some(Type::Custom(format!("tuple_{}", ptypes.len())));
                }
            } else if self.match_token(&TokenKind::Equal) {
                let _ = self.parse_expression()?;
            }

            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);

            variants.push(EnumVariant {
                name: vname,
                payload,
                span: vspan,
            });
        }

        self.expect(TokenKind::RBrace)?;

        Ok(EnumDef {
            name,
            generic_params,
            is_pub,
            variants,
            directives,
            span,
        })
    }


    pub(crate) fn parse_struct(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<StructDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Struct)?;

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected struct name, found {:?} at line {}", other, span.line)),
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

        let mut fields = Vec::new();
        if self.match_token(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let field_span = self.current_span();
                while let TokenKind::Directive(_) = self.peek_kind() {
                    self.advance();
                    if self.match_token(&TokenKind::LParen) {
                        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                            self.advance();
                        }
                        self.match_token(&TokenKind::RParen);
                    }
                }
                let is_field_pub = self.match_token(&TokenKind::Pub);
                let raw_name = self.parse_identifier_or_keyword()?;
                let field_name = if raw_name == "val" {
                    self.parse_identifier_or_keyword()?
                } else {
                    raw_name
                };

                let mut field_type = Type::Void;
                if self.match_token(&TokenKind::Colon) {
                    field_type = self.parse_type()?;
                }
                if self.match_token(&TokenKind::By) {
                    let _ = self.parse_identifier_or_keyword()?;
                }
                if self.match_token(&TokenKind::Equal) {
                    let _ = self.parse_expression()?;
                }
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);

                fields.push(StructField {
                    name: field_name,
                    field_type,
                    is_pub: is_field_pub,
                    span: field_span,
                });
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            self.match_token(&TokenKind::SemiColon);
        }

        Ok(StructDef {
            name,
            generic_params,
            is_pub,
            is_partial: false,
            is_sealed: false,
            is_extension_only: false,
            is_open: false,
            is_closed: false,
            friend_modules: vec![],
            extension_points: vec![],
            fields,
            directives,
            span,
        })
    }


    pub(crate) fn parse_event(&mut self, is_pub: bool) -> Result<EventDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Event)?;
        let name = self.parse_identifier_or_keyword()?;
        let mut generic_params = Vec::new();
        let mut parent_event = None;
        let mut channel_kind = None;
        let mut channel_target = None;
        let mut with_attributes = Vec::new();
        let mut fields = Vec::new();
        let directives = Vec::new();

        // Generic event streams: event Stream<T>
        if self.match_token(&TokenKind::Less) {
            while !self.check(&TokenKind::Greater) && !self.check(&TokenKind::EOF) {
                generic_params.push(self.parse_identifier_or_keyword()?);
                if !self.match_token(&TokenKind::Comma) { break; }
            }
            self.expect(TokenKind::Greater)?;
        }

        // Parenthesized payload fields: event UserLogin(user_id: str, ip: str)
        if self.match_token(&TokenKind::LParen) {
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                let f_span = self.current_span();
                let f_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Colon);
                let f_type = self.parse_type().unwrap_or(Type::Void);
                fields.push(StructField {
                    name: f_name,
                    field_type: f_type,
                    is_pub: true,
                    span: f_span,
                });
                if !self.match_token(&TokenKind::Comma) { break; }
            }
            self.expect(TokenKind::RParen)?;
        }

        // Channel definitions:
        // event Client <-> Server
        // event Inbound -> Outbound
        // event Sensor <~> Hub
        if self.match_token(&TokenKind::BiArrow) {
            channel_kind = Some(EventChannelKind::Duplex);
            channel_target = Some(self.parse_identifier_or_keyword()?);
        } else if self.match_token(&TokenKind::Arrow) {
            channel_kind = Some(EventChannelKind::SingleDirection);
            channel_target = Some(self.parse_identifier_or_keyword()?);
        } else if self.match_token(&TokenKind::TildeBiArrow) {
            channel_kind = Some(EventChannelKind::HalfDuplex);
            channel_target = Some(self.parse_identifier_or_keyword()?);
        }

        // Inherited/subtyped event: event SystemAlert : ErrorEvent
        if self.match_token(&TokenKind::Colon) {
            parent_event = Some(self.parse_identifier_or_keyword()?);
        }

        // Event options & attributes: with ring_buffer(4096), wal_persisted, retention(30d)
        if self.match_token(&TokenKind::With) {
            while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                let attr_name = self.parse_identifier_or_keyword()?;
                let mut attr_full = attr_name;
                if self.match_token(&TokenKind::LParen) {
                    attr_full.push('(');
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        let arg = self.parse_identifier_or_keyword_or_int().unwrap_or_else(|_| "1".to_string());
                        attr_full.push_str(&arg);
                        if self.match_token(&TokenKind::Comma) { attr_full.push(','); }
                    }
                    self.expect(TokenKind::RParen)?;
                    attr_full.push(')');
                }
                with_attributes.push(attr_full);
                if !self.match_token(&TokenKind::Comma) { break; }
            }
        }

        if self.match_token(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let f_span = self.current_span();
                let f_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Colon);
                let f_type = self.parse_type().unwrap_or(Type::Void);
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
                fields.push(StructField {
                    name: f_name,
                    field_type: f_type,
                    is_pub: true,
                    span: f_span,
                });
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            self.match_token(&TokenKind::SemiColon);
        }

        Ok(EventDef {
            name,
            is_pub,
            generic_params,
            parent_event,
            channel_kind,
            channel_target,
            with_attributes,
            fields,
            directives,
            span,
        })
    }


    pub(crate) fn parse_event_hub(&mut self, is_pub: bool) -> Result<EventHubDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Hub)?;
        let name = self.parse_identifier_or_keyword()?;
        self.expect(TokenKind::LBrace)?;
        let mut owns_events = Vec::new();
        let mut handlers = Vec::new();
        let mut routes = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.check(&TokenKind::Owns) {
                self.advance();
                self.match_token(&TokenKind::Colon);
                if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                    let mut list = self.parse_string_list()?;
                    owns_events.append(&mut list);
                } else {
                    owns_events.push(self.parse_identifier_or_string()?);
                }
                self.match_token(&TokenKind::SemiColon);
            } else if self.check(&TokenKind::On) {
                self.advance();
                let h_span = self.current_span();
                let event_name = self.parse_identifier_or_keyword()?;
                let mut handler_op = None;
                let mut body = None;

                if self.match_token(&TokenKind::Arrow) {
                    let op_expr = self.parse_expression()?;
                    handler_op = Some(op_expr);
                    self.match_token(&TokenKind::SemiColon);
                } else if self.check(&TokenKind::LBrace) {
                    let blk = self.parse_block()?;
                    body = Some(blk);
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }

                handlers.push(EventHandlerDef {
                    event_name,
                    handler_op,
                    body,
                    span: h_span,
                });
            } else if self.check(&TokenKind::Ident("route".to_string())) {
                self.advance();
                let pattern = self.parse_identifier_or_string()?;
                self.expect(TokenKind::Arrow)?;
                let dest = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                routes.push((pattern, dest));
            } else {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(EventHubDef {
            name,
            is_pub,
            owns_events,
            handlers,
            routes,
            span,
        })
    }

}
