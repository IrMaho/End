use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    fn parse_bool_val(&mut self) -> Result<bool, String> {
        if self.match_token(&TokenKind::True) {
            Ok(true)
        } else if self.match_token(&TokenKind::False) {
            Ok(false)
        } else {
            let s = self.parse_identifier_or_keyword()?;
            match s.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                other => Err(format!("Expected boolean, found {}", other)),
            }
        }
    }

    pub(crate) fn parse_refactor_session_decl(&mut self) -> Result<RefactorSessionDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Ident("session".to_string()))?;
        let agent_name = self.parse_identifier_or_string()?;
        let mut target = String::new();
        let mut scope = Vec::new();
        let mut forbid = Vec::new();
        let mut goals = Vec::new();

        self.expect(TokenKind::LBrace)?;
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let key = self.parse_identifier_or_keyword()?;
            self.match_token(&TokenKind::Colon);
            match key.as_str() {
                "target" => {
                    target = self.parse_identifier_or_string()?;
                }
                "scope" => {
                    scope = self.parse_string_list()?;
                }
                "forbid" => {
                    forbid = self.parse_string_list()?;
                }
                "goals" | "goal" => {
                    goals = self.parse_string_list()?;
                }
                _ => {
                    let _ = self.parse_identifier_or_string();
                }
            }
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(RefactorSessionDef {
            agent_name,
            target,
            scope,
            forbid,
            goals,
            span,
        })
    }

    pub(crate) fn parse_decomposition_plan_decl(&mut self) -> Result<DecompositionPlanDef, String> {
        let span = self.current_span();
        let mut source = String::new();
        let mut target_architecture = String::new();
        let mut facade_name = None;
        let mut submodules = Vec::new();

        if self.match_token(&TokenKind::For) || self.match_token(&TokenKind::From) {
            source = self.parse_identifier_or_string()?;
        } else if !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::EOF) {
            source = self.parse_identifier_or_string()?;
        }

        self.expect(TokenKind::LBrace)?;
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let key = self.parse_identifier_or_keyword()?;
            self.match_token(&TokenKind::Colon);
            match key.as_str() {
                "source" => {
                    source = self.parse_identifier_or_string()?;
                }
                "target_architecture" | "architecture" => {
                    target_architecture = self.parse_identifier_or_string()?;
                }
                "facade" | "facade_name" => {
                    facade_name = Some(self.parse_identifier_or_string()?);
                }
                "submodules" | "modules" => {
                    self.expect(TokenKind::LBracket)?;
                    while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::EOF) {
                        self.expect(TokenKind::LBrace)?;
                        let mut sub_name = String::new();
                        let mut sub_role = "submodule".to_string();
                        let mut sub_symbols = Vec::new();
                        let mut sub_max_loc = 500;

                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let f_key = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            match f_key.as_str() {
                                "name" => sub_name = self.parse_identifier_or_string()?,
                                "role" => sub_role = self.parse_identifier_or_string()?,
                                "symbols" => sub_symbols = self.parse_string_list()?,
                                "max_loc" | "max_lines" => {
                                    if let TokenKind::IntLit(n) = self.peek_kind() {
                                        sub_max_loc = *n as usize;
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
                        }
                        self.expect(TokenKind::RBrace)?;
                        submodules.push(SubmodulePlan {
                            name: sub_name,
                            role: sub_role,
                            symbols: sub_symbols,
                            max_loc: sub_max_loc,
                        });
                        self.match_token(&TokenKind::Comma);
                    }
                    self.expect(TokenKind::RBracket)?;
                }
                _ => {
                    if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                        let _ = self.parse_string_list();
                    } else if let TokenKind::IntLit(_) = self.peek_kind() {
                        self.advance();
                    } else {
                        let _ = self.parse_identifier_or_string();
                    }
                }
            }
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(DecompositionPlanDef {
            source,
            target_architecture,
            submodules,
            facade_name,
            span,
        })
    }

    pub(crate) fn parse_conservation_audit_decl(&mut self) -> Result<ConservationAuditDef, String> {
        let span = self.current_span();
        let mut original_source = String::new();
        let mut original_loc = 0;
        let mut original_symbols = Vec::new();
        let mut new_loc = 0;
        let mut accounted_symbols = Vec::new();
        let mut unaccounted_count = 0;
        let mut allow_semantic_deletion = false;

        self.expect(TokenKind::LBrace)?;
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let key = self.parse_identifier_or_keyword()?;
            self.match_token(&TokenKind::Colon);
            match key.as_str() {
                "original_source" | "source" => original_source = self.parse_identifier_or_string()?,
                "original_loc" | "original_lines" => {
                    if let TokenKind::IntLit(n) = self.peek_kind() { original_loc = *n as usize; self.advance(); }
                }
                "original_symbols" => original_symbols = self.parse_string_list()?,
                "new_loc" | "new_lines" => {
                    if let TokenKind::IntLit(n) = self.peek_kind() { new_loc = *n as usize; self.advance(); }
                }
                "accounted_symbols" => accounted_symbols = self.parse_string_list()?,
                "unaccounted" | "unaccounted_count" => {
                    if let TokenKind::IntLit(n) = self.peek_kind() { unaccounted_count = *n as usize; self.advance(); }
                }
                "allow_deletion" | "allow_semantic_deletion" => {
                    allow_semantic_deletion = self.parse_bool_val()?;
                }
                _ => { let _ = self.parse_identifier_or_string(); }
            }
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(ConservationAuditDef {
            original_source,
            original_loc,
            original_symbols,
            new_loc,
            accounted_symbols,
            unaccounted_count,
            allow_semantic_deletion,
            span,
        })
    }

    pub(crate) fn parse_solid_audit_decl(&mut self) -> Result<SolidAuditDef, String> {
        let span = self.current_span();
        let module_name = self.parse_identifier_or_string()?;
        let mut verify_srp = true;
        let mut verify_ocp = true;
        let mut verify_lsp = true;
        let mut verify_isp = true;
        let mut verify_dip = true;
        let mut max_responsibilities = 1;

        if self.check(&TokenKind::LBrace) {
            self.advance();
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let key = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Colon);
                match key.as_str() {
                    "srp" => verify_srp = self.parse_bool_val()?,
                    "ocp" => verify_ocp = self.parse_bool_val()?,
                    "lsp" => verify_lsp = self.parse_bool_val()?,
                    "isp" => verify_isp = self.parse_bool_val()?,
                    "dip" => verify_dip = self.parse_bool_val()?,
                    "max_responsibilities" => if let TokenKind::IntLit(n) = self.peek_kind() { max_responsibilities = *n as usize; self.advance(); },
                    _ => { let _ = self.parse_identifier_or_string(); }
                }
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            self.match_token(&TokenKind::SemiColon);
        }

        Ok(SolidAuditDef {
            module_name,
            verify_srp,
            verify_ocp,
            verify_lsp,
            verify_isp,
            verify_dip,
            max_responsibilities,
            span,
        })
    }

    pub(crate) fn parse_refactoring_tx_decl(&mut self) -> Result<RefactoringTxDef, String> {
        let span = self.current_span();
        let tx_name = self.parse_identifier_or_string()?;
        let mut checkpoint = "baseline".to_string();
        let mut steps = Vec::new();
        let mut auto_rollback = true;
        let mut run_test_gate = true;
        let mut run_build_gate = true;
        let mut max_lines_limit = 500;

        self.expect(TokenKind::LBrace)?;
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let key = self.parse_identifier_or_keyword()?;
            self.match_token(&TokenKind::Colon);
            match key.as_str() {
                "checkpoint" => checkpoint = self.parse_identifier_or_string()?,
                "steps" => steps = self.parse_string_list()?,
                "auto_rollback" | "rollback" => auto_rollback = self.parse_bool_val()?,
                "test_gate" | "run_test_gate" => run_test_gate = self.parse_bool_val()?,
                "build_gate" | "run_build_gate" => run_build_gate = self.parse_bool_val()?,
                "max_lines" | "max_loc" => if let TokenKind::IntLit(n) = self.peek_kind() { max_lines_limit = *n as usize; self.advance(); },
                _ => { let _ = self.parse_identifier_or_string(); }
            }
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(RefactoringTxDef {
            tx_name,
            checkpoint,
            steps,
            auto_rollback,
            run_test_gate,
            run_build_gate,
            max_lines_limit,
            span,
        })
    }

    pub(crate) fn parse_symbol_inventory_decl(&mut self) -> Result<SymbolInventoryDef, String> {
        let span = self.current_span();
        let module_name = self.parse_identifier_or_string()?;
        let mut classes = Vec::new();
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut public_exports = Vec::new();
        let mut internal_symbols = Vec::new();

        self.expect(TokenKind::LBrace)?;
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let key = self.parse_identifier_or_keyword()?;
            self.match_token(&TokenKind::Colon);
            match key.as_str() {
                "classes" | "class" => classes = self.parse_string_list()?,
                "functions" | "fn" => functions = self.parse_string_list()?,
                "types" | "type" => types = self.parse_string_list()?,
                "public_exports" | "exports" => public_exports = self.parse_string_list()?,
                "internal_symbols" | "internal" => internal_symbols = self.parse_string_list()?,
                _ => { let _ = self.parse_identifier_or_string(); }
            }
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(SymbolInventoryDef {
            module_name,
            classes,
            functions,
            types,
            public_exports,
            internal_symbols,
            span,
        })
    }

    pub(crate) fn parse_traceable_map_decl(&mut self) -> Result<TraceableMapDef, String> {
        let span = self.current_span();
        let mut source_module = String::new();
        if self.peek_kind() == &TokenKind::Ident("destination".to_string()) || self.peek_kind() == &TokenKind::Ident("dest".to_string()) {
            self.advance();
        }
        if self.match_token(&TokenKind::For) || self.match_token(&TokenKind::From) {
            source_module = self.parse_identifier_or_string()?;
        }
        let mut mappings = Vec::new();

        self.expect(TokenKind::LBrace)?;
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let sym = self.parse_identifier_or_keyword()?;
            self.expect(TokenKind::Arrow)?;
            let dest = self.parse_identifier_or_keyword()?;
            mappings.push((sym, dest));
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);
        }
        self.expect(TokenKind::RBrace)?;

        Ok(TraceableMapDef {
            source_module,
            mappings,
            span,
        })
    }
}
