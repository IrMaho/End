use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub fn parse_feature_def(&mut self, is_pub: bool, mut pending_directives: Vec<Directive>) -> Result<FeatureDef, String> {
        let span = self.current_span();
        if self.match_token(&TokenKind::Feature) {}

        let name = self.parse_identifier_or_keyword()?;
        let mut version = None;
        let mut owner = None;
        let mut parent = None;
        let mut architecture_template = None;
        let mut is_replaceable = false;
        let mut is_evolvable = false;

        // Directives like @version("2.1"), @owned("payments"), @evolvable
        while let TokenKind::Directive(d) = self.peek_kind() {
            let dir_name = d.clone();
            self.advance();
            let mut args = Vec::new();
            if self.match_token(&TokenKind::LParen) {
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::StringLit(s) | TokenKind::Ident(s) => args.push(s),
                        TokenKind::IntLit(i) => args.push(i.to_string()),
                        _ => {}
                    }
                    if !self.match_token(&TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::RParen)?;
            }
            if dir_name == "@version" {
                version = args.first().cloned();
            } else if dir_name == "@owned" || dir_name == "@owner" {
                owner = args.first().cloned();
            } else if dir_name == "@evolvable" {
                is_evolvable = true;
            }
            pending_directives.push(Directive { name: dir_name, args, span: span.clone() });
        }

        // extends Parent
        if self.match_token(&TokenKind::Extends) {
            parent = Some(self.parse_identifier_or_keyword()?);
        } else if let TokenKind::Ident(id) = self.peek_kind() {
            if id == "extends" {
                self.advance();
                parent = Some(self.parse_identifier_or_keyword()?);
            }
        }

        // : CleanFeature (Architecture Template)
        if self.match_token(&TokenKind::Colon) {
            architecture_template = Some(self.parse_identifier_or_keyword()?);
        }

        // Inline depends
        let mut needs = Vec::new();
        while self.match_token(&TokenKind::Depends) {
            let dep_name = self.parse_identifier_or_keyword()?;
            needs.push(FeatureDependency {
                name: dep_name,
                sub_contract: None,
                type_params: vec![],
                why: None,
                is_typed: false,
                span: self.current_span(),
            });
        }

        // Inline compose
        let mut compose = Vec::new();
        if self.match_token(&TokenKind::Compose) {
            self.expect(TokenKind::LBrace)?;
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                compose.push(self.parse_identifier_or_keyword()?);
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(FeatureDef {
                name,
                version,
                owner,
                parent,
                architecture_template,
                is_pub,
                is_replaceable,
                is_evolvable,
                api: None,
                implementations: vec![],
                needs,
                boundary: None,
                exposes: vec![],
                extensions: vec![],
                compose,
                contracts: vec![],
                invariants: vec![],
                tests: vec![],
                requires_capabilities: vec![],
                permissions: None,
                lifecycle: None,
                decisions: vec![],
                nested_features: vec![],
                forbids: vec![],
                allows: vec![],
                decorations: vec![],
                span,
            });
        }

        let mut api: Option<FeatureApi> = None;
        let mut implementations: Vec<FeatureImpl> = Vec::new();
        let mut boundary: Option<FeatureBoundary> = None;
        let mut exposes = Vec::new();
        let mut extensions = Vec::new();
        let mut contracts = Vec::new();
        let mut invariants = Vec::new();
        let mut tests = Vec::new();
        let mut requires_capabilities = Vec::new();
        let mut permissions: Option<FeaturePermission> = None;
        let mut lifecycle: Option<FeatureLifecycle> = None;
        let mut decisions = Vec::new();
        let mut nested_features = Vec::new();
        let mut forbids = Vec::new();
        let mut allows = Vec::new();
        let mut decorations = Vec::new();

        if self.check(&TokenKind::LBrace) {
            self.expect(TokenKind::LBrace)?;

            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                match self.peek_kind() {
                    TokenKind::Replaceable => {
                        self.advance();
                        is_replaceable = true;
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Evolvable => {
                        self.advance();
                        is_evolvable = true;
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Version => {
                        self.advance();
                        self.match_token(&TokenKind::Colon);
                        version = Some(self.parse_identifier_or_string()?);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::OwnedBy | TokenKind::Owned => {
                        self.advance();
                        self.match_token(&TokenKind::Colon);
                        owner = Some(self.parse_identifier_or_string()?);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Architecture => {
                        self.advance();
                        self.match_token(&TokenKind::Colon);
                        architecture_template = Some(self.parse_identifier_or_keyword()?);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Api => {
                        self.parse_feature_api_member(&mut api)?;
                    }
                    TokenKind::Impl | TokenKind::Implementation => {
                        self.parse_feature_impl_member(&mut implementations)?;
                    }
                    TokenKind::Needs => {
                        self.parse_feature_needs_member(&mut needs)?;
                    }
                    TokenKind::Boundary => {
                        self.parse_feature_boundary_member(&mut boundary)?;
                    }
                    TokenKind::Expose | TokenKind::Exposes => {
                        self.parse_feature_exposes_member(&mut exposes)?;
                    }
                    TokenKind::Extension | TokenKind::ExtensionPoint => {
                        self.parse_feature_extension_point_member(&mut extensions)?;
                    }
                    TokenKind::Contract => {
                        self.parse_feature_contract_member(&mut contracts)?;
                    }
                    TokenKind::Invariant => {
                        self.advance();
                        if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                invariants.push(self.parse_expression()?);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            invariants.push(self.parse_expression()?);
                            self.match_token(&TokenKind::SemiColon);
                        }
                    }
                    TokenKind::Test | TokenKind::Testing => {
                        self.advance();
                        let t_span = self.current_span();
                        self.expect(TokenKind::LBrace)?;
                        let mut test_statements = Vec::new();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            match self.parse_statement() {
                                Ok(stmt) => test_statements.push(stmt),
                                Err(_) => {
                                    self.synchronize();
                                    if self.check(&TokenKind::RBrace) || self.check(&TokenKind::EOF) {
                                        break;
                                    }
                                }
                            }
                        }
                        self.expect(TokenKind::RBrace)?;
                        tests.push(FunctionDef {
                            name: format!("{}_test", name),
                            generic_params: vec![],
                            is_pub: false,
                            params: vec![],
                            return_type: Type::Void,
                            body: Block { statements: test_statements, span: t_span.clone() },
                            directives: vec![],
                            morphic_param: None,
                            span: t_span,
                        });
                    }
                    TokenKind::Requires => {
                        self.advance();
                        if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                requires_capabilities.push(self.parse_identifier_or_keyword()?);
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            requires_capabilities.push(self.parse_identifier_or_keyword()?);
                            self.match_token(&TokenKind::SemiColon);
                        }
                    }
                    TokenKind::Allow => {
                        self.advance();
                        if self.check(&TokenKind::LBrace) {
                            self.advance();
                            let perm = permissions.get_or_insert_with(|| FeaturePermission { allow: vec![], deny: vec![], span: self.current_span() });
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                perm.allow.push(self.parse_identifier_or_keyword()?);
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            allows.push(self.parse_identifier_or_keyword()?);
                            self.match_token(&TokenKind::SemiColon);
                        }
                    }
                    TokenKind::Deny => {
                        self.advance();
                        let perm = permissions.get_or_insert_with(|| FeaturePermission { allow: vec![], deny: vec![], span: self.current_span() });
                        if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                perm.deny.push(self.parse_identifier_or_keyword()?);
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        } else {
                            perm.deny.push(self.parse_identifier_or_keyword()?);
                            self.match_token(&TokenKind::SemiColon);
                        }
                    }
                    TokenKind::Policy => {
                        self.advance();
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
                    }
                    TokenKind::Forbid => {
                        self.advance();
                        forbids.push(self.parse_identifier_or_keyword()?);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Compose => {
                        self.advance();
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
                    }
                    TokenKind::Decorate => {
                        self.advance();
                        if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                decorations.push(self.parse_identifier_or_keyword()?);
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        }
                    }
                    TokenKind::Lifecycle => {
                        self.parse_feature_lifecycle_member(&mut lifecycle)?;
                    }
                    TokenKind::Decision => {
                        self.parse_feature_decision_member(&mut decisions)?;
                    }
                    TokenKind::Feature => {
                        let nested = self.parse_feature_def(false, vec![])?;
                        nested_features.push(nested);
                    }
                    TokenKind::Skills => {
                        self.advance();
                        self.match_token(&TokenKind::Colon);
                        let mut list = self.parse_string_list()?;
                        requires_capabilities.append(&mut list);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Tasks => {
                        self.advance();
                        self.match_token(&TokenKind::Colon);
                        let list = self.parse_string_list()?;
                        for t in list {
                            decisions.push(FeatureDecision { target: t, reason: "".to_string(), span: self.current_span() });
                        }
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Requirement => {
                        self.advance();
                        self.match_token(&TokenKind::Colon);
                        let req_text = self.parse_identifier_or_string()?;
                        let c_span = self.current_span();
                        contracts.push(FeatureContractClause { rule: req_text, is_negative: false, span: c_span });
                        self.match_token(&TokenKind::SemiColon);
                    }
                    TokenKind::Ident(s) if s == "extension_point" => {
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
                    }
                    TokenKind::Ident(s) => {
                        self.parse_feature_ident_member(&s.clone(), &mut version, &mut owner, &mut architecture_template, &mut needs, &mut compose, &mut decorations)?;
                    }
                    _ => {
                        if self.match_token(&TokenKind::SemiColon) {
                            continue;
                        }
                        self.advance();
                    }
                }
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            self.match_token(&TokenKind::SemiColon);
        }

        Ok(FeatureDef {
            name,
            version,
            owner,
            parent,
            architecture_template,
            is_pub,
            is_replaceable,
            is_evolvable,
            api,
            implementations,
            needs,
            boundary,
            exposes,
            extensions,
            compose,
            contracts,
            invariants,
            tests,
            requires_capabilities,
            permissions,
            lifecycle,
            decisions,
            nested_features,
            forbids,
            allows,
            decorations,
            span,
        })
    }
}
