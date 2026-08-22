use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_module_def(&mut self, is_pub: bool, _directives: Vec<Directive>) -> Result<ModuleDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Mod)?;
        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected module name, found {:?}", other)),
        };
        let mut parent = None;
        if self.match_token(&TokenKind::Derives) {
            parent = match self.advance().kind {
                TokenKind::Ident(p) => Some(p),
                other => return Err(format!("Expected parent module name after derives, found {:?}", other)),
            };
        }
        self.expect(TokenKind::LBrace)?;
        let mut structs = Vec::new();
        let mut functions = Vec::new();
        let mut overrides = Vec::new();
        let mut statements = Vec::new();
        let mut responsibility = None;
        let mut owns = Vec::new();
        let mut exposes = Vec::new();
        let mut depends = Vec::new();
        let mut depends_only = None;
        let mut forbid = Vec::new();
        let mut is_sealed = false;
        let mut purity = None;
        let mut cohesion = None;

        let mut facets = ModuleFacets::default();
        let mut has_facets = false;
        let mut contract = ModuleContract::default();
        let mut has_contract = false;
        let mut skills = Vec::new();
        let mut is_evolvable_mod = _directives.iter().any(|d| d.name == "@evolvable" || d.name == "evolvable");

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let mut pending_directives = Vec::new();
            while let TokenKind::Directive(d) = self.peek_kind() {
                let dir_name = d.clone();
                let dir_span = self.current_span();
                self.advance();
                if dir_name == "@evolvable" || dir_name == "evolvable" {
                    is_evolvable_mod = true;
                }
                pending_directives.push(Directive {
                    name: dir_name,
                    args: Vec::new(),
                    span: dir_span,
                });
            }
            let current_kind = self.peek_kind().clone();
            match current_kind {
                TokenKind::Responsibility => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    responsibility = Some(self.parse_identifier_or_string()?);
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Requires => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    has_contract = true;
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        contract.requires.append(&mut list);
                    } else {
                        contract.requires.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Provides => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    has_contract = true;
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        contract.provides.append(&mut list);
                    } else {
                        contract.provides.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Guarantee | TokenKind::Guarantees => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    has_contract = true;
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        contract.guarantees.append(&mut list);
                    } else {
                        contract.guarantees.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Skill | TokenKind::Skills => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        skills.append(&mut list);
                    } else {
                        skills.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Api if self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.api.push(self.parse_function(true, vec![])?);
                        } else if self.match_token(&TokenKind::Pub) && self.check(&TokenKind::Fn) {
                            facets.api.push(self.parse_function(true, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Ident(ref s) if s == "api" && self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.api.push(self.parse_function(true, vec![])?);
                        } else if self.match_token(&TokenKind::Pub) && self.check(&TokenKind::Fn) {
                            facets.api.push(self.parse_function(true, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Implementation | TokenKind::Impl if self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.implementation.push(self.parse_function(false, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Ident(ref s) if (s == "implementation" || s == "impl") && self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.implementation.push(self.parse_function(false, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Test | TokenKind::Testing if self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.tests.push(self.parse_function(false, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Ident(ref s) if (s == "tests" || s == "testing") && self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.tests.push(self.parse_function(false, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Extension | TokenKind::Extend if self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.extension.push(self.parse_function(false, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Ident(ref s) if (s == "extension" || s == "extensions") && self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Fn) {
                            facets.extension.push(self.parse_function(false, vec![])?);
                        } else {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Architecture if self.peek_next_kind() == Some(&TokenKind::LBrace) => {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    has_facets = true;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        facets.architecture.push(self.parse_identifier_or_string()?);
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Owns => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        owns.append(&mut list);
                    } else {
                        owns.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Exposes => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        exposes.append(&mut list);
                    } else {
                        exposes.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::DependsOnly => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let list = self.parse_string_list()?;
                        depends_only = Some(list);
                    } else {
                        depends_only = Some(vec![self.parse_identifier_or_string()?]);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Depends => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        depends.append(&mut list);
                    } else {
                        depends.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Forbid => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                        let mut list = self.parse_string_list()?;
                        forbid.append(&mut list);
                    } else {
                        forbid.push(self.parse_identifier_or_string()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Boundary => {
                    self.advance();
                    if self.match_token(&TokenKind::Sealed) {
                        is_sealed = true;
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Sealed => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    if self.match_token(&TokenKind::True) {
                        is_sealed = true;
                    } else if self.match_token(&TokenKind::False) {
                        is_sealed = false;
                    } else {
                        is_sealed = true;
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Purity => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    purity = Some(self.parse_identifier_or_string()?);
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Cohesion => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    self.match_token(&TokenKind::GreaterEqual);
                    self.match_token(&TokenKind::Equal);
                    if let TokenKind::FloatLit(f) = self.peek_kind() {
                        cohesion = Some(*f);
                        self.advance();
                    } else if let TokenKind::IntLit(i) = self.peek_kind() {
                        cohesion = Some(*i as f64);
                        self.advance();
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Struct => {
                    structs.push(self.parse_struct(false, pending_directives)?);
                }
                TokenKind::Fn => {
                    functions.push(self.parse_function(false, pending_directives)?);
                }
                TokenKind::Override => {
                    self.advance();
                    if self.check(&TokenKind::Fn) {
                        overrides.push(self.parse_function(false, pending_directives)?);
                    } else {
                        self.advance();
                    }
                }
                TokenKind::Pub => {
                    self.advance();
                    match self.peek_kind() {
                        TokenKind::Struct => {
                            structs.push(self.parse_struct(true, pending_directives)?);
                        }
                        TokenKind::Fn => {
                            functions.push(self.parse_function(true, pending_directives)?);
                        }
                        TokenKind::Override => {
                            self.advance();
                            overrides.push(self.parse_function(true, pending_directives)?);
                        }
                        _ => { self.advance(); }
                    }
                }
                TokenKind::SemiColon => { self.advance(); }
                _ => {
                    if let Ok(stmt) = self.parse_statement() {
                        statements.push(stmt);
                    } else {
                        self.advance();
                    }
                }
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(ModuleDef {
            name,
            parent,
            is_pub,
            is_partial: false,
            is_evolvable: is_evolvable_mod,
            responsibility,
            owns,
            exposes,
            depends,
            depends_only,
            forbid,
            is_sealed,
            purity,
            cohesion,
            facets: if has_facets { Some(facets) } else { None },
            contract: if has_contract { Some(contract) } else { None },
            overlay_target: None,
            skills,
            structs,
            functions,
            overrides,
            statements,
            span,
        })
    }

}
