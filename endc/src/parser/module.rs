use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub fn parse_module(&mut self, module_name: &str) -> Result<Module, String> {
        let mut imports = Vec::new();
        let mut enums = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut functions = Vec::new();
        let mut modules = Vec::new();
        let mut extensions = Vec::new();
        let mut features = Vec::new();
        let mut contracts = Vec::new();
        let mut architecture_templates = Vec::new();
        let mut architecture_rules = Vec::new();
        let mut feature_migrations = Vec::new();
        let mut statements = Vec::new();
        let start_span = self.current_span();

        while !self.check(&TokenKind::EOF) {
            let mut pending_directives = Vec::new();
            while let TokenKind::Directive(d) = self.peek_kind() {
                let dir_name = d.clone();
                let dir_span = self.current_span();
                self.advance();
                let mut args = Vec::new();
                if self.match_token(&TokenKind::LParen) {
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        match self.peek_kind() {
                            TokenKind::StringLit(s) => {
                                args.push(s.clone());
                                self.advance();
                            }
                            TokenKind::Ident(i) => {
                                let mut arg = i.clone();
                                self.advance();
                                if self.match_token(&TokenKind::Equal) || self.match_token(&TokenKind::Colon) {
                                    arg.push('=');
                                    match self.peek_kind() {
                                        TokenKind::StringLit(s) => { arg.push_str(s); self.advance(); }
                                        TokenKind::Ident(s) => { arg.push_str(s); self.advance(); }
                                        TokenKind::True => { arg.push_str("true"); self.advance(); }
                                        TokenKind::False => { arg.push_str("false"); self.advance(); }
                                        TokenKind::IntLit(n) => { arg.push_str(&n.to_string()); self.advance(); }
                                        _ => {}
                                    }
                                }
                                args.push(arg);
                            }
                            TokenKind::True => {
                                args.push("true".to_string());
                                self.advance();
                            }
                            TokenKind::False => {
                                args.push("false".to_string());
                                self.advance();
                            }
                            TokenKind::IntLit(n) => {
                                args.push(n.to_string());
                                self.advance();
                            }
                            _ => {
                                self.advance();
                            }
                        }
                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                }
                if dir_name == "@import_c" || dir_name == "@c" {
                    if let Some(first_arg) = args.first() {
                        imports.push(ImportStmt {
                            kind: ImportKind::C(first_arg.clone()),
                            path: first_arg.clone(),
                            alias: None,
                            span: dir_span,
                        });
                    }
                    continue;
                }

                pending_directives.push(Directive {
                    name: dir_name,
                    args,
                    span: dir_span,
                });
            }

            let peek_k = self.peek_kind().clone();
            let cur_span = self.current_span();
            if let Some(cap_stmt) = self.parse_capability_composition_statement(&peek_k, &cur_span)? {
                statements.push(cap_stmt);
                continue;
            }
            match peek_k {
                TokenKind::Abstract => {
                    self.advance();
                    if self.check(&TokenKind::Class) {
                        let mut c = self.parse_class(false, pending_directives)?;
                        c.is_abstract = true;
                        statements.push(Statement::ClassDecl(c));
                    } else {
                        let mut f = self.parse_function(false, pending_directives)?;
                        f.directives.push(Directive { name: "@abstract".to_string(), args: vec![], span: f.span.clone() });
                        functions.push(f);
                    }
                }
                TokenKind::Sealed => {
                    self.advance();
                    if self.check(&TokenKind::Class) {
                        let mut c = self.parse_class(false, pending_directives)?;
                        c.is_sealed = true;
                        statements.push(Statement::ClassDecl(c));
                    } else if self.check(&TokenKind::Mod) {
                        let mut m = self.parse_module_def(false, pending_directives)?;
                        m.is_sealed = true;
                        modules.push(m);
                    } else if self.check(&TokenKind::Struct) {
                        let mut s = self.parse_struct(false, pending_directives)?;
                        s.is_sealed = true;
                        structs.push(s);
                    } else if self.check(&TokenKind::Boundary) || (if let TokenKind::Ident(s) = self.peek_kind() { s.eq_ignore_ascii_case("boundary") } else { false }) {
                        self.advance();
                        let name = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::SemiColon);
                        statements.push(Statement::LayerSealedDecl { target_kind: "module".to_string(), target_name: name, span: self.current_span() });
                    } else {
                        let name = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::SemiColon);
                        statements.push(Statement::SealedDecl { boundary_name: name, span: self.current_span() });
                    }
                }
                TokenKind::Open => {
                    self.advance();
                    if self.check(&TokenKind::Class) {
                        let mut c = self.parse_class(false, pending_directives)?;
                        c.is_open = true;
                        statements.push(Statement::ClassDecl(c));
                    } else if self.check(&TokenKind::Mod) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "module" || s == "mod" } else { false }) {
                        self.advance();
                        let name = self.parse_identifier_or_keyword()?;
                        if self.match_token(&TokenKind::SemiColon) {
                            statements.push(Statement::OpenClosedTypeDecl { is_open: true, name, span: self.current_span() });
                        } else {
                            let mut m = self.parse_module_def(false, pending_directives)?;
                            m.name = name.clone();
                            statements.push(Statement::OpenClosedTypeDecl { is_open: true, name, span: self.current_span() });
                            modules.push(m);
                        }
                    } else if self.check(&TokenKind::Struct) {
                        let mut s = self.parse_struct(false, pending_directives)?;
                        s.is_open = true;
                        statements.push(Statement::OpenClosedTypeDecl { is_open: true, name: s.name.clone(), span: self.current_span() });
                        structs.push(s);
                    } else {
                        let mut name = self.parse_identifier_or_keyword()?;
                        if name == "type" {
                            name = self.parse_identifier_or_keyword()?;
                        }
                        self.match_token(&TokenKind::SemiColon);
                        statements.push(Statement::OpenClosedTypeDecl { is_open: true, name, span: self.current_span() });
                    }
                }
                TokenKind::Closed => {
                    self.advance();
                    if self.check(&TokenKind::Mod) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "module" || s == "mod" } else { false }) {
                        self.advance();
                        let name = self.parse_identifier_or_keyword()?;
                        if self.match_token(&TokenKind::SemiColon) {
                            statements.push(Statement::OpenClosedTypeDecl { is_open: false, name, span: self.current_span() });
                        } else {
                            let mut m = self.parse_module_def(false, pending_directives)?;
                            m.name = name.clone();
                            statements.push(Statement::OpenClosedTypeDecl { is_open: false, name, span: self.current_span() });
                            modules.push(m);
                        }
                    } else if self.check(&TokenKind::Struct) {
                        let mut s = self.parse_struct(false, pending_directives)?;
                        s.is_closed = true;
                        statements.push(Statement::OpenClosedTypeDecl { is_open: false, name: s.name.clone(), span: self.current_span() });
                        structs.push(s);
                    } else {
                        let mut name = self.parse_identifier_or_keyword()?;
                        if name == "type" {
                            name = self.parse_identifier_or_keyword()?;
                        }
                        self.match_token(&TokenKind::SemiColon);
                        statements.push(Statement::OpenClosedTypeDecl { is_open: false, name, span: self.current_span() });
                    }
                }
                TokenKind::Partial => {
                    self.advance();
                    if self.check(&TokenKind::Struct) {
                        let mut s = self.parse_struct(false, pending_directives)?;
                        s.is_partial = true;
                        structs.push(s);
                    } else if self.check(&TokenKind::Mod) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "module" || s == "mod" } else { false }) {
                        let mut m = self.parse_module_def(false, pending_directives)?;
                        m.is_partial = true;
                        modules.push(m);
                    }
                }
                TokenKind::Augment => {
                    extensions.push(self.parse_extension_block()?);
                }
                TokenKind::Use => {
                    let checkpoint = self.checkpoint();
                    self.advance();
                    if self.check(&TokenKind::Syntax) || self.check(&TokenKind::Feature) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "syntax" || s == "feature" } else { false }) {
                        self.restore_checkpoint(checkpoint);
                        let stmt = self.parse_statement()?;
                        statements.push(stmt);
                    } else {
                        self.restore_checkpoint(checkpoint);
                        imports.push(self.parse_import()?);
                    }
                }
                TokenKind::Import => {
                    imports.push(self.parse_import()?);
                }
                TokenKind::Enum => {
                    enums.push(self.parse_enum(false, pending_directives)?);
                }
                TokenKind::Struct => {
                    structs.push(self.parse_struct(false, pending_directives)?);
                }
                TokenKind::Trait => {
                    traits.push(self.parse_trait(false)?);
                }
                TokenKind::Impl => {
                    impls.push(self.parse_impl()?);
                }
                TokenKind::Fn => {
                    match self.parse_function(false, pending_directives) {
                        Ok(f) => functions.push(f),
                        Err(e) => {
                            if !self.diagnostics.has_errors() {
                                let span = self.current_span();
                                self.emit_e005(&span, "valid function definition", &format!("{:?}", self.peek_kind()), &e);
                            }
                            self.synchronize();
                        }
                    }
                }
                TokenKind::Extern => {
                    self.advance();
                    match self.parse_function(false, pending_directives) {
                        Ok(mut f) => {
                            f.directives.push(Directive { name: "@extern".to_string(), args: vec![], span: f.span.clone() });
                            functions.push(f);
                        }
                        Err(e) => {
                            if !self.diagnostics.has_errors() {
                                let span = self.current_span();
                                self.emit_e005(&span, "valid function definition", &format!("{:?}", self.peek_kind()), &e);
                            }
                            self.synchronize();
                        }
                    }
                }
                TokenKind::Val => {
                    self.advance();
                    let _name = self.parse_identifier_or_keyword()?;
                    if self.match_token(&TokenKind::Colon) {
                        let _ = self.parse_type()?;
                    }
                    self.expect(TokenKind::Equal)?;
                    let _ = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                }
                TokenKind::Mod => {
                    modules.push(self.parse_module_def(false, pending_directives)?);
                }
                TokenKind::Extend => {
                    let ext = self.parse_extension_block()?;
                    statements.push(Statement::ImplementContract {
                        contract: ext.target.clone(),
                        target: None,
                        methods: ext.functions.clone(),
                        span: ext.span.clone(),
                    });
                    extensions.push(ext);
                }
                TokenKind::Feature => {
                    let feat = self.parse_feature_def(false, pending_directives)?;
                    features.push(feat.clone());
                    statements.push(Statement::FeatureStatement(feat));
                }
                TokenKind::Contract => {
                    let ctr = self.parse_contract_def(false)?;
                    contracts.push(ctr.clone());
                    statements.push(Statement::ContractDefinition(ctr));
                }
                TokenKind::Migration => {
                    let mig = self.parse_feature_migration()?;
                    feature_migrations.push(mig.clone());
                    let from_v = mig.from_version.parse::<usize>().unwrap_or(0);
                    let to_v = mig.to_version.parse::<usize>().unwrap_or(0);
                    if from_v > 0 || to_v > 0 {
                        statements.push(Statement::ModuleMigrationDecl {
                            module_name: mig.feature_name.clone(),
                            from_version: from_v,
                            to_version: to_v,
                            renames: mig.renames.clone(),
                            span: mig.span.clone(),
                        });
                    }
                    statements.push(Statement::FeatureMigrationStatement(mig));
                }
                TokenKind::Replace => {
                    self.advance();
                    let target_kind = self.parse_identifier_or_keyword()?;
                    if target_kind == "feature" {
                        let target = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::With);
                        let with_provider = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::SemiColon);
                        statements.push(Statement::ReplaceFeature { target, with_provider, span: self.current_span() });
                    } else if target_kind == "module" || target_kind == "mod" {
                        let target = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::With);
                        let replacement = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::SemiColon);
                        statements.push(Statement::ReplaceModuleDecl { target, replacement, span: self.current_span() });
                    } else {
                        let mut target = target_kind;
                        while self.match_token(&TokenKind::Dot) {
                            target.push('.');
                            target.push_str(&self.parse_identifier_or_keyword()?);
                        }
                        self.match_token(&TokenKind::With);
                        let replacement = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::SemiColon);
                        statements.push(Statement::ReplaceModuleDecl { target, replacement, span: self.current_span() });
                    }
                }
                TokenKind::Decorate => {
                    self.advance();
                    let _ = self.match_token(&TokenKind::Feature);
                    let target = self.parse_identifier_or_keyword()?;
                    let mut decorators = Vec::new();
                    if self.match_token(&TokenKind::With) {
                        if self.check(&TokenKind::LBracket) {
                            decorators = self.parse_string_list()?;
                        } else {
                            decorators.push(self.parse_identifier_or_keyword()?);
                        }
                        self.match_token(&TokenKind::SemiColon);
                    } else if self.check(&TokenKind::LBrace) {
                        self.advance();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            decorators.push(self.parse_identifier_or_keyword()?);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                    }
                    statements.push(Statement::DecorateFeature { target, decorators, span: self.current_span() });
                }
                TokenKind::Compose => {
                    self.advance();
                    if self.check(&TokenKind::Feature) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "feature" } else { false }) {
                        self.advance();
                        let target = self.parse_identifier_or_keyword()?;
                        let mut components = Vec::new();
                        if self.match_token(&TokenKind::With) {
                            if self.check(&TokenKind::LBracket) {
                                components = self.parse_string_list()?;
                            } else {
                                components.push(self.parse_identifier_or_keyword()?);
                            }
                            self.match_token(&TokenKind::SemiColon);
                        } else if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                components.push(self.parse_identifier_or_keyword()?);
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        }
                        statements.push(Statement::ComposeFeature { target, components, span: self.current_span() });
                    } else {
                        let mut modules = Vec::new();
                        if self.check(&TokenKind::LBracket) {
                            modules = self.parse_string_list()?;
                        } else if self.check(&TokenKind::LBrace) {
                            self.advance();
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                modules.push(self.parse_identifier_or_keyword()?);
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                        }
                        statements.push(Statement::ModuleComposeDecl { modules, span: self.current_span() });
                    }
                }
                TokenKind::Pub => {
                    self.advance();
                    match self.peek_kind() {
                        TokenKind::Enum => {
                            enums.push(self.parse_enum(true, pending_directives)?);
                        }
                        TokenKind::Struct => {
                            structs.push(self.parse_struct(true, pending_directives)?);
                        }
                        TokenKind::Trait => {
                            traits.push(self.parse_trait(true)?);
                        }
                        TokenKind::Fn => {
                            match self.parse_function(true, pending_directives) {
                                Ok(f) => functions.push(f),
                                Err(e) => {
                                    if !self.diagnostics.has_errors() {
                                        let span = self.current_span();
                                        self.emit_e005(&span, "valid function definition", &format!("{:?}", self.peek_kind()), &e);
                                    }
                                    self.synchronize();
                                }
                            }
                        }
                        TokenKind::Feature => {
                            let feat = self.parse_feature_def(true, pending_directives)?;
                            features.push(feat.clone());
                            statements.push(Statement::FeatureStatement(feat));
                        }
                        TokenKind::Contract => {
                            let ctr = self.parse_contract_def(true)?;
                            contracts.push(ctr.clone());
                            statements.push(Statement::ContractDefinition(ctr));
                        }
                        TokenKind::Extern => {
                            self.advance();
                            match self.parse_function(true, pending_directives) {
                                Ok(mut f) => {
                                    f.directives.push(Directive { name: "@extern".to_string(), args: vec![], span: f.span.clone() });
                                    functions.push(f);
                                }
                                Err(e) => {
                                    if !self.diagnostics.has_errors() {
                                        let span = self.current_span();
                                        self.emit_e005(&span, "valid function definition", &format!("{:?}", self.peek_kind()), &e);
                                    }
                                    self.synchronize();
                                }
                            }
                        }
                        TokenKind::Val => {
                            self.advance();
                            let _name = self.parse_identifier_or_keyword()?;
                            if self.match_token(&TokenKind::Colon) {
                                let _ = self.parse_type()?;
                            }
                            self.expect(TokenKind::Equal)?;
                            let _ = self.parse_expression()?;
                            self.match_token(&TokenKind::SemiColon);
                        }
                        TokenKind::Mod => {
                            modules.push(self.parse_module_def(true, pending_directives)?);
                        }
                        TokenKind::Operation => {
                            statements.push(Statement::OperationDecl(self.parse_operation(true)?));
                        }
                        TokenKind::Event => {
                            statements.push(Statement::EventDecl(self.parse_event(true)?));
                        }
                        TokenKind::Class | TokenKind::Abstract | TokenKind::Sealed | TokenKind::Open => {
                            let class_def = self.parse_class(true, pending_directives)?;
                            statements.push(Statement::ClassDecl(class_def));
                        }
                        TokenKind::Hub => {
                            statements.push(Statement::EventHubDecl(self.parse_event_hub(true)?));
                        }
                        other => {
                            let span = self.current_span();
                            let actual = format!("{:?}", other);
                            let expected = "enum, struct, trait, val, class, event, operation or fn after 'pub'";
                            let raw_msg = format!(
                                "Expected enum, struct, trait, val or fn after 'pub', found {:?} at line {}",
                                other,
                                span.line
                            );
                            let formatted = self.emit_e005(&span, expected, &actual, &raw_msg);
                            return Err(formatted);
                        }
                    }
                }
                TokenKind::SemiColon | TokenKind::RBrace => {
                    self.advance();
                }
                TokenKind::EOF => break,
                _ => {
                    match self.parse_statement() {
                        Ok(stmt) => {
                            match &stmt {
                                Statement::FeatureStatement(f) => {
                                    features.push(f.clone());
                                }
                                Statement::ContractDefinition(c) => {
                                    contracts.push(c.clone());
                                }
                                Statement::ArchitectureTemplate(a) => {
                                    architecture_templates.push(a.clone());
                                }
                                Statement::ArchitectureRuleStatement(r) => {
                                    architecture_rules.push(r.clone());
                                }
                                Statement::FeatureMigrationStatement(m) => {
                                    feature_migrations.push(m.clone());
                                }
                                _ => {}
                            }
                            statements.push(stmt);
                        }
                        Err(e) => {
                            if !self.diagnostics.has_errors() {
                                let span = self.current_span();
                                self.emit_e005(&span, "valid statement", &format!("{:?}", self.peek_kind()), &e);
                            }
                            self.synchronize();
                        }
                    }
                }
            }
        }

        if self.diagnostics.has_errors() {
            let error_msgs: Vec<String> = self.diagnostics.diagnostics()
                .iter()
                .filter(|d| matches!(d.severity, crate::diagnostics::Severity::Error))
                .map(|d| d.message.clone())
                .collect();
            return Err(error_msgs.join("\n"));
        }

        Ok(Module {
            name: module_name.to_string(),
            imports,
            enums,
            structs,
            traits,
            impls,
            functions,
            modules,
            extensions,
            features,
            contracts,
            architecture_templates,
            architecture_rules,
            feature_migrations,
            statements,
            span: start_span,
        })
    }
}
