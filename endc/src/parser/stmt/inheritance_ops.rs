use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_inheritance_ops_statement(
        &mut self,
        peek_k: &TokenKind,
        span: &Span,
    ) -> Result<Option<Statement>, String> {
        let span = span.clone();
        match peek_k {
            TokenKind::Class | TokenKind::Abstract | TokenKind::Sealed | TokenKind::Open => {
                let is_class_decl = match peek_k {
                    TokenKind::Class => true,
                    TokenKind::Abstract | TokenKind::Sealed | TokenKind::Open => {
                        self.peek_next_kind() == Some(&TokenKind::Class)
                    }
                    _ => false,
                };
                if is_class_decl {
                    let class_def = self.parse_class(false, vec![])?;
                    return Ok(Some(Statement::ClassDecl(class_def)));
                }
                Ok(None)
            }
            TokenKind::Trait => {
                let trait_def = self.parse_trait(false)?;
                Ok(Some(Statement::TraitDecl(trait_def)))
            }
            TokenKind::Conflict => {
                self.advance();
                let left = self.parse_dotted_path_str()?;
                let right = self.parse_dotted_path_str()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ConflictStmt(ConflictDef { left, right, span })))
            }
            TokenKind::Resolve => {
                self.advance();
                let preferred = self.parse_dotted_path_str()?;
                let mut over = None;
                if self.match_token(&TokenKind::Over) || self.match_token(&TokenKind::By) {
                    over = Some(self.parse_dotted_path_str()?);
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::ResolveConflictStmt(ResolutionDef {
                    preferred,
                    over,
                    is_merge: false,
                    span,
                })))
            }
            TokenKind::Inspect => {
                self.advance();
                let target_kind = self.parse_identifier_or_keyword()?;
                if target_kind == "inheritance" {
                    let target = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Some(Statement::InspectInheritanceStmt(InspectInheritanceDef { target, span })));
                }
                let target = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::InspectInheritanceStmt(InspectInheritanceDef { target, span })))
            }
            TokenKind::Inherit | TokenKind::Inherits => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let inherit_def = self.parse_inherit_clause(target, span)?;
                Ok(Some(Statement::InheritStmt(inherit_def)))
            }
            TokenKind::Super => {
                self.advance();
                let mut target_parent = None;
                if self.match_token(&TokenKind::LParen) {
                    target_parent = Some(self.parse_identifier_or_keyword()?);
                    self.expect(TokenKind::RParen)?;
                }
                self.expect(TokenKind::Dot)?;
                let method = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LParen)?;
                let mut args = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    args.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                let is_superchain = self.match_token(&TokenKind::SuperChain);
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::SuperCallStmt(SuperCall {
                    target_parent,
                    method,
                    args,
                    is_superchain,
                    span,
                })))
            }
            TokenKind::Delegates => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let parent = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Some(Statement::InheritStmt(InheritDef {
                    target,
                    parent,
                    kind: InheritKind::Standard,
                    alias: None,
                    only: vec![],
                    except: vec![],
                    transforms: vec![],
                    mappings: vec![],
                    condition: None,
                    is_contractual: false,
                    is_replaceable: false,
                    is_delegation: true,
                    capability_grants: vec![],
                    capability_denials: vec![],
                    permission_removals: vec![],
                    body: None,
                    span,
                })))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn parse_dotted_path_str(&mut self) -> Result<String, String> {
        let mut path = self.parse_identifier_or_keyword()?;
        while self.match_token(&TokenKind::Dot) {
            path.push('.');
            path.push_str(&self.parse_identifier_or_keyword()?);
        }
        Ok(path)
    }

    pub(crate) fn parse_inherit_clause(&mut self, target: String, span: Span) -> Result<InheritDef, String> {
        let mut kind = InheritKind::Standard;
        let mut is_contractual = false;
        let mut is_replaceable = false;
        let mut is_delegation = false;

        if self.match_token(&TokenKind::Replaceable) {
            is_replaceable = true;
        }
        if self.match_token(&TokenKind::Delegation) || self.match_token(&TokenKind::Delegates) {
            is_delegation = true;
        }

        if self.match_token(&TokenKind::Surface) {
            let surface_name = self.parse_identifier_or_keyword()?;
            kind = InheritKind::Surface(surface_name);
        } else if self.match_token(&TokenKind::Shape) {
            let shape_name = self.parse_identifier_or_keyword()?;
            kind = InheritKind::Shape(shape_name);
        } else if self.match_token(&TokenKind::Behavioral) || self.match_token(&TokenKind::Behavior) {
            let beh_name = self.parse_identifier_or_keyword()?;
            kind = InheritKind::Behavior(beh_name);
        } else if self.match_token(&TokenKind::Contract) {
            let ctr_name = self.parse_identifier_or_keyword()?;
            kind = InheritKind::Contract(ctr_name);
        } else if self.match_token(&TokenKind::Capabilities) || self.match_token(&TokenKind::Capability) {
            kind = InheritKind::Capabilities;
        } else if self.match_token(&TokenKind::Permissions) || self.match_token(&TokenKind::Permission) {
            kind = InheritKind::Permissions;
        } else if self.match_token(&TokenKind::Events) || self.match_token(&TokenKind::Event) {
            kind = InheritKind::Events;
        } else if self.match_token(&TokenKind::Feature) {
            kind = InheritKind::Feature;
        } else if self.match_token(&TokenKind::Architecture) {
            kind = InheritKind::Architecture;
        } else if self.match_token(&TokenKind::Policy) {
            kind = InheritKind::Policy;
        } else if self.match_token(&TokenKind::Lifecycle) {
            kind = InheritKind::Lifecycle;
        }

        let parent = if !self.check(&TokenKind::SemiColon)
            && !self.check(&TokenKind::As)
            && !self.check(&TokenKind::Only)
            && !self.check(&TokenKind::Except)
            && !self.check(&TokenKind::Without)
            && !self.check(&TokenKind::Over)
            && !self.check(&TokenKind::EOF)
        {
            self.parse_identifier_or_keyword()?
        } else {
            target.clone()
        };
        if self.match_token(&TokenKind::Dot) {
            let member = self.parse_identifier_or_keyword()?;
            if member == "surface" {
                let surf = self.parse_identifier_or_keyword()?;
                kind = InheritKind::Surface(surf);
            } else if member == "shape" {
                let shp = self.parse_identifier_or_keyword()?;
                kind = InheritKind::Shape(shp);
            } else {
                kind = InheritKind::Surface(member);
            }
        }

        let mut alias = None;
        if self.match_token(&TokenKind::As) {
            alias = Some(self.parse_identifier_or_keyword()?);
        }

        let mut only = Vec::new();
        if self.match_token(&TokenKind::Only) {
            self.expect(TokenKind::LBrace)?;
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                only.push(self.parse_identifier_or_keyword()?);
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
        }

        let mut except = Vec::new();
        if self.match_token(&TokenKind::Except) {
            self.expect(TokenKind::LBrace)?;
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let mut name = self.parse_identifier_or_keyword()?;
                if self.match_token(&TokenKind::LParen) {
                    self.expect(TokenKind::RParen)?;
                    name.push_str("()");
                }
                except.push(name);
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
        }

        let mut transforms = Vec::new();
        if self.match_token(&TokenKind::Transform) {
            self.expect(TokenKind::LBrace)?;
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let action = self.parse_identifier_or_keyword()?;
                let from = self.parse_identifier_or_keyword()?;
                let to = if self.match_token(&TokenKind::Arrow) {
                    self.parse_identifier_or_keyword()?
                } else {
                    String::new()
                };
                transforms.push((format!("{} {}", action, from), to));
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
        }

        let mut mappings = Vec::new();
        if self.match_token(&TokenKind::Map) {
            self.expect(TokenKind::LBrace)?;
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let from = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::Arrow)?;
                let to = self.parse_identifier_or_keyword()?;
                mappings.push((from, to));
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
        }

        let mut condition = None;
        if self.match_token(&TokenKind::When) || self.match_token(&TokenKind::If) {
            condition = Some(self.parse_expression()?);
        }

        if self.match_token(&TokenKind::Contractually) {
            is_contractual = true;
        }

        let mut permission_removals = Vec::new();
        if self.match_token(&TokenKind::Without) {
            let _ = self.match_token(&TokenKind::Permissions);
            let perm = self.parse_dotted_path_str()?;
            permission_removals.push(perm);
        }

        let mut capability_grants = Vec::new();
        let mut capability_denials = Vec::new();
        let mut body = None;

        if self.check(&TokenKind::LBrace) {
            let mut stmts = Vec::new();
            self.advance(); // consume {
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                if self.match_token(&TokenKind::Inherit) {
                    if self.match_token(&TokenKind::Capability) || self.match_token(&TokenKind::Capabilities) {
                        let cap = self.parse_dotted_path_str()?;
                        capability_grants.push(cap);
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                        continue;
                    }
                }
                if self.match_token(&TokenKind::Deny) {
                    if self.match_token(&TokenKind::Capability) || self.match_token(&TokenKind::Capabilities) {
                        let cap = self.parse_dotted_path_str()?;
                        capability_denials.push(cap);
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                        continue;
                    }
                }
                stmts.push(self.parse_statement()?);
            }
            self.expect(TokenKind::RBrace)?;
            body = Some(Block { statements: stmts, span: span.clone() });
        } else {
            self.match_token(&TokenKind::SemiColon);
        }

        Ok(InheritDef {
            target,
            parent,
            kind,
            alias,
            only,
            except,
            transforms,
            mappings,
            condition,
            is_contractual,
            is_replaceable,
            is_delegation,
            capability_grants,
            capability_denials,
            permission_removals,
            body,
            span,
        })
    }
}
