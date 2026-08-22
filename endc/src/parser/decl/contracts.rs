use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub fn parse_contract_def(&mut self, _is_pub: bool) -> Result<ContractDef, String> {
        let span = self.current_span();
        if self.match_token(&TokenKind::Contract) {}
        let mut name = String::new();
        while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::EOF) {
            let part = self.parse_identifier_or_keyword()?;
            if part != "Module" && part != "mod" {
                name = part;
            }
        }
        self.expect(TokenKind::LBrace)?;
        let mut methods = Vec::new();
        let mut clauses = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.match_token(&TokenKind::Must) {
                let mut text = "must".to_string();
                while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::Ident(s) | TokenKind::StringLit(s) => {
                            text.push(' ');
                            text.push_str(&s);
                        }
                        _ => {}
                    }
                }
                self.match_token(&TokenKind::SemiColon);
                clauses.push(text);
            } else if self.check(&TokenKind::Guarantees) {
                let key = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Colon);
                let list = self.parse_string_list()?;
                clauses.push(format!("{}: [{}]", key, list.join(", ")));
                for item in &list {
                    clauses.push(item.clone());
                }
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            } else {
                let m_span = self.current_span();
                let m_name = self.parse_identifier_or_keyword()?;
                if self.match_token(&TokenKind::Colon) {
                    let list = self.parse_string_list()?;
                    clauses.push(format!("{}: [{}]", m_name, list.join(", ")));
                    for item in &list {
                        clauses.push(item.clone());
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                    continue;
                }
                let mut params = Vec::new();
                if self.match_token(&TokenKind::LParen) {
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        let is_mut = self.match_token(&TokenKind::Mut);
                        let p_name = self.parse_identifier_or_keyword()?;
                        let mut param_type = Type::Void;
                        if self.match_token(&TokenKind::Colon) {
                            param_type = self.parse_type()?;
                        }
                        params.push(FunctionParam { name: p_name, param_type, is_mut, span: self.current_span() });
                        if !self.match_token(&TokenKind::Comma) { break; }
                    }
                    self.expect(TokenKind::RParen)?;
                }
                let mut return_type = Type::Void;
                if self.match_token(&TokenKind::Arrow) {
                    return_type = self.parse_type()?;
                }
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
                methods.push(TraitMethodDef {
                    name: m_name,
                    generic_params: vec![],
                    params,
                    return_type,
                    span: m_span,
                });
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(ContractDef {
            name,
            methods,
            clauses,
            is_evolved: false,
            span,
        })
    }

    pub fn parse_architecture_rule_or_template(&mut self) -> Result<Statement, String> {
        let span = self.current_span();
        self.expect(TokenKind::Architecture)?;
        let name = self.parse_identifier_or_keyword()?;
        self.expect(TokenKind::LBrace)?;

        let mut allowed_flows = Vec::new();
        let mut forbidden_flows = Vec::new();
        let mut layers = Vec::new();
        let mut rules = Vec::new();
        let mut invariants = Vec::new();
        let mut cycle_free = false;
        let mut max_depth = None;

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let key = self.parse_identifier_or_keyword()?;
            if self.match_token(&TokenKind::Colon) {
                match key.as_str() {
                    "layers" => {
                        let mut list = self.parse_string_list()?;
                        layers.append(&mut list);
                    }
                    "rules" => {
                        let mut list = self.parse_string_list()?;
                        rules.append(&mut list);
                    }
                    "invariants" => {
                        let mut list = self.parse_string_list()?;
                        invariants.append(&mut list);
                    }
                    "directions" => {
                        while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let src = self.parse_identifier_or_keyword()?;
                            if self.match_token(&TokenKind::Arrow) {
                                let dst = self.parse_identifier_or_keyword()?;
                                allowed_flows.push((src, dst));
                            } else if self.match_token(&TokenKind::BangArrow) {
                                let dst = self.parse_identifier_or_keyword()?;
                                forbidden_flows.push((src, dst));
                            }
                            if !self.match_token(&TokenKind::Comma) { break; }
                        }
                    }
                    "cycle_free" => {
                        if self.match_token(&TokenKind::True) {
                            cycle_free = true;
                        } else if self.match_token(&TokenKind::False) {
                            cycle_free = false;
                        }
                    }
                    "max_depth" => {
                        if let TokenKind::IntLit(n) = self.peek_kind() {
                            max_depth = Some(*n as usize);
                            self.advance();
                        }
                    }
                    _ => {
                        if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                            let _ = self.parse_string_list();
                        } else {
                            let _ = self.parse_identifier_or_string();
                        }
                    }
                }
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            } else if self.match_token(&TokenKind::Arrow) {
                let right = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
                allowed_flows.push((key, right));
            } else if self.match_token(&TokenKind::BangArrow) {
                let right = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
                forbidden_flows.push((key, right));
            } else {
                layers.push(key);
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(Statement::ArchitectureDecl {
            name: name.clone(),
            layers: layers.clone(),
            rules: rules.clone(),
            directions: allowed_flows.clone(),
            invariants: invariants.clone(),
            cycle_free,
            max_depth,
            span: span.clone(),
        })
    }

    pub fn parse_feature_migration(&mut self) -> Result<FeatureMigrationDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Migration)?;
        let feature_name = self.parse_identifier_or_keyword()?;
        self.match_token(&TokenKind::From);
        let from_version = self.parse_identifier_or_keyword_or_int()?;
        if !self.match_token(&TokenKind::To) {
            self.match_token(&TokenKind::Arrow);
        }
        let to_version = self.parse_identifier_or_keyword_or_int()?;
        self.expect(TokenKind::LBrace)?;

        let mut renames = Vec::new();
        let mut replacements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.match_token(&TokenKind::Rename) {
                let old_sym = self.parse_identifier_or_string()?;
                if self.match_token(&TokenKind::LParen) {
                    self.match_token(&TokenKind::RParen);
                }
                if !self.match_token(&TokenKind::To) {
                    self.expect(TokenKind::Arrow)?;
                }
                let new_sym = self.parse_identifier_or_string()?;
                if self.match_token(&TokenKind::LParen) {
                    self.match_token(&TokenKind::RParen);
                }
                self.match_token(&TokenKind::SemiColon);
                renames.push((old_sym, new_sym));
            } else if self.match_token(&TokenKind::Replace) {
                let old_sym = self.parse_identifier_or_string()?;
                if !self.match_token(&TokenKind::With) {
                    self.match_token(&TokenKind::Arrow);
                }
                let new_sym = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                replacements.push((old_sym, new_sym));
            } else {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace)?;

        Ok(FeatureMigrationDef {
            feature_name,
            from_version,
            to_version,
            renames,
            replacements,
            span,
        })
    }


}
