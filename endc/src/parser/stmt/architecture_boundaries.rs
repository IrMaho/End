use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_architecture_boundaries_statement(&mut self, peek_k: &TokenKind, span: &Span) -> Result<Option<Statement>, String> {
        match peek_k {
            TokenKind::Preserve
            | TokenKind::Compat
            | TokenKind::Stable
            | TokenKind::Sealed
            | TokenKind::Friend
            | TokenKind::PrivateTo
            | TokenKind::Surface
            | TokenKind::Leak
            | TokenKind::Purity
            | TokenKind::View
            | TokenKind::Lens
            | TokenKind::AgentScope
            | TokenKind::BudgetContext
            | TokenKind::Move
            | TokenKind::Migrate
            | TokenKind::Redirect
            | TokenKind::Deprecate => {}
            _ => return Ok(None),
        }

        let span = span.clone();
        let stmt = self.parse_architecture_boundaries_statement_inner(peek_k, span)?;
        Ok(Some(stmt))
    }

    fn parse_architecture_boundaries_statement_inner(&mut self, peek_k: &TokenKind, span: Span) -> Result<Statement, String> {
        match peek_k {
            TokenKind::Preserve => {
                self.advance();
                if self.peek_kind() == &TokenKind::Ident("refactor".to_string()) {
                    self.advance();
                }
                let preserves = self.parse_string_list()?;
                let body = self.parse_block()?;
                Ok(Statement::PreserveRefactorDecl { preserves, body, span })
            }
            TokenKind::Compat => {
                self.advance();
                let version = self.parse_identifier_or_keyword()?;
                let mut module_name = String::new();
                if self.match_token(&TokenKind::For) {
                    module_name = self.parse_identifier_or_keyword()?;
                }
                let body = self.parse_block()?;
                Ok(Statement::CompatDecl { module_name, version, body, span })
            }
            TokenKind::Stable => {
                self.advance();
                let mut api_name = self.parse_identifier_or_keyword()?;
                if (api_name == "API" || api_name == "api") && !self.check(&TokenKind::SemiColon) {
                    api_name = self.parse_identifier_or_keyword()?;
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::StableDecl { api_name, span })
            }
            TokenKind::Sealed => {
                self.advance();
                let target_kind = if self.match_token(&TokenKind::Mod) {
                    "module".to_string()
                } else if self.match_token(&TokenKind::Struct) {
                    "struct".to_string()
                } else {
                    let k = self.parse_identifier_or_keyword().unwrap_or_else(|_| "boundary".to_string());
                    if k == "Boundary" || k == "boundary" { "boundary".to_string() } else { k }
                };
                let target_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::LayerSealedDecl { target_kind, target_name, span })
            }
            TokenKind::Friend => {
                self.advance();
                let target_kind = if self.match_token(&TokenKind::Mod) {
                    "module".to_string()
                } else {
                    "type".to_string()
                };
                let name1 = self.parse_identifier_or_keyword()?;
                let (target_name, friend_name) = if self.match_token(&TokenKind::To) {
                    let name2 = self.parse_identifier_or_keyword()?;
                    (name1, name2)
                } else {
                    (String::new(), name1)
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::LayerFriendDecl { target_kind, target_name, friend_name, span })
            }
            TokenKind::PrivateTo => {
                self.advance();
                let symbol = self.parse_identifier_or_keyword()?;
                let module_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::PrivateToDecl { symbol, module_name, span })
            }
            TokenKind::Surface => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut exposes = Vec::new();
                let mut hides = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "expose" || key == "exposes" {
                        let mut l = self.parse_string_list()?;
                        exposes.append(&mut l);
                    } else if key == "hide" || key == "hides" {
                        let mut l = self.parse_string_list()?;
                        hides.append(&mut l);
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::SurfaceDecl { name, exposes, hides, span })
            }
            TokenKind::Leak => {
                self.advance();
                let mut module_name = String::new();
                if self.peek_kind() == &TokenKind::Ident("check".to_string()) || self.peek_kind() == &TokenKind::Ident("payments".to_string()) {
                    let _ = self.parse_identifier_or_keyword();
                    module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                }
                if self.match_token(&TokenKind::Forbid) {
                    // consumed forbid
                }
                let symbol = self.parse_identifier_or_keyword()?;
                if self.peek_kind() == &TokenKind::Ident("leaking".to_string()) {
                    self.advance();
                }
                self.match_token(&TokenKind::Through);
                let through = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::LeakCheckDecl { module_name, symbol, through, span })
            }
            TokenKind::Purity => {
                self.advance();
                let mut module_name = String::new();
                while !self.check(&TokenKind::Colon) && !self.check(&TokenKind::Equal) && !self.check(&TokenKind::EOF) {
                    let part = self.parse_identifier_or_keyword()?;
                    if part != "Module" && part != "mod" {
                        module_name = part;
                    }
                }
                self.match_token(&TokenKind::Colon);
                self.match_token(&TokenKind::Equal);
                let level = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::PurityDecl { module_name, level, span })
            }
            TokenKind::View => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut includes = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "include" || key == "includes" {
                        includes = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ViewDecl { name, includes, span })
            }
            TokenKind::Lens => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut focus = String::new();
                let mut hide = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "focus" {
                        focus = self.parse_identifier_or_string()?;
                    } else if key == "hide" {
                        hide = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::LensDecl { name, focus, hide, span })
            }
            TokenKind::AgentScope => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut modules = Vec::new();
                let mut forbid = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "modules" || key == "module" {
                        modules = self.parse_string_list()?;
                    } else if key == "forbid" {
                        forbid = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentScopeDecl { name, modules, forbid, span })
            }
            TokenKind::BudgetContext => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut token_budget = 8192;
                let mut priority = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "token_budget" || key == "budget" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            token_budget = *i as usize;
                            self.advance();
                        }
                    } else if key == "priority" {
                        priority = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::BudgetContextDecl { name, token_budget, priority, span })
            }
            TokenKind::Move => {
                self.advance();
                let symbol = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::From);
                let from_mod = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::To);
                let to_mod = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::MoveDecl { symbol, from_mod, to_mod, span })
            }
            TokenKind::Migrate => {
                self.advance();
                let entity = self.parse_identifier_or_keyword()?;
                let mut from_mod = String::new();
                let mut to_mod = String::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "from" {
                            from_mod = self.parse_identifier_or_string()?;
                        } else if key == "to" {
                            to_mod = self.parse_identifier_or_string()?;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    self.match_token(&TokenKind::From);
                    from_mod = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::To);
                    to_mod = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::MigrateDecl { entity, from_mod, to_mod, span })
            }
            TokenKind::Redirect => {
                self.advance();
                let from_api = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::Arrow);
                let to_api = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::RedirectDecl { from_api, to_api, span })
            }
            TokenKind::Deprecate => {
                self.advance();
                let target_api = self.parse_identifier_or_string()?;
                let mut after_cond = String::new();
                let mut remove_cond = String::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "after" {
                            after_cond = self.parse_identifier_or_string()?;
                        } else if key == "remove" {
                            remove_cond = self.parse_identifier_or_string()?;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    if self.match_token(&TokenKind::After) {
                        after_cond = self.parse_identifier_or_string()?;
                    }
                    if self.match_token(&TokenKind::Remove) {
                        remove_cond = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::DeprecateDecl { target_api, after_cond, remove_cond, span })
            }
            _ => unreachable!(),
        }
    }
}
