use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_feature_api_member(&mut self, api: &mut Option<FeatureApi>) -> Result<(), String> {
                        self.advance();
                        let mut api_functions = Vec::new();
                        let mut api_structs = Vec::new();
                        let mut api_enums = Vec::new();
                        let mut api_traits = Vec::new();
                        let mut exposed_symbols = Vec::new();
                        let mut raw_signatures = Vec::new();
                        let api_span = self.current_span();
                        self.expect(TokenKind::LBrace)?;
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            if self.match_token(&TokenKind::Fn) {
                                let sig_name = self.parse_identifier_or_keyword()?;
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
                                let body = if self.check(&TokenKind::LBrace) {
                                    self.parse_block()?
                                } else {
                                    self.match_token(&TokenKind::SemiColon);
                                    Block { statements: vec![], span: api_span.clone() }
                                };
                                exposed_symbols.push(sig_name.clone());
                                raw_signatures.push(sig_name.clone());
                                api_functions.push(FunctionDef {
                                    name: sig_name,
                                    generic_params: vec![],
                                    is_pub: true,
                                    params,
                                    return_type,
                                    body,
                                    directives: vec![],
                                    morphic_param: None,
                                    span: api_span.clone(),
                                });
                            } else if self.match_token(&TokenKind::Struct) {
                                let s = self.parse_struct(true, vec![])?;
                                api_structs.push(s);
                            } else if self.match_token(&TokenKind::Enum) {
                                let e = self.parse_enum(true, vec![])?;
                                api_enums.push(e);
                            } else if self.match_token(&TokenKind::Trait) {
                                let t = self.parse_trait(true)?;
                                api_traits.push(t);
                            } else {
                                let sig_name = self.parse_identifier_or_keyword()?;
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
                                self.match_token(&TokenKind::SemiColon);
                                exposed_symbols.push(sig_name.clone());
                                raw_signatures.push(sig_name.clone());
                                api_functions.push(FunctionDef {
                                    name: sig_name,
                                    generic_params: vec![],
                                    is_pub: true,
                                    params,
                                    return_type,
                                    body: Block { statements: vec![], span: api_span.clone() },
                                    directives: vec![],
                                    morphic_param: None,
                                    span: api_span.clone(),
                                });
                            }
                        }
                        self.expect(TokenKind::RBrace)?;
                        *api = Some(FeatureApi {
                            functions: api_functions,
                            structs: api_structs,
                            enums: api_enums,
                            traits: api_traits,
                            exposed_symbols,
                            raw_signatures,
                            span: api_span,
                        });
        Ok(())
    }

    pub(crate) fn parse_feature_impl_member(&mut self, implementations: &mut Vec<FeatureImpl>) -> Result<(), String> {
                        self.advance();
                        let impl_span = self.current_span();
                        let mut impl_name = None;
                        let mut target_contract = None;
                        if !self.check(&TokenKind::LBrace) {
                            let n = self.parse_identifier_or_keyword()?;
                            if n == "implements" || n == "for" {
                                target_contract = Some(self.parse_identifier_or_keyword()?);
                            } else {
                                impl_name = Some(n);
                                if self.match_token(&TokenKind::Implements) || self.match_token(&TokenKind::Colon) {
                                    target_contract = Some(self.parse_identifier_or_keyword()?);
                                }
                            }
                        }
                        self.expect(TokenKind::LBrace)?;
                        let mut fns = Vec::new();
                        let mut sts = Vec::new();
                        let mut stmts = Vec::new();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            if self.check(&TokenKind::Fn) {
                                fns.push(self.parse_function(true, vec![])?);
                            } else if self.check(&TokenKind::Struct) {
                                sts.push(self.parse_struct(false, vec![])?);
                            } else {
                                match self.parse_statement() {
                                    Ok(stmt) => stmts.push(stmt),
                                    Err(_) => {
                                        self.synchronize();
                                        if self.check(&TokenKind::RBrace) || self.check(&TokenKind::EOF) {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        self.expect(TokenKind::RBrace)?;
                        implementations.push(FeatureImpl {
                            name: impl_name,
                            target_contract,
                            functions: fns,
                            structs: sts,
                            statements: stmts,
                            span: impl_span,
                        });
        Ok(())
    }

    pub(crate) fn parse_feature_needs_member(&mut self, needs: &mut Vec<FeatureDependency>) -> Result<(), String> {
                        self.advance();
                        if self.match_token(&TokenKind::Colon) {
                            if self.match_token(&TokenKind::LBracket) {
                                while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::EOF) {
                                    let dep_name = self.parse_identifier_or_keyword()?;
                                    needs.push(FeatureDependency {
                                        name: dep_name,
                                        sub_contract: None,
                                        type_params: vec![],
                                        why: None,
                                        is_typed: false,
                                        span: self.current_span(),
                                    });
                                    if !self.match_token(&TokenKind::Comma) { break; }
                                }
                                self.expect(TokenKind::RBracket)?;
                                self.match_token(&TokenKind::SemiColon);
                            } else {
                                let dep_name = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::SemiColon);
                                needs.push(FeatureDependency {
                                    name: dep_name,
                                    sub_contract: None,
                                    type_params: vec![],
                                    why: None,
                                    is_typed: false,
                                    span: self.current_span(),
                                });
                            }
                        } else if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                let dep_span = self.current_span();
                                let dep_name = self.parse_identifier_or_keyword()?;
                                let mut sub_contract = None;
                                if self.match_token(&TokenKind::Dot) {
                                    sub_contract = Some(self.parse_identifier_or_keyword()?);
                                }
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                                needs.push(FeatureDependency {
                                    name: dep_name,
                                    sub_contract,
                                    type_params: vec![],
                                    why: None,
                                    is_typed: false,
                                    span: dep_span,
                                });
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            let dep_span = self.current_span();
                            let dep_name = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::SemiColon);
                            needs.push(FeatureDependency {
                                name: dep_name,
                                sub_contract: None,
                                type_params: vec![],
                                why: None,
                                is_typed: false,
                                span: dep_span,
                            });
                        }
        Ok(())
    }

    pub(crate) fn parse_feature_boundary_member(&mut self, boundary: &mut Option<FeatureBoundary>) -> Result<(), String> {
                        self.advance();
                        let b_span = self.current_span();
                        self.expect(TokenKind::LBrace)?;
                        let mut layers = Vec::new();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            layers.push(self.parse_identifier_or_string()?);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                        *boundary = Some(FeatureBoundary { layers, span: b_span });
        Ok(())
    }

    pub(crate) fn parse_feature_exposes_member(&mut self, exposes: &mut Vec<String>) -> Result<(), String> {
                        self.advance();
                        if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                exposes.push(self.parse_identifier_or_keyword()?);
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            exposes.push(self.parse_identifier_or_keyword()?);
                            self.match_token(&TokenKind::SemiColon);
                        }
        Ok(())
    }

    pub(crate) fn parse_feature_extension_point_member(&mut self, extensions: &mut Vec<FeatureExtensionPoint>) -> Result<(), String> {
                        self.advance();
                        let ext_span = self.current_span();
                        let ext_name = self.parse_identifier_or_keyword()?;
                        let mut priority = None;
                        if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                let k = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::Colon);
                                if k == "priority" {
                                    if let TokenKind::IntLit(p) = self.peek_kind() {
                                        priority = Some(*p);
                                        self.advance();
                                    }
                                } else {
                                    let _ = self.parse_identifier_or_string();
                                }
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            self.match_token(&TokenKind::SemiColon);
                        }
                        extensions.push(FeatureExtensionPoint {
                            name: ext_name,
                            allowed_types: vec![],
                            priority,
                            span: ext_span,
                        });
        Ok(())
    }

    pub(crate) fn parse_feature_contract_member(&mut self, contracts: &mut Vec<FeatureContractClause>) -> Result<(), String> {
                        self.advance();
                        let c_span = self.current_span();
                        self.expect(TokenKind::LBrace)?;
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let k = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            if self.check(&TokenKind::LBracket) {
                                let list = self.parse_string_list()?;
                                for item in list {
                                    contracts.push(FeatureContractClause {
                                        rule: format!("{}: {}", k, item),
                                        is_negative: false,
                                        span: c_span.clone(),
                                    });
                                }
                            } else {
                                let val = self.parse_identifier_or_string()?;
                                contracts.push(FeatureContractClause {
                                    rule: format!("{}: {}", k, val),
                                    is_negative: false,
                                    span: c_span.clone(),
                                });
                            }
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
        Ok(())
    }

    pub(crate) fn parse_feature_lifecycle_member(&mut self, lifecycle: &mut Option<FeatureLifecycle>) -> Result<(), String> {
        self.advance();
        let l_span = self.current_span();
        self.expect(TokenKind::LBrace)?;
        let mut state = "stable".to_string();
        let mut replace_with = None;
        let mut migration_path = None;
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let k = self.parse_identifier_or_keyword()?;
            if k == "experimental" || k == "stable" || k == "deprecated" {
                state = k;
            } else if k == "replace_with" {
                self.match_token(&TokenKind::Colon);
                replace_with = Some(self.parse_identifier_or_keyword()?);
            } else if k == "migration" {
                self.match_token(&TokenKind::Colon);
                migration_path = Some(self.parse_identifier_or_keyword()?);
            }
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;
        *lifecycle = Some(FeatureLifecycle { state, replace_with, migration_path, span: l_span });
        Ok(())
    }

    pub(crate) fn parse_feature_decision_member(&mut self, decisions: &mut Vec<FeatureDecision>) -> Result<(), String> {
                        self.advance();
                        let d_span = self.current_span();
                        let target = self.parse_identifier_or_keyword()?;
                        let mut reason = String::new();
                        self.expect(TokenKind::LBrace)?;
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let k = self.parse_identifier_or_keyword()?;
                            if k == "reason" {
                                self.match_token(&TokenKind::Colon);
                                reason = self.parse_identifier_or_string()?;
                            }
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                        decisions.push(FeatureDecision { target, reason, span: d_span });
        Ok(())
    }

    pub(crate) fn parse_feature_ident_member(
        &mut self,
        s: &str,
        version: &mut Option<String>,
        owner: &mut Option<String>,
        architecture_template: &mut Option<String>,
        needs: &mut Vec<FeatureDependency>,
        compose: &mut Vec<String>,
        decorations: &mut Vec<String>,
    ) -> Result<(), String> {
        let k = s.to_string();
        self.advance();
        if k == "version" {
            self.match_token(&TokenKind::Colon);
            *version = Some(self.parse_identifier_or_string()?);
            self.match_token(&TokenKind::SemiColon);
        } else if k == "owner" {
            self.match_token(&TokenKind::Colon);
            *owner = Some(self.parse_identifier_or_string()?);
            self.match_token(&TokenKind::SemiColon);
        } else if k == "architecture" {
            self.match_token(&TokenKind::Colon);
            *architecture_template = Some(self.parse_identifier_or_keyword()?);
            self.match_token(&TokenKind::SemiColon);
                        } else if k == "needs" {
                            self.match_token(&TokenKind::Colon);
                            if self.match_token(&TokenKind::LBracket) {
                                while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::EOF) {
                                    let dep_name = self.parse_identifier_or_keyword()?;
                                    needs.push(FeatureDependency {
                                        name: dep_name,
                                        sub_contract: None,
                                        type_params: vec![],
                                        why: None,
                                        is_typed: false,
                                        span: self.current_span(),
                                    });
                                    if !self.match_token(&TokenKind::Comma) { break; }
                                }
                                self.expect(TokenKind::RBracket)?;
                                self.match_token(&TokenKind::SemiColon);
                            }
                        } else if k == "compose" {
                            if self.match_token(&TokenKind::Colon) {
                                if self.match_token(&TokenKind::LBracket) {
                                    while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::EOF) {
                                        compose.push(self.parse_identifier_or_keyword()?);
                                        if !self.match_token(&TokenKind::Comma) { break; }
                                    }
                                    self.expect(TokenKind::RBracket)?;
                                    self.match_token(&TokenKind::SemiColon);
                                }
                            } else if self.check(&TokenKind::LBrace) {
                                self.advance();
                                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                    compose.push(self.parse_identifier_or_keyword()?);
                                    self.match_token(&TokenKind::Comma);
                                    self.match_token(&TokenKind::SemiColon);
                                }
                                self.expect(TokenKind::RBrace)?;
                            }
                        } else if k == "decorate" {
                            if self.check(&TokenKind::LBrace) {
                                self.advance();
                                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                    decorations.push(self.parse_identifier_or_keyword()?);
                                    self.match_token(&TokenKind::Comma);
                                    self.match_token(&TokenKind::SemiColon);
                                }
                                self.expect(TokenKind::RBrace)?;
                            }
                        } else if k == "policy" || k == "rules" || k == "constraints" {
                            if self.check(&TokenKind::LBrace) {
                                self.advance();
                                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                    let _ = self.parse_identifier_or_string();
                                    self.match_token(&TokenKind::Comma);
                                    self.match_token(&TokenKind::SemiColon);
                                }
                                self.expect(TokenKind::RBrace)?;
                            } else {
                                self.match_token(&TokenKind::SemiColon);
                            }
                        } else if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                self.advance();
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            self.match_token(&TokenKind::SemiColon);
                        }
        Ok(())
    }
}
