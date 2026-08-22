use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use std::collections::HashSet;

#[allow(dead_code)]
pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    pub filename: String,
    pub enum_names: HashSet<String>,
}

impl Parser {
    pub fn new(filename: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            filename: filename.into(),
            enum_names: HashSet::new(),
        }
    }

    fn peek(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else {
            self.tokens[self.tokens.len() - 1].clone()
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let current = self.peek();
        if std::mem::discriminant(&current.kind) == std::mem::discriminant(&kind) {
            Ok(self.advance())
        } else {
            Err(format!(
                "Expected token {:?}, found {:?} at line {}, col {}",
                kind, current.kind, current.span.line, current.span.col
            ))
        }
    }

    pub fn current_span(&self) -> Span {
        self.peek().span.clone()
    }

    pub fn parse_identifier_or_keyword(&mut self) -> Result<String, String> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(n) => Ok(n),
            TokenKind::Struct => Ok("st".to_string()),
            TokenKind::Val => Ok("val".to_string()),
            TokenKind::Mut => Ok("mut".to_string()),
            TokenKind::Target => Ok("target".to_string()),
            TokenKind::Match => Ok("match".to_string()),
            TokenKind::Fn => Ok("fn".to_string()),
            TokenKind::In => Ok("in".to_string()),
            TokenKind::Asm => Ok("asm".to_string()),
            TokenKind::Region => Ok("region".to_string()),
            TokenKind::Spawn => Ok("spawn".to_string()),
            TokenKind::Defer => Ok("defer".to_string()),
            TokenKind::Import => Ok("import".to_string()),
            TokenKind::Pub => Ok("pub".to_string()),
            TokenKind::As => Ok("as".to_string()),
            TokenKind::Alloc => Ok("alloc".to_string()),
            TokenKind::Catch => Ok("catch".to_string()),
            TokenKind::Skip => Ok("skip".to_string()),
            TokenKind::Trait => Ok("trait".to_string()),
            TokenKind::Impl => Ok("impl".to_string()),
            TokenKind::Async => Ok("async".to_string()),
            TokenKind::Await => Ok("await".to_string()),
            TokenKind::Mod => Ok("mod".to_string()),
            TokenKind::With => Ok("with".to_string()),
            TokenKind::Extern => Ok("extern".to_string()),
            TokenKind::Intent => Ok("intent".to_string()),
            TokenKind::Prove => Ok("prove".to_string()),
            TokenKind::Assume => Ok("assume".to_string()),
            TokenKind::Guarantee => Ok("guarantee".to_string()),
            TokenKind::Invariant => Ok("invariant".to_string()),
            TokenKind::Because => Ok("because".to_string()),
            TokenKind::Why => Ok("why".to_string()),
            TokenKind::Protect => Ok("protect".to_string()),
            TokenKind::Frozen => Ok("frozen".to_string()),
            TokenKind::MutableBy => Ok("mutable_by".to_string()),
            TokenKind::Owned => Ok("owned".to_string()),
            TokenKind::Handoff => Ok("handoff".to_string()),
            TokenKind::ReturnTo => Ok("return_to".to_string()),
            TokenKind::Compute => Ok("compute".to_string()),
            TokenKind::RaceFree => Ok("race_free".to_string()),
            TokenKind::Order => Ok("order".to_string()),
            TokenKind::Deterministic => Ok("deterministic".to_string()),
            TokenKind::Replay => Ok("replay".to_string()),
            TokenKind::Checkpoint => Ok("checkpoint".to_string()),
            TokenKind::Rollback => Ok("rollback".to_string()),
            TokenKind::Transaction => Ok("transaction".to_string()),
            TokenKind::Speculative => Ok("speculative".to_string()),
            TokenKind::Fallback => Ok("fallback".to_string()),
            TokenKind::Budget => Ok("budget".to_string()),
            TokenKind::Deadline => Ok("deadline".to_string()),
            TokenKind::Priority => Ok("priority".to_string()),
            TokenKind::Quality => Ok("quality".to_string()),
            TokenKind::Tradeoff => Ok("tradeoff".to_string()),
            TokenKind::Adapt => Ok("adapt".to_string()),
            TokenKind::Observe => Ok("observe".to_string()),
            TokenKind::Watch => Ok("watch".to_string()),
            TokenKind::React => Ok("react".to_string()),
            TokenKind::Stream => Ok("stream".to_string()),
            TokenKind::Flow => Ok("flow".to_string()),
            TokenKind::Choose => Ok("choose".to_string()),
            TokenKind::Race => Ok("race".to_string()),
            TokenKind::Hedge => Ok("hedge".to_string()),
            TokenKind::CancelSafe => Ok("cancel_safe".to_string()),
            TokenKind::Agent => Ok("agent".to_string()),
            TokenKind::Task => Ok("task".to_string()),
            TokenKind::Accept => Ok("accept".to_string()),
            TokenKind::Reject => Ok("reject".to_string()),
            TokenKind::Baseline => Ok("baseline".to_string()),
            TokenKind::Regression => Ok("regression".to_string()),
            TokenKind::Explain => Ok("explain".to_string()),
            TokenKind::Context => Ok("context".to_string()),
            TokenKind::Slice => Ok("slice".to_string()),
            TokenKind::Patch => Ok("patch".to_string()),
            TokenKind::Evolve => Ok("evolve".to_string()),
            TokenKind::Verify => Ok("verify".to_string()),
            TokenKind::Goal => Ok("goal".to_string()),
            TokenKind::Preserve => Ok("preserve".to_string()),
            TokenKind::Allow => Ok("allow".to_string()),
            TokenKind::To => Ok("to".to_string()),
            TokenKind::On => Ok("on".to_string()),
            TokenKind::MutateToken => Ok("mutate".to_string()),
            TokenKind::Boundary => Ok("boundary".to_string()),
            TokenKind::Responsibility => Ok("responsibility".to_string()),
            TokenKind::Owns => Ok("owns".to_string()),
            TokenKind::Exposes => Ok("exposes".to_string()),
            TokenKind::DependsOnly => Ok("depends_only".to_string()),
            TokenKind::Depends => Ok("depends".to_string()),
            TokenKind::Forbid => Ok("forbid".to_string()),
            TokenKind::Layer => Ok("layer".to_string()),
            TokenKind::Direction => Ok("direction".to_string()),
            TokenKind::Split => Ok("split".to_string()),
            TokenKind::Partition => Ok("partition".to_string()),
            TokenKind::Extract => Ok("extract".to_string()),
            TokenKind::Cluster => Ok("cluster".to_string()),
            TokenKind::Separate => Ok("separate".to_string()),
            TokenKind::Contract => Ok("contract".to_string()),
            TokenKind::Port => Ok("port".to_string()),
            TokenKind::Adapter => Ok("adapter".to_string()),
            TokenKind::Facade => Ok("facade".to_string()),
            TokenKind::Gateway => Ok("gateway".to_string()),
            TokenKind::Compat => Ok("compat".to_string()),
            TokenKind::Stable => Ok("stable".to_string()),
            TokenKind::Sealed => Ok("sealed".to_string()),
            TokenKind::Friend => Ok("friend".to_string()),
            TokenKind::PrivateTo => Ok("private_to".to_string()),
            TokenKind::Surface => Ok("surface".to_string()),
            TokenKind::Leak => Ok("leak".to_string()),
            TokenKind::Purity => Ok("purity".to_string()),
            TokenKind::View => Ok("view".to_string()),
            TokenKind::Lens => Ok("lens".to_string()),
            TokenKind::AgentScope => Ok("agent_scope".to_string()),
            TokenKind::BudgetContext => Ok("budget_context".to_string()),
            TokenKind::TokenBudget => Ok("token_budget".to_string()),
            TokenKind::Move => Ok("move".to_string()),
            TokenKind::Migrate => Ok("migrate".to_string()),
            TokenKind::Redirect => Ok("redirect".to_string()),
            TokenKind::Deprecate => Ok("deprecate".to_string()),
            TokenKind::CycleFree => Ok("cycle_free".to_string()),
            TokenKind::MaxFanout => Ok("max_fanout".to_string()),
            TokenKind::MaxFanin => Ok("max_fanin".to_string()),
            TokenKind::MaxDepth => Ok("max_dependency_depth".to_string()),
            TokenKind::Cohesion => Ok("cohesion".to_string()),
            TokenKind::Modularize => Ok("modularize".to_string()),
            TokenKind::Decompose => Ok("decompose".to_string()),
            TokenKind::Architecture => Ok("architecture".to_string()),
            TokenKind::Repair => Ok("repair".to_string()),
            TokenKind::Gravity => Ok("gravity".to_string()),
            TokenKind::Deny => Ok("deny".to_string()),
            TokenKind::Into => Ok("into".to_string()),
            TokenKind::From => Ok("from".to_string()),
            TokenKind::Toward => Ok("toward".to_string()),
            TokenKind::Optimize => Ok("optimize".to_string()),
            TokenKind::RejectIf => Ok("reject_if".to_string()),
            TokenKind::Never => Ok("never".to_string()),
            TokenKind::After => Ok("after".to_string()),
            TokenKind::Remove => Ok("remove".to_string()),
            TokenKind::Hide => Ok("hide".to_string()),
            TokenKind::Focus => Ok("focus".to_string()),
            TokenKind::By => Ok("by".to_string()),
            TokenKind::Through => Ok("through".to_string()),
            TokenKind::Bridge => Ok("bridge".to_string()),
            TokenKind::Derives => Ok("derives".to_string()),
            TokenKind::Override => Ok("override".to_string()),
            TokenKind::Extend => Ok("extend".to_string()),
            TokenKind::InlineC => Ok("inline_c".to_string()),
            TokenKind::Lease => Ok("lease".to_string()),
            TokenKind::Borrow => Ok("borrow".to_string()),
            TokenKind::During => Ok("during".to_string()),
            TokenKind::True => Ok("true".to_string()),
            TokenKind::False => Ok("false".to_string()),
            TokenKind::Operation => Ok("operation".to_string()),
            TokenKind::Event => Ok("event".to_string()),
            TokenKind::Hub => Ok("hub".to_string()),
            TokenKind::Emit => Ok("emit".to_string()),
            TokenKind::Compose => Ok("compose".to_string()),
            TokenKind::Retry => Ok("retry".to_string()),
            TokenKind::Repeat => Ok("repeat".to_string()),
            TokenKind::When => Ok("when".to_string()),
            TokenKind::Subscribes => Ok("subscribes".to_string()),
            TokenKind::Analyze => Ok("analyze".to_string()),
            TokenKind::Memoize => Ok("memoize".to_string()),
            TokenKind::Equivalent => Ok("equivalent".to_string()),
            TokenKind::Merge => Ok("merge".to_string()),
            TokenKind::Inline => Ok("inline".to_string()),
            TokenKind::Then => Ok("then".to_string()),
            TokenKind::Requires => Ok("requires".to_string()),
            TokenKind::Effects => Ok("effects".to_string()),
            TokenKind::Version => Ok("version".to_string()),
            TokenKind::Feature => Ok("feature".to_string()),
            TokenKind::Skill => Ok("skill".to_string()),
            TokenKind::Skills => Ok("skills".to_string()),
            TokenKind::Satisfies => Ok("satisfies".to_string()),
            TokenKind::Rules => Ok("rules".to_string()),
            TokenKind::Constraints => Ok("constraints".to_string()),
            TokenKind::Requirement => Ok("requirement".to_string()),
            TokenKind::Implements => Ok("implements".to_string()),
            TokenKind::Verifies => Ok("verifies".to_string()),
            TokenKind::Claim => Ok("claim".to_string()),
            TokenKind::Complete => Ok("complete".to_string()),
            TokenKind::Evidence => Ok("evidence".to_string()),
            TokenKind::Todo => Ok("todo".to_string()),
            TokenKind::Knowledge => Ok("knowledge".to_string()),
            TokenKind::Decision => Ok("decision".to_string()),
            TokenKind::Approval => Ok("approval".to_string()),
            TokenKind::Review => Ok("review".to_string()),
            TokenKind::ReviewBy => Ok("review_by".to_string()),
            TokenKind::Confidence => Ok("confidence".to_string()),
            TokenKind::Change => Ok("change".to_string()),
            TokenKind::AgentBoundary => Ok("agent_boundary".to_string()),
            TokenKind::AgentContext => Ok("agent_context".to_string()),
            TokenKind::ContextFirewall => Ok("context_firewall".to_string()),
            TokenKind::AgentApi => Ok("agent_api".to_string()),
            TokenKind::Agentability => Ok("agentability".to_string()),
            TokenKind::RegressionGuard => Ok("regression_guard".to_string()),
            TokenKind::Adversarial => Ok("adversarial".to_string()),
            TokenKind::Tasks => Ok("tasks".to_string()),
            TokenKind::Profile => Ok("profile".to_string()),
            TokenKind::Hard => Ok("hard".to_string()),
            TokenKind::Soft => Ok("soft".to_string()),
            TokenKind::Structural => Ok("structural".to_string()),
            TokenKind::Semantic => Ok("semantic".to_string()),
            TokenKind::Behavioral => Ok("behavioral".to_string()),
            TokenKind::Performance => Ok("performance".to_string()),
            TokenKind::Security => Ok("security".to_string()),
            TokenKind::Testing => Ok("testing".to_string()),
            TokenKind::Summary => Ok("summary".to_string()),
            TokenKind::Risks => Ok("risks".to_string()),
            TokenKind::Recommendation => Ok("recommendation".to_string()),
            TokenKind::Notes => Ok("notes".to_string()),
            other => Err(format!("Expected identifier, found {:?} at line {}", other, tok.span.line)),
        }
    }

    pub fn parse_identifier_or_string(&mut self) -> Result<String, String> {
        let mut res = match self.peek_kind() {
            TokenKind::StringLit(s) => {
                let r = s.clone();
                self.advance();
                r
            }
            _ => self.parse_identifier_or_keyword()?,
        };

        while self.match_token(&TokenKind::Dot) {
            let next_part = self.parse_identifier_or_keyword()?;
            res.push('.');
            res.push_str(&next_part);
        }

        Ok(res)
    }

    pub fn parse_string_list(&mut self) -> Result<Vec<String>, String> {
        let mut list = Vec::new();
        if self.match_token(&TokenKind::LBracket) {
            while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::EOF) {
                if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                    let mut nested = self.parse_string_list()?;
                    list.append(&mut nested);
                } else {
                    list.push(self.parse_identifier_or_string()?);
                }
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBracket)?;
        } else if self.match_token(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                if self.check(&TokenKind::LBracket) || self.check(&TokenKind::LBrace) {
                    let mut nested = self.parse_string_list()?;
                    list.append(&mut nested);
                } else {
                    list.push(self.parse_identifier_or_string()?);
                }
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            list.push(self.parse_identifier_or_string()?);
        }
        Ok(list)
    }

    pub fn parse_key_value_pairs(&mut self) -> Result<Vec<(String, String)>, String> {
        let mut pairs = Vec::new();
        if self.match_token(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let k = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Colon);
                let v = self.parse_identifier_or_string()?;
                pairs.push((k, v));
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
            self.expect(TokenKind::RBrace)?;
        }
        Ok(pairs)
    }


    pub fn parse_module(&mut self, module_name: &str) -> Result<Module, String> {
        let mut imports = Vec::new();
        let mut enums = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut functions = Vec::new();
        let mut modules = Vec::new();
        let mut extensions = Vec::new();
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

            match self.peek_kind() {
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
                    functions.push(self.parse_function(false, pending_directives)?);
                }
                TokenKind::Extern => {
                    self.advance();
                    let mut f = self.parse_function(false, pending_directives)?;
                    f.directives.push(Directive { name: "@extern".to_string(), args: vec![], span: f.span.clone() });
                    functions.push(f);
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
                    extensions.push(self.parse_extension_block()?);
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
                            functions.push(self.parse_function(true, pending_directives)?);
                        }
                        TokenKind::Extern => {
                            self.advance();
                            let mut f = self.parse_function(true, pending_directives)?;
                            f.directives.push(Directive { name: "@extern".to_string(), args: vec![], span: f.span.clone() });
                            functions.push(f);
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
                        TokenKind::Hub => {
                            statements.push(Statement::EventHubDecl(self.parse_event_hub(true)?));
                        }
                        other => {
                            return Err(format!(
                                "Expected enum, struct, trait, val or fn after 'pub', found {:?} at line {}",
                                other,
                                self.current_span().line
                            ))
                        }
                    }
                }
                TokenKind::SemiColon => {
                    self.advance();
                }
                TokenKind::EOF => break,
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt);
                }
            }
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
            statements,
            span: start_span,
        })
    }

    fn parse_import(&mut self) -> Result<ImportStmt, String> {
        let span = self.current_span();
        self.expect(TokenKind::Import)?;

        let (kind, path) = match self.peek_kind() {
            TokenKind::Directive(d) => {
                let dir = d.clone();
                self.advance();
                self.expect(TokenKind::LParen)?;
                let p = match self.advance().kind {
                    TokenKind::StringLit(s) => s,
                    other => return Err(format!("Expected string path in import directive, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;

                match dir.as_str() {
                    "@c" => (ImportKind::C(p.clone()), p),
                    "@zig" => (ImportKind::Zig(p.clone()), p),
                    "@rust" => (ImportKind::Rust(p.clone()), p),
                    "@go" => (ImportKind::Go(p.clone()), p),
                    _ => (ImportKind::Standard, p),
                }
            }
            TokenKind::StringLit(s) => {
                let p = s.clone();
                self.advance();
                (ImportKind::Standard, p)
            }
            TokenKind::Ident(_) => {
                let mut full_path = String::new();
                while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::As) && !self.check(&TokenKind::EOF) {
                    match self.peek_kind() {
                        TokenKind::Ident(id) => {
                            full_path.push_str(id);
                            self.advance();
                        }
                        TokenKind::Dot => {
                            full_path.push('.');
                            self.advance();
                        }
                        TokenKind::Star => {
                            full_path.push('*');
                            self.advance();
                        }
                        _ => break,
                    }
                }
                (ImportKind::Standard, full_path.clone())
            }
            other => return Err(format!("Invalid import syntax: {:?} at line {}", other, span.line)),
        };

        let mut alias = None;
        if self.match_token(&TokenKind::As) {
            match self.advance().kind {
                TokenKind::Ident(a) => alias = Some(a),
                other => return Err(format!("Expected alias identifier after 'as', found {:?}", other)),
            }
        }

        self.match_token(&TokenKind::SemiColon);

        Ok(ImportStmt {
            kind,
            path,
            alias,
            span,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let _span = self.current_span();
        if self.match_token(&TokenKind::Bang) {
            let inner = self.parse_type()?;
            return Ok(Type::Result(Box::new(inner), None));
        }

        if self.match_token(&TokenKind::Star) {
            let inner = self.parse_type()?;
            return Ok(Type::Pointer(Box::new(inner)));
        }

        if self.match_token(&TokenKind::LBracket) {
            if self.match_token(&TokenKind::RBracket) {
                let inner = self.parse_type()?;
                return Ok(Type::Slice(Box::new(inner)));
            } else if let TokenKind::IntLit(n) = self.peek_kind() {
                let size = *n as usize;
                self.advance();
                self.expect(TokenKind::RBracket)?;
                let inner = self.parse_type()?;
                return Ok(Type::Array(Box::new(inner), size));
            }
        }

        match self.peek_kind() {
            TokenKind::Ident(_)
            | TokenKind::Sealed
            | TokenKind::Contract
            | TokenKind::Security
            | TokenKind::Boundary
            | TokenKind::Stable
            | TokenKind::Compat
            | TokenKind::Purity => {
                let type_name = self.parse_identifier_or_keyword()?;
                let ty = match type_name.as_str() {
                    "void" => Type::Void,
                    "bool" => Type::Bool,
                    "i8" => Type::I8,
                    "i16" => Type::I16,
                    "i32" | "int" => Type::I32,
                    "i64" => Type::I64,
                    "u8" | "byte" => Type::U8,
                    "u16" => Type::U16,
                    "u32" => Type::U32,
                    "u64" => Type::U64,
                    "f32" | "float" => Type::F32,
                    "f64" => Type::F64,
                    "f32x4" => Type::Simd(Box::new(Type::F32), 4),
                    "f32x8" => Type::Simd(Box::new(Type::F32), 8),
                    "i32x4" => Type::Simd(Box::new(Type::I32), 4),
                    "i32x8" => Type::Simd(Box::new(Type::I32), 8),
                    "str" | "string" => Type::Str,
                    "Allocator" => Type::Allocator,
                    "Box" | "box" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Box(Box::new(inner))
                    }
                    "Rc" | "rc" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Rc(Box::new(inner))
                    }
                    "Arc" | "arc" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Arc(Box::new(inner))
                    }
                    "Channel" | "channel" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Channel(Box::new(inner))
                    }
                    "Operation" | "operation" | "Op" | "op" => {
                        if self.match_token(&TokenKind::Less) {
                            let tin = self.parse_type()?;
                            let mut tout = None;
                            if self.match_token(&TokenKind::Comma) {
                                tout = Some(Box::new(self.parse_type()?));
                            }
                            self.expect(TokenKind::Greater)?;
                            Type::Operation(Some(Box::new(tin)), tout)
                        } else {
                            Type::Operation(None, None)
                        }
                    }
                    "OperationResult" => Type::OperationResult,
                    "Event" | "event" => {
                        if self.match_token(&TokenKind::Less) {
                            let ev_name = self.parse_identifier_or_keyword()?;
                            self.expect(TokenKind::Greater)?;
                            Type::Event(ev_name)
                        } else {
                            Type::Event("Any".into())
                        }
                    }
                    "region" => {
                        if self.match_token(&TokenKind::Less) {
                            let reg_name = match self.advance().kind {
                                TokenKind::Ident(s) => s,
                                other => return Err(format!("Expected region name, found {:?}", other)),
                            };
                            self.expect(TokenKind::Greater)?;
                            Type::Region(reg_name)
                        } else {
                            Type::Region("default".into())
                        }
                    }
                    other => {
                        if self.match_token(&TokenKind::Less) {
                            let mut params = Vec::new();
                            while !self.check(&TokenKind::Greater) && !self.check(&TokenKind::EOF) {
                                params.push(self.parse_type()?);
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(TokenKind::Greater)?;
                            Type::Generic(other.to_string(), params)
                        } else {
                            Type::Custom(other.to_string())
                        }
                    }
                };
                Ok(ty)
            }
            TokenKind::Operation => {
                self.advance();
                if self.match_token(&TokenKind::Less) {
                    let tin = self.parse_type()?;
                    let mut tout = None;
                    if self.match_token(&TokenKind::Comma) {
                        tout = Some(Box::new(self.parse_type()?));
                    }
                    self.expect(TokenKind::Greater)?;
                    Ok(Type::Operation(Some(Box::new(tin)), tout))
                } else {
                    Ok(Type::Operation(None, None))
                }
            }
            TokenKind::Event => {
                self.advance();
                if self.match_token(&TokenKind::Less) {
                    let ev_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::Greater)?;
                    Ok(Type::Event(ev_name))
                } else {
                    Ok(Type::Event("Any".into()))
                }
            }
            other => Err(format!("Expected type, found {:?} at line {}", other, self.current_span().line)),
        }
    }

    fn parse_enum(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<EnumDef, String> {
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
                payload = Some(self.parse_type()?);
                self.expect(TokenKind::RParen)?;
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

    fn parse_struct(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<StructDef, String> {
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

        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let field_span = self.current_span();
            let is_field_pub = self.match_token(&TokenKind::Pub);
            let field_name = self.parse_identifier_or_keyword()?;

            self.expect(TokenKind::Colon)?;
            let field_type = self.parse_type()?;
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

        Ok(StructDef {
            name,
            generic_params,
            is_pub,
            fields,
            directives,
            span,
        })
    }

    fn parse_function(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<FunctionDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Fn)?;

        let (name, morphic_param) = match self.advance().kind {
            TokenKind::Ident(n) => (n, None),
            TokenKind::MorphicIdent(m) => {
                let p = if m.starts_with('{') && m.contains('}') {
                    let end_brace = m.find('}').unwrap();
                    Some(m[1..end_brace].to_string())
                } else {
                    None
                };
                (m, p)
            }
            other => return Err(format!("Expected function name, found {:?} at line {}", other, span.line)),
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
            let mut param_name = self.parse_identifier_or_keyword()?;
            if is_ref {
                param_name = format!("&{}", param_name);
            }

            let mut param_type = Type::Void;
            if self.match_token(&TokenKind::Colon) {
                param_type = self.parse_type()?;
            } else if param_name == "&self" || param_name == "self" {
                param_type = Type::Custom("Self".to_string());
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

    fn parse_operation(&mut self, is_pub: bool) -> Result<OperationDef, String> {
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

    fn parse_event(&mut self, is_pub: bool) -> Result<EventDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Event)?;
        let name = self.parse_identifier_or_keyword()?;
        let mut fields = Vec::new();

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
            fields,
            span,
        })
    }

    fn parse_event_hub(&mut self, is_pub: bool) -> Result<EventHubDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Hub)?;
        let name = self.parse_identifier_or_keyword()?;
        self.expect(TokenKind::LBrace)?;
        let mut owns_events = Vec::new();
        let mut handlers = Vec::new();

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
            span,
        })
    }

    fn parse_trait(&mut self, is_pub: bool) -> Result<TraitDef, String> {
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

            self.match_token(&TokenKind::SemiColon);

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

    fn parse_module_def(&mut self, is_pub: bool, _directives: Vec<Directive>) -> Result<ModuleDef, String> {
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

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let mut pending_directives = Vec::new();
            while let TokenKind::Directive(d) = self.peek_kind() {
                let dir_name = d.clone();
                let dir_span = self.current_span();
                self.advance();
                pending_directives.push(Directive {
                    name: dir_name,
                    args: Vec::new(),
                    span: dir_span,
                });
            }
            match self.peek_kind() {
                TokenKind::Responsibility => {
                    self.advance();
                    self.match_token(&TokenKind::Colon);
                    responsibility = Some(self.parse_identifier_or_string()?);
                    self.match_token(&TokenKind::SemiColon);
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
            responsibility,
            owns,
            exposes,
            depends,
            depends_only,
            forbid,
            is_sealed,
            purity,
            cohesion,
            structs,
            functions,
            overrides,
            statements,
            span,
        })
    }

    fn parse_extension_block(&mut self) -> Result<ExtensionBlock, String> {
        let span = self.current_span();
        self.expect(TokenKind::Extend)?;
        let is_struct = if self.match_token(&TokenKind::Struct) {
            true
        } else if self.match_token(&TokenKind::Mod) {
            false
        } else {
            true
        };
        let target = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected target identifier in extend block, found {:?}", other)),
        };
        self.expect(TokenKind::LBrace)?;
        let mut functions = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let mut pending_directives = Vec::new();
            while let TokenKind::Directive(d) = self.peek_kind() {
                let dir_name = d.clone();
                let dir_span = self.current_span();
                self.advance();
                pending_directives.push(Directive {
                    name: dir_name,
                    args: Vec::new(),
                    span: dir_span,
                });
            }
            match self.peek_kind() {
                TokenKind::Fn => {
                    functions.push(self.parse_function(false, pending_directives)?);
                }
                TokenKind::Pub => {
                    self.advance();
                    if self.check(&TokenKind::Fn) {
                        functions.push(self.parse_function(true, pending_directives)?);
                    }
                }
                TokenKind::SemiColon => { self.advance(); }
                _ => { self.advance(); }
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(ExtensionBlock {
            target,
            is_struct,
            functions,
            span,
        })
    }

    fn parse_impl(&mut self) -> Result<ImplBlock, String> {
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

    fn parse_block(&mut self) -> Result<Block, String> {
        let span = self.current_span();
        self.expect(TokenKind::LBrace)?;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Block { statements, span })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let span = self.current_span();

        match self.peek_kind() {
            TokenKind::ValBang => {
                self.advance();
                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected variable name after val!, found {:?}", other)),
                };
                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }
                self.expect(TokenKind::Equal)?;
                let expr = self.parse_expression()?;
                let fallback = if self.match_token(&TokenKind::QuestionQuestion) {
                    self.parse_expression()?
                } else {
                    Expression::Lit(Literal::Int(0), span.clone())
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::QuantumUnwrap {
                    name,
                    var_type,
                    expr,
                    fallback,
                    span,
                })
            }
            TokenKind::Lease | TokenKind::Borrow => {
                self.advance();

                // lease cpu(cores, priority) { body }
                if let TokenKind::Ident(ref peek_id) = self.peek_kind().clone() {
                    if peek_id == "cpu" {
                        self.advance(); // consume "cpu"
                        self.expect(TokenKind::LParen)?;
                        let cores = self.parse_expression()?;
                        let priority = if self.match_token(&TokenKind::Comma) {
                            Some(self.parse_expression()?)
                        } else {
                            None
                        };
                        self.expect(TokenKind::RParen)?;
                        let body = self.parse_block()?;
                        return Ok(Statement::LeaseCpu {
                            cores,
                            priority,
                            body,
                            span,
                        });
                    }

                    // lease listen(event_expr) while condition { body }
                    if peek_id == "listen" {
                        self.advance(); // consume "listen"
                        self.expect(TokenKind::LParen)?;
                        let event_expr = self.parse_expression()?;
                        self.expect(TokenKind::RParen)?;
                        let mut condition = None;
                        if self.match_token(&TokenKind::While) || self.match_token(&TokenKind::During) {
                            if !self.check(&TokenKind::LBrace) {
                                condition = Some(self.parse_expression()?);
                            }
                        }
                        let body = self.parse_block()?;
                        return Ok(Statement::LeaseEvent {
                            event_expr,
                            condition,
                            body,
                            span,
                        });
                    }

                    // lease loop(budget) for item in iterable { body }
                    if peek_id == "loop" {
                        self.advance(); // consume "loop"
                        self.expect(TokenKind::LParen)?;
                        let budget = self.parse_expression()?;
                        self.expect(TokenKind::RParen)?;
                        self.expect(TokenKind::For)?;
                        let item_name = self.parse_identifier_or_keyword()?;
                        self.expect(TokenKind::In)?;
                        let iterable = self.parse_expression()?;
                        let body = self.parse_block()?;
                        return Ok(Statement::LeaseLoop {
                            budget: Some(budget),
                            item_name,
                            iterable,
                            body,
                            span,
                        });
                    }
                }

                // lease for item in iterable { body }  (zero-allocation fused loop)
                if self.check(&TokenKind::For) {
                    self.advance(); // consume "for"
                    let item_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::In)?;
                    let iterable = self.parse_expression()?;
                    let body = self.parse_block()?;
                    return Ok(Statement::LeaseLoop {
                        budget: None,
                        item_name,
                        iterable,
                        body,
                        span,
                    });
                }

                // Existing: lease val name = expr { body } / lease val name = expr;
                let is_mut = if self.match_token(&TokenKind::Mut) {
                    true
                } else {
                    self.match_token(&TokenKind::Val);
                    false
                };

                let name = self.parse_identifier_or_keyword()?;

                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut owner = String::new();
                    let mut duration = "task".to_string();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "owner" {
                            owner = self.parse_identifier_or_string()?;
                        } else if key == "duration" {
                            duration = self.parse_identifier_or_string()?;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::AgentLeaseDecl { module_name: name, owner, duration, span });
                }

                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }

                self.expect(TokenKind::Equal)?;
                let initializer = self.parse_expression()?;

                let mut condition = None;
                if self.match_token(&TokenKind::While) || self.match_token(&TokenKind::During) {
                    if !self.check(&TokenKind::LBrace) {
                        condition = Some(self.parse_expression()?);
                    }
                }

                if self.check(&TokenKind::LBrace) {
                    let body = self.parse_block()?;
                    Ok(Statement::LeaseBlock {
                        name,
                        var_type,
                        initializer,
                        condition,
                        body,
                        span,
                    })
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::VarDecl {
                        name,
                        var_type,
                        is_mut,
                        is_lease: true,
                        initializer: Some(initializer),
                        span,
                    })
                }
            }
            TokenKind::Val | TokenKind::Mut => {
                let is_mut = self.peek_kind() == &TokenKind::Mut;
                self.advance();

                let name = self.parse_identifier_or_keyword()?;

                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }

                let mut initializer = None;
                if self.match_token(&TokenKind::Equal) {
                    initializer = Some(self.parse_expression()?);
                }

                self.match_token(&TokenKind::SemiColon);

                Ok(Statement::VarDecl {
                    name,
                    var_type,
                    is_mut,
                    is_lease: false,
                    initializer,
                    span,
                })
            }
            TokenKind::Return => {
                self.advance();
                let value = if !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::RBrace) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Return { value, span })
            }
            TokenKind::If => {
                self.advance();
                let condition = self.parse_expression()?;
                let then_block = self.parse_block()?;
                let mut else_block = None;
                if self.match_token(&TokenKind::Else) {
                    if self.check(&TokenKind::If) {
                        let if_stmt = self.parse_statement()?;
                        else_block = Some(Block {
                            statements: vec![if_stmt],
                            span: self.current_span(),
                        });
                    } else {
                        else_block = Some(self.parse_block()?);
                    }
                }
                Ok(Statement::If {
                    condition,
                    then_block,
                    else_block,
                    span,
                })
            }
            TokenKind::While => {
                self.advance();
                let condition = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Statement::While {
                    condition,
                    body,
                    span,
                })
            }
            TokenKind::Parallel => {
                self.advance();
                if self.match_token(&TokenKind::Choose) {
                    self.expect(TokenKind::LBrace)?;
                    let mut branches = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let name = self.parse_identifier_or_keyword()?;
                        self.expect(TokenKind::FatArrow)?;
                        let blk = if self.check(&TokenKind::LBrace) {
                            self.parse_block()?
                        } else {
                            let s = self.parse_statement()?;
                            Block { statements: vec![s], span: self.current_span() }
                        };
                        branches.push((name, blk));
                        self.match_token(&TokenKind::Comma);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::ParallelChoose { branches, span });
                }
                self.expect(TokenKind::For)?;
                let item_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected item name after 'parallel for', found {:?}", other)),
                };
                self.expect(TokenKind::In)?;
                let iterable = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Statement::ParallelFor {
                    item_name,
                    iterable,
                    body,
                    span,
                })
            }
            TokenKind::For => {
                self.advance();
                let item_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected item name after 'for', found {:?}", other)),
                };
                self.expect(TokenKind::In)?;
                let iterable = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Statement::ForIn {
                    item_name,
                    iterable,
                    body,
                    span,
                })
            }
            TokenKind::Match => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::LBrace)?;
                let mut arms = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    arms.push(self.parse_match_arm()?);
                    self.match_token(&TokenKind::Comma);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::Match { expr, arms, span })
            }
            TokenKind::Region => {
                self.advance();
                let reg_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected region name, found {:?}", other)),
                };
                let body = self.parse_block()?;
                Ok(Statement::RegionBlock {
                    name: reg_name,
                    body,
                    span,
                })
            }
            TokenKind::InlineC => {
                let span = self.current_span();
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut code = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    match self.peek_kind() {
                        TokenKind::StringLit(s) => {
                            code.push_str(s);
                            code.push('\n');
                            self.advance();
                        }
                        _ => {
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::InlineC { code, span })
            }
            TokenKind::Asm => {
                self.advance();
                let arch = match self.advance().kind {
                    TokenKind::Ident(n) | TokenKind::StringLit(n) => n,
                    other => return Err(format!("Expected target architecture for asm, found {:?}", other)),
                };
                self.expect(TokenKind::LBrace)?;
                let mut asm_code = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    match self.peek_kind() {
                        TokenKind::StringLit(s) => {
                            asm_code.push_str(s);
                            asm_code.push('\n');
                            self.advance();
                        }
                        _ => {
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AsmBlock {
                    arch,
                    code: asm_code,
                    span,
                })
            }
            TokenKind::Target => {
                self.advance();
                let target_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected target name after 'target', found {:?}", other)),
                };
                let body = self.parse_block()?;
                Ok(Statement::TargetBlock {
                    target: target_name,
                    body,
                    span,
                })
            }
            TokenKind::Defer => {
                self.advance();
                let expr = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Defer { expr, span })
            }
            TokenKind::Spawn => {
                self.advance();
                let call = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Spawn { call, span })
            }
            TokenKind::Skip => {
                self.advance();
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Skip { span })
            }
            TokenKind::Intent => {
                self.advance();
                if let TokenKind::Ident(ref id) = self.peek_kind().clone() {
                    if id == "diff" {
                        self.advance();
                        self.expect(TokenKind::LBrace)?;
                        let mut preserve = Vec::new();
                        let mut change = Vec::new();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let key = self.parse_identifier_or_keyword()?;
                            self.expect(TokenKind::Colon)?;
                            let list = self.parse_string_list()?;
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                            if key == "preserve" {
                                preserve = list;
                            } else if key == "change" || key == "allow" {
                                change = list;
                            }
                        }
                        self.expect(TokenKind::RBrace)?;
                        return Ok(Statement::IntentDiff { preserve, change, span });
                    }
                }

                let mut name = None;
                let mut goal = String::new();
                let mut preserve = Vec::new();
                let mut optimize = Vec::new();

                match self.peek_kind() {
                    TokenKind::StringLit(s) => {
                        goal = s.clone();
                        self.advance();
                    }
                    TokenKind::Ident(_) => {
                        name = Some(self.parse_identifier_or_keyword()?);
                    }
                    _ => {}
                }

                let mut body = None;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut stmts = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Goal) {
                            self.advance();
                            self.match_token(&TokenKind::Colon);
                            goal = self.parse_identifier_or_string()?;
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else if self.check(&TokenKind::Preserve) {
                            self.advance();
                            self.match_token(&TokenKind::Colon);
                            preserve = self.parse_string_list()?;
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else if self.check(&TokenKind::Optimize) {
                            self.advance();
                            self.match_token(&TokenKind::Colon);
                            optimize = self.parse_string_list()?;
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else {
                            if let Ok(stmt) = self.parse_statement() {
                                stmts.push(stmt);
                            } else {
                                let key = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::Colon);
                                if key == "goal" {
                                    goal = self.parse_identifier_or_string()?;
                                } else if key == "preserve" {
                                    preserve = self.parse_string_list()?;
                                } else if key == "optimize" {
                                    optimize = self.parse_string_list()?;
                                }
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    if !stmts.is_empty() {
                        body = Some(Block { statements: stmts, span: span.clone() });
                    }
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }

                if name.is_none() && body.is_none() && (!goal.is_empty() || !preserve.is_empty() || !optimize.is_empty()) {
                    Ok(Statement::IntentDecl { goal, preserve, optimize, span })
                } else {
                    Ok(Statement::Intent { name, goal, preserve, body, span })
                }
            }
            TokenKind::Prove => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Prove { condition, span })
            }
            TokenKind::Assume => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Assume { condition, span })
            }
            TokenKind::Guarantee => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Guarantee { condition, span })
            }
            TokenKind::Invariant => {
                self.advance();
                if let TokenKind::StringLit(s) = self.peek_kind() {
                    let s_val = s.clone();
                    self.advance();
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ArchInvariantDecl { rule: s_val, span });
                }
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let e = self.parse_expression()?;
                    self.expect(TokenKind::RBrace)?;
                    e
                } else {
                    let e = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    e
                };
                Ok(Statement::Invariant { condition, span })
            }
            TokenKind::Verify => {
                self.advance();
                if self.check(&TokenKind::Adversarial) {
                    self.advance();
                    self.expect(TokenKind::LBrace)?;
                    let mut skill = None;
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "skill" {
                            skill = Some(self.parse_identifier_or_string()?);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    Ok(Statement::VerifyTask { target: "adversarial".to_string(), is_adversarial: true, skill, span })
                } else if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut invariants = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        invariants.push(self.parse_expression()?);
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    Ok(Statement::VerifyBlock { invariants, span })
                } else {
                    let target = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::VerifyTask { target, is_adversarial: false, skill: None, span })
                }
            }
            TokenKind::Because => {
                self.advance();
                let rationale = match self.peek_kind() {
                    TokenKind::StringLit(s) => {
                        let r = s.clone();
                        self.advance();
                        r
                    }
                    _ => self.parse_identifier_or_keyword()?,
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Because { rationale, span })
            }
            TokenKind::Why => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let mut rationale = String::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if let TokenKind::StringLit(s) = self.peek_kind() {
                            rationale.push_str(s);
                            self.advance();
                        } else {
                            rationale.push_str(&self.parse_identifier_or_keyword()?);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else if self.match_token(&TokenKind::Colon) {
                    rationale = self.parse_identifier_or_string()?;
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::Why { target, rationale, span })
            }
            TokenKind::Protect => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::ProtectBlock { body, span })
            }
            TokenKind::Frozen => {
                self.advance();
                let symbol = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Frozen { symbol, span })
            }
            TokenKind::MutableBy => {
                self.advance();
                let mut roles = Vec::new();
                while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                    roles.push(self.parse_identifier_or_keyword()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::MutableBy { roles, span })
            }
            TokenKind::Owned => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }
                self.expect(TokenKind::Equal)?;
                let initializer = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Owned { name, var_type, initializer, span })
            }
            TokenKind::Handoff => {
                self.advance();
                let resource = self.parse_identifier_or_keyword()?;
                if !self.match_token(&TokenKind::Arrow) {
                    self.match_token(&TokenKind::To);
                }
                let target_domain = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Handoff { resource, target_domain, span })
            }
            TokenKind::ReturnTo => {
                self.advance();
                let source_domain = self.parse_identifier_or_keyword()?;
                let resource = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ReturnTo { source_domain, resource, span })
            }
            TokenKind::Compute => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let body = self.parse_block()?;
                let mut fallback = None;
                if self.match_token(&TokenKind::Fallback) {
                    let _ = self.parse_identifier_or_keyword().ok();
                    fallback = Some(self.parse_block()?);
                }
                Ok(Statement::ComputeBlock { target, body, fallback, span })
            }
            TokenKind::RaceFree => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::RaceFreeBlock { body, span })
            }
            TokenKind::Order => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let mode = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Order { mode, span })
            }
            TokenKind::Deterministic => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::DeterministicBlock { body, span })
            }
            TokenKind::Replay => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::ReplayBlock { body, span })
            }
            TokenKind::Checkpoint => {
                self.advance();
                let state_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Checkpoint { state_name, span })
            }
            TokenKind::Rollback => {
                self.advance();
                self.match_token(&TokenKind::To);
                let checkpoint_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Rollback { checkpoint_name, span })
            }
            TokenKind::Transaction => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::TransactionBlock { body, span })
            }
            TokenKind::Speculative => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::SpeculativeBlock { body, span })
            }
            TokenKind::Fallback => {
                self.advance();
                let target = self.parse_identifier_or_keyword().unwrap_or_else(|_| "default".to_string());
                let body = self.parse_block()?;
                Ok(Statement::FallbackBlock { target, body, span })
            }
            TokenKind::Budget => {
                self.advance();
                let specs = self.parse_key_value_pairs()?;
                let mut body = None;
                if self.check(&TokenKind::LBrace) {
                    body = Some(self.parse_block()?);
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::BudgetBlock { specs, body, span })
            }
            TokenKind::Deadline => {
                self.advance();
                let duration = self.parse_identifier_or_string()?;
                let body = self.parse_block()?;
                Ok(Statement::DeadlineBlock { duration, body, span })
            }
            TokenKind::Priority => {
                self.advance();
                let level = self.parse_identifier_or_keyword()?;
                let body = self.parse_block()?;
                Ok(Statement::PriorityBlock { level, body, span })
            }
            TokenKind::Quality => {
                self.advance();
                let pairs = self.parse_key_value_pairs()?;
                let min_metric = pairs.iter().find(|(k, _)| k == "min" || k == "min_metric").map(|(_, v)| v.clone()).unwrap_or_else(|| "1.0".to_string());
                let max_latency = pairs.iter().find(|(k, _)| k == "max_latency" || k == "latency").map(|(_, v)| v.clone()).unwrap_or_else(|| "16ms".to_string());
                let body = self.parse_block()?;
                Ok(Statement::QualityBlock { min_metric, max_latency, body, span })
            }
            TokenKind::Tradeoff => {
                self.advance();
                let pairs = self.parse_key_value_pairs()?;
                let prefer = pairs.iter().find(|(k, _)| k == "prefer").map(|(_, v)| v.clone()).unwrap_or_else(|| "latency".to_string());
                let sacrifice = pairs.iter().find(|(k, _)| k == "sacrifice").map(|(_, v)| v.clone()).unwrap_or_else(|| "memory".to_string());
                let body = self.parse_block()?;
                Ok(Statement::TradeoffBlock { prefer, sacrifice, body, span })
            }
            TokenKind::Adapt => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut branches = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    self.match_token(&TokenKind::If);
                    let cond = self.parse_expression()?;
                    self.expect(TokenKind::FatArrow)?;
                    let blk = if self.check(&TokenKind::LBrace) {
                        self.parse_block()?
                    } else {
                        let stmt = self.parse_statement()?;
                        Block { statements: vec![stmt], span: self.current_span() }
                    };
                    branches.push((cond, blk));
                    self.match_token(&TokenKind::Comma);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AdaptBlock { branches, span })
            }
            TokenKind::Observe => {
                self.advance();
                let ident_name = self.parse_identifier_or_keyword()?;
                let op_expr = Expression::Ident(ident_name, span.clone());
                if self.match_token(&TokenKind::As) {
                    let alias = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::ObserveOp { op_expr, alias, span })
                } else {
                    let mut metrics = vec![if let Expression::Ident(id, _) = &op_expr { id.clone() } else { format!("{:?}", op_expr) }];
                    while self.match_token(&TokenKind::Comma) {
                        metrics.push(self.parse_identifier_or_keyword()?);
                    }
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::Observe { metrics, span })
                }
            }
            TokenKind::Emit => {
                self.advance();
                let event_name = self.parse_identifier_or_keyword()?;
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
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::EmitEvent {
                    event_name,
                    args,
                    span,
                })
            }
            TokenKind::Operation => {
                Ok(Statement::OperationDecl(self.parse_operation(false)?))
            }
            TokenKind::Event => {
                Ok(Statement::EventDecl(self.parse_event(false)?))
            }
            TokenKind::Hub => {
                Ok(Statement::EventHubDecl(self.parse_event_hub(false)?))
            }
            TokenKind::Watch => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut event = "mutate".to_string();
                if self.match_token(&TokenKind::On) {
                    event = self.parse_identifier_or_keyword()?;
                }
                self.match_token(&TokenKind::FatArrow);
                let handler = if self.check(&TokenKind::LBrace) {
                    self.parse_block()?
                } else {
                    let s = self.parse_statement()?;
                    Block { statements: vec![s], span: self.current_span() }
                };
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::WatchBlock { target, event, handler, span })
            }
            TokenKind::React => {
                self.advance();
                self.match_token(&TokenKind::To);
                let event = self.parse_expression()?;
                let handler = self.parse_block()?;
                Ok(Statement::ReactBlock { event, handler, span })
            }
            TokenKind::Stream => {
                self.advance();
                let source = self.parse_expression()?;
                self.expect(TokenKind::LBrace)?;
                let mut operations = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    operations.push(self.parse_expression()?);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::StreamBlock { source, operations, span })
            }
            TokenKind::Flow => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut steps = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    steps.push(self.parse_expression()?);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::FlowBlock { steps, span })
            }
            TokenKind::Race => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut branches = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    if self.check(&TokenKind::LBrace) {
                        branches.push(self.parse_block()?);
                    } else {
                        let stmt = self.parse_statement()?;
                        branches.push(Block { statements: vec![stmt], span: self.current_span() });
                    }
                    self.match_token(&TokenKind::Comma);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::RaceBlock { branches, span })
            }
            TokenKind::Hedge => {
                self.advance();
                let mut delay_ms = Expression::Lit(Literal::Int(20), span.clone());
                if self.peek_kind() != &TokenKind::LBrace {
                    let _ = self.parse_identifier_or_keyword().ok();
                    delay_ms = self.parse_expression()?;
                    if matches!(self.peek_kind(), TokenKind::Ident(id) if id == "ms" || id == "s" || id == "us") {
                        self.advance();
                    }
                }
                let primary = self.parse_block()?;
                let mut fallback = Block { statements: vec![], span: span.clone() };
                if self.match_token(&TokenKind::Fallback) {
                    fallback = self.parse_block()?;
                }
                Ok(Statement::HedgeBlock { delay_ms, primary, fallback, span })
            }
            TokenKind::CancelSafe => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Statement::CancelSafeBlock { body, span })
            }
            TokenKind::Agent => {
                self.advance();
                if self.check(&TokenKind::Lease) {
                    self.advance();
                    let module_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::LBrace)?;
                    let mut owner = String::new();
                    let mut duration = "task".to_string();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "owner" {
                            owner = self.parse_identifier_or_string()?;
                        } else if key == "duration" {
                            duration = self.parse_identifier_or_string()?;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::AgentLeaseDecl { module_name, owner, duration, span });
                }
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut capabilities = Vec::new();
                    let mut cannot = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        let list = self.parse_string_list()?;
                        if key == "capabilities" || key == "capability" {
                            capabilities.extend(list);
                        } else if key == "cannot" {
                            cannot.extend(list);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::AgentCapabilityDecl { capabilities, cannot, span });
                }
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut scope = String::new();
                let mut goal = String::new();
                let mut constraints = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "scope" {
                        scope = self.parse_identifier_or_string()?;
                    } else if key == "goal" {
                        goal = self.parse_identifier_or_string()?;
                    } else if key == "constraints" {
                        constraints = self.parse_string_list()?;
                    } else {
                        let _ = self.parse_identifier_or_string();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Statement::AgentContract { name, scope, goal, constraints, body, span })
            }
            TokenKind::Task => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut owner = None;
                    let mut status = None;
                    let mut requirement = None;
                    let mut implementation = None;
                    let mut skills = Vec::new();
                    let mut change_budget = Vec::new();
                    let mut evidence = Vec::new();
                    let mut body_stmts = Vec::new();

                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Evidence) {
                            self.advance();
                            self.expect(TokenKind::LBrace)?;
                            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                                let ek = self.parse_identifier_or_keyword()?;
                                self.match_token(&TokenKind::Colon);
                                let ev = self.parse_identifier_or_string()?;
                                evidence.push((ek, ev));
                                self.match_token(&TokenKind::Comma);
                                self.match_token(&TokenKind::SemiColon);
                            }
                            self.expect(TokenKind::RBrace)?;
                            continue;
                        }

                        let is_spec = self.check(&TokenKind::Ident("owner".to_string()))
                            || self.check(&TokenKind::Ident("status".to_string()))
                            || self.check(&TokenKind::Requirement)
                            || self.check(&TokenKind::Ident("implementation".to_string()))
                            || self.check(&TokenKind::Skills)
                            || self.check(&TokenKind::Ident("change_budget".to_string()));

                        if is_spec {
                            let key = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            if key == "owner" {
                                owner = Some(self.parse_identifier_or_string()?);
                            } else if key == "status" {
                                status = Some(self.parse_identifier_or_string()?);
                            } else if key == "requirement" || key == "requirements" {
                                requirement = Some(self.parse_identifier_or_string()?);
                            } else if key == "implementation" {
                                implementation = Some(self.parse_identifier_or_string()?);
                            } else if key == "skills" || key == "skill" {
                                skills = self.parse_string_list()?;
                            } else if key == "change_budget" {
                                change_budget = self.parse_string_list()?;
                            }
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        } else {
                            if let Ok(stmt) = self.parse_statement() {
                                body_stmts.push(stmt);
                            } else {
                                self.advance();
                            }
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    if owner.is_some() || status.is_some() || requirement.is_some() || !skills.is_empty() || !evidence.is_empty() {
                        Ok(Statement::AgentTaskContractDecl {
                            name, owner, status, requirement, implementation, skills, change_budget, evidence, span
                        })
                    } else {
                        Ok(Statement::TaskDecl { name, body: Block { statements: body_stmts, span: span.clone() }, span })
                    }
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::TaskDecl { name, body: Block { statements: Vec::new(), span: span.clone() }, span })
                }
            }
            TokenKind::Accept => {
                self.advance();
                let conditions = self.parse_string_list()?;
                Ok(Statement::AcceptBlock { conditions, span })
            }
            TokenKind::Reject => {
                self.advance();
                if self.check(&TokenKind::If) {
                    self.advance();
                }
                let conditions = self.parse_string_list()?;
                Ok(Statement::RejectBlock { conditions, span })
            }
            TokenKind::Baseline => {
                self.advance();
                let metrics = self.parse_key_value_pairs()?;
                Ok(Statement::BaselineBlock { metrics, span })
            }
            TokenKind::Regression => {
                self.advance();
                let condition = if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let cond = self.parse_identifier_or_string()?;
                    self.expect(TokenKind::RBrace)?;
                    cond
                } else {
                    let cond = self.parse_identifier_or_string()?;
                    self.match_token(&TokenKind::SemiColon);
                    cond
                };
                Ok(Statement::RegressionCheck { condition, span })
            }
            TokenKind::Explain => {
                self.advance();
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                    let op_name = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ExplainOpDecl { op_name, span });
                }
                let mut topic = "general".to_string();
                let mut rationale = String::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let s = self.parse_identifier_or_string()?;
                        if self.match_token(&TokenKind::Colon) {
                            topic = s;
                            rationale = self.parse_identifier_or_string()?;
                        } else {
                            rationale = s;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    rationale = self.parse_identifier_or_string()?;
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::ExplainBlock { topic, rationale, span })
            }
            TokenKind::Context => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut includes = Vec::new();
                let mut excludes = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    let list = self.parse_string_list()?;
                    if key == "include" || key == "includes" {
                        includes = list;
                    } else if key == "exclude" || key == "excludes" {
                        excludes = list;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Statement::ContextBlock { name, includes, excludes, body, span })
            }
            TokenKind::Slice => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut from_target = String::new();
                let mut includes = Vec::new();
                let mut excludes = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "from" {
                        from_target = self.parse_identifier_or_string()?;
                    } else if key == "include" || key == "includes" {
                        includes = self.parse_string_list()?;
                    } else if key == "exclude" || key == "excludes" {
                        excludes = self.parse_string_list()?;
                    } else {
                        let _ = self.parse_identifier_or_string();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::SliceDecl { name, from_target, includes, excludes, span })
            }
            TokenKind::Patch => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                let body = self.parse_block()?;
                Ok(Statement::PatchDecl { target, body, span })
            }
            TokenKind::Evolve => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                if target == "operation" {
                    let op_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::LBrace)?;
                    let mut preserve = Vec::new();
                    let mut optimize = Vec::new();
                    let mut allow = Vec::new();
                    let mut reject = Vec::new();

                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "preserve" {
                            let mut p = self.parse_string_list()?;
                            preserve.append(&mut p);
                        } else if key == "optimize" {
                            let mut opt = self.parse_string_list()?;
                            optimize.append(&mut opt);
                        } else if key == "allow" {
                            let mut a = self.parse_string_list()?;
                            allow.append(&mut a);
                        } else if key == "reject" || key == "reject_if" {
                            let mut r = self.parse_string_list()?;
                            reject.append(&mut r);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::EvolveOpDecl { op_name, preserve, optimize, allow, reject, span });
                }
                if target == "architecture" {
                    self.expect(TokenKind::LBrace)?;
                    let mut from = String::new();
                    let mut toward = String::new();
                    let mut target_modules = 25;
                    let mut preserve = Vec::new();
                    let mut optimize = Vec::new();
                    let mut reject_if = Vec::new();
                    let mut verify = Vec::new();

                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "from" {
                            from = self.parse_identifier_or_string()?;
                            if self.match_token(&TokenKind::Toward) {
                                toward = self.parse_identifier_or_string()?;
                            }
                        } else if key == "toward" {
                            toward = self.parse_identifier_or_string()?;
                        } else if key == "target" || key == "target_modules" || key == "modules" {
                            if let TokenKind::IntLit(i) = self.peek_kind() {
                                target_modules = *i as usize;
                                self.advance();
                            }
                        } else if key == "preserve" {
                            let mut p = self.parse_string_list()?;
                            preserve.append(&mut p);
                        } else if key == "optimize" {
                            let mut opt = self.parse_string_list()?;
                            optimize.append(&mut opt);
                        } else if key == "reject_if" || key == "reject" {
                            let mut r = self.parse_string_list()?;
                            reject_if.append(&mut r);
                        } else if key == "verify" {
                            let mut v = self.parse_string_list()?;
                            verify.append(&mut v);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Statement::EvolveArchDecl { from, toward, target_modules, preserve, optimize, reject_if, verify, span });
                }

                self.expect(TokenKind::LBrace)?;
                let mut intent = String::new();
                let mut preserve = Vec::new();
                let mut budget = None;
                let mut allow = Vec::new();
                let mut reject = Vec::new();
                let mut verify = Vec::new();
                let mut accept = Vec::new();

                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "intent" {
                        intent = self.parse_identifier_or_string()?;
                    } else if key == "preserve" {
                        preserve = self.parse_string_list()?;
                    } else if key == "budget" {
                        budget = Some(self.parse_identifier_or_string()?);
                    } else if key == "allow" {
                        allow = self.parse_string_list()?;
                    } else if key == "reject" {
                        reject = self.parse_string_list()?;
                    } else if key == "verify" {
                        verify = self.parse_string_list()?;
                    } else if key == "accept" {
                        accept = self.parse_string_list()?;
                    } else {
                        let _ = self.parse_string_list();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                let body = if self.check(&TokenKind::LBrace) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Statement::EvolveBlock { target, intent, preserve, budget, allow, reject, verify, accept, body, span })
            }

            TokenKind::Boundary => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut allows = Vec::new();
                let mut denies = Vec::new();
                let mut is_sealed = false;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        if key == "allow" {
                            allows.push(self.parse_identifier_or_string()?);
                        } else if key == "deny" {
                            denies.push(self.parse_identifier_or_string()?);
                        } else if key == "sealed" {
                            is_sealed = true;
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else if self.match_token(&TokenKind::Sealed) {
                    is_sealed = true;
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::BoundaryDecl { name, allows, denies, is_sealed, span })
            }
            TokenKind::Responsibility => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let description = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ResponsibilityDecl { module_name: "".to_string(), description, span })
            }
            TokenKind::Owns => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let symbols = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::OwnsDecl { module_name: "".to_string(), symbols, span })
            }
            TokenKind::Exposes => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let symbols = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ExposesDecl { module_name: "".to_string(), symbols, span })
            }
            TokenKind::DependsOnly => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let target_module = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?.join(", ")
                } else {
                    self.parse_identifier_or_string()?
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DependsDecl { from_module: "".to_string(), target_module, is_only: true, span })
            }
            TokenKind::Depends => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let target_module = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DependsDecl { from_module: "".to_string(), target_module, is_only: false, span })
            }
            TokenKind::Forbid => {
                self.advance();
                let from = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Arrow);
                let to = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ForbidDecl { from, to, span })
            }
            TokenKind::Layer => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut forbid_depends = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        if self.match_token(&TokenKind::Forbid) {
                            if self.match_token(&TokenKind::Depends) {
                                // consumed depends
                            }
                            forbid_depends.push(self.parse_identifier_or_keyword()?);
                        } else {
                            let _ = self.advance();
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::LayerDecl { name, forbid_depends, span })
            }
            TokenKind::Direction => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let from = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Arrow);
                let to = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DirectionDecl { from, to, span })
            }
            TokenKind::Split => {
                self.advance();
                let mut is_op = false;
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                    is_op = true;
                }
                let entity = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Into);
                let parts = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                if is_op {
                    Ok(Statement::SplitOpDecl { op_name: entity, sub_ops: parts, span })
                } else {
                    Ok(Statement::SplitDecl { entity, parts, span })
                }
            }
            TokenKind::Partition => {
                self.advance();
                let entity = self.parse_identifier_or_keyword()?;
                let mut by = "responsibility".to_string();
                if self.match_token(&TokenKind::By) {
                    by = self.parse_identifier_or_keyword()?;
                }
                let parts = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::PartitionDecl { entity, by, parts, span })
            }
            TokenKind::Extract => {
                self.advance();
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                    let op_name = self.parse_identifier_or_keyword()?;
                    let mut from_mod = String::new();
                    let mut condition = String::new();
                    if self.match_token(&TokenKind::From) {
                        from_mod = self.parse_identifier_or_keyword()?;
                    }
                    if self.peek_kind() == &TokenKind::Ident("where".to_string()) || self.match_token(&TokenKind::When) {
                        if self.peek_kind() == &TokenKind::Ident("where".to_string()) {
                            self.advance();
                        }
                        condition = self.parse_identifier_or_string()?;
                        if self.match_token(&TokenKind::EqualEqual) || self.match_token(&TokenKind::Equal) {
                            condition.push_str(" == ");
                            condition.push_str(&self.parse_identifier_or_string()?);
                        }
                    }
                    self.match_token(&TokenKind::SemiColon);
                    return Ok(Statement::ExtractOpDecl { op_name, from_mod, condition, span });
                }
                self.expect(TokenKind::LBrace)?;
                let mut symbols = Vec::new();
                let mut into_module = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "symbols" || key == "symbol" {
                        symbols = self.parse_string_list()?;
                    } else if key == "into" {
                        into_module = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ExtractDecl { symbols, into_module, span })
            }
            TokenKind::Cluster => {
                self.advance();
                let mut by = "semantic".to_string();
                if self.match_token(&TokenKind::By) {
                    by = self.parse_identifier_or_keyword()?;
                }
                self.expect(TokenKind::LBrace)?;
                let mut predicate = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let s = self.parse_identifier_or_string()?;
                    if !predicate.is_empty() { predicate.push(' '); }
                    predicate.push_str(&s);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ClusterDecl { by, predicate, span })
            }
            TokenKind::Separate => {
                self.advance();
                let left = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::From);
                let right = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::SeparateDecl { left, right, span })
            }
            TokenKind::Merge => {
                self.advance();
                let source_ops = self.parse_string_list()?;
                let mut as_name = String::new();
                if self.match_token(&TokenKind::As) {
                    as_name = self.parse_identifier_or_keyword()?;
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::MergeOpDecl { source_ops, as_name, span })
            }
            TokenKind::Inline => {
                self.advance();
                if self.match_token(&TokenKind::Operation) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "operation" } else { false }) {
                    if self.peek_kind() == &TokenKind::Ident("operation".to_string()) {
                        self.advance();
                    }
                }
                let op_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::InlineOpDecl { op_name, span })
            }
            TokenKind::Contract => {
                self.advance();
                let mut module_name = String::new();
                while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::EOF) {
                    let part = self.parse_identifier_or_keyword()?;
                    if part != "Module" && part != "mod" {
                        module_name = part;
                    }
                }
                self.expect(TokenKind::LBrace)?;
                let mut accepts = Vec::new();
                let mut returns = Vec::new();
                let mut guarantees = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "accepts" || key == "accept" {
                        accepts = self.parse_string_list()?;
                    } else if key == "returns" || key == "return" {
                        returns = self.parse_string_list()?;
                    } else if key == "guarantees" || key == "guarantee" {
                        guarantees = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ModuleContractDecl { module_name, accepts, returns, guarantees, span })
            }
            TokenKind::Port => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut methods = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let m = self.parse_identifier_or_keyword()?;
                    if self.match_token(&TokenKind::LParen) {
                        self.match_token(&TokenKind::RParen);
                    }
                    methods.push(m);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::PortDecl { name, methods, span })
            }
            TokenKind::Adapter => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut port = String::new();
                if self.match_token(&TokenKind::For) || self.match_token(&TokenKind::Impl) || self.peek_kind() != &TokenKind::LBrace {
                    port = self.parse_identifier_or_keyword().unwrap_or_default();
                }
                let body = self.parse_block()?;
                Ok(Statement::AdapterDecl { name, port, body, span })
            }
            TokenKind::Facade => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut exposes = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "expose" || key == "exposes" {
                        if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                            let mut list = self.parse_string_list()?;
                            exposes.append(&mut list);
                        } else {
                            exposes.push(self.parse_identifier_or_string()?);
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::FacadeDecl { name, exposes, span })
            }
            TokenKind::Gateway => {
                self.advance();
                let from_mod = self.parse_identifier_or_keyword()?;
                if self.match_token(&TokenKind::From) {
                    let f = self.parse_identifier_or_keyword()?;
                    let _ = f;
                }
                self.match_token(&TokenKind::To);
                self.match_token(&TokenKind::Arrow);
                let to_mod = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut allowed_calls = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "allow" || key == "allows" {
                        allowed_calls = self.parse_string_list()?;
                    } else {
                        allowed_calls.push(key);
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::GatewayDecl { from_mod, to_mod, allowed_calls, span })
            }
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
                let api_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::StableDecl { api_name, span })
            }
            TokenKind::Sealed => {
                self.advance();
                let mut boundary_name = String::new();
                while !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                    let part = self.parse_identifier_or_keyword()?;
                    if part != "Boundary" && part != "boundary" {
                        boundary_name = part;
                        break;
                    }
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::SealedDecl { boundary_name, span })
            }
            TokenKind::Friend => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::To);
                let friend_module = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::FriendDecl { module_name, friend_module, span })
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
            TokenKind::CycleFree => {
                self.advance();
                self.match_token(&TokenKind::Equal);
                self.match_token(&TokenKind::Colon);
                let scope = if self.match_token(&TokenKind::True) || self.match_token(&TokenKind::False) {
                    "modules".to_string()
                } else {
                    self.parse_identifier_or_keyword().unwrap_or_else(|_| "modules".to_string())
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::CycleFreeDecl { scope, span })
            }
            TokenKind::MaxFanout => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::Colon);
                let mut limit = 5;
                if let TokenKind::IntLit(i) = self.peek_kind() {
                    limit = *i as usize;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::FanoutDecl { module_name, limit, span })
            }
            TokenKind::MaxFanin => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::Colon);
                let mut limit = 20;
                if let TokenKind::IntLit(i) = self.peek_kind() {
                    limit = *i as usize;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::FaninDecl { module_name, limit, span })
            }
            TokenKind::MaxDepth => {
                self.advance();
                self.match_token(&TokenKind::Colon);
                let mut limit = 6;
                if let TokenKind::IntLit(i) = self.peek_kind() {
                    limit = *i as usize;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::DepthDecl { limit, span })
            }
            TokenKind::Cohesion => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::GreaterEqual);
                self.match_token(&TokenKind::Equal);
                self.match_token(&TokenKind::Colon);
                let mut min_threshold = 0.8;
                if let TokenKind::FloatLit(f) = self.peek_kind() {
                    min_threshold = *f;
                    self.advance();
                } else if let TokenKind::IntLit(i) = self.peek_kind() {
                    min_threshold = *i as f64;
                    self.advance();
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::CohesionDecl { module_name, min_threshold, span })
            }
            TokenKind::Modularize => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut target_files_min = 5;
                let mut target_files_max = 20;
                let mut preserve = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "target_files" || key == "target" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            target_files_min = *i as usize;
                            target_files_max = *i as usize;
                            self.advance();
                            if self.match_token(&TokenKind::Dot) && self.match_token(&TokenKind::Dot) {
                                if let TokenKind::IntLit(i2) = self.peek_kind() {
                                    target_files_max = *i2 as usize;
                                    self.advance();
                                }
                            }
                        }
                    } else if key == "preserve" {
                        let mut p = self.parse_string_list()?;
                        preserve.append(&mut p);
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ModularizeDecl { target, target_files_min, target_files_max, preserve, span })
            }
            TokenKind::Decompose => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut target_modules = None;
                let mut optimize = Vec::new();
                let mut preserve = Vec::new();
                let mut gravity = None;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "target" || key == "target_modules" || key == "modules" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            target_modules = Some(*i as usize);
                            self.advance();
                            let _ = self.parse_identifier_or_keyword().ok();
                        }
                    } else if key == "optimize" {
                        let mut opt = self.parse_string_list()?;
                        optimize.append(&mut opt);
                    } else if key == "preserve" {
                        let mut p = self.parse_string_list()?;
                        preserve.append(&mut p);
                    } else if key == "gravity" {
                        gravity = Some(self.parse_identifier_or_string()?);
                    } else {
                        let _ = self.parse_string_list();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::DecomposeDecl { target, target_modules, optimize, preserve, gravity, span })
            }
            TokenKind::Architecture => {
                self.advance();
                let name = self.parse_identifier_or_keyword().unwrap_or_else(|_| "system".to_string());
                self.expect(TokenKind::LBrace)?;
                let mut layers = Vec::new();
                let mut rules = Vec::new();
                let mut directions = Vec::new();
                let mut invariants = Vec::new();
                let mut cycle_free = false;
                let mut max_depth = None;

                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    if key == "layers" || key == "layer" {
                        self.match_token(&TokenKind::Colon);
                        layers = self.parse_string_list()?;
                    } else if key == "rules" || key == "rule" {
                        self.match_token(&TokenKind::Colon);
                        rules = self.parse_string_list()?;
                    } else if key == "direction" || key == "directions" {
                        self.match_token(&TokenKind::Colon);
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) && !self.check(&TokenKind::Layer) && !self.check(&TokenKind::Ident("rules".to_string())) {
                            if let Ok(from) = self.parse_identifier_or_keyword() {
                                self.match_token(&TokenKind::Arrow);
                                if let Ok(to) = self.parse_identifier_or_keyword() {
                                    directions.push((from, to));
                                }
                            } else {
                                break;
                            }
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                    } else if key == "invariant" || key == "invariants" {
                        self.match_token(&TokenKind::Colon);
                        invariants = self.parse_string_list()?;
                    } else if key == "cycle_free" {
                        cycle_free = true;
                        self.match_token(&TokenKind::Colon);
                        self.match_token(&TokenKind::Equal);
                        if self.match_token(&TokenKind::True) || self.match_token(&TokenKind::False) {
                            // consumed
                        } else {
                            let _ = self.parse_identifier_or_keyword().ok();
                        }
                    } else if key == "max_dependency_depth" || key == "max_depth" || key == "depth" {
                        self.match_token(&TokenKind::Colon);
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            max_depth = Some(*i as usize);
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ArchitectureDecl { name, layers, rules, directions, invariants, cycle_free, max_depth, span })
            }
            TokenKind::Repair => {
                self.advance();
                let target = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::RepairDecl { target, span })
            }
            TokenKind::Gravity => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut weights = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let k = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    let mut w = 1.0;
                    if let TokenKind::IntLit(i) = self.peek_kind() {
                        w = *i as f64 / 100.0;
                        self.advance();
                        self.match_token(&TokenKind::Percent);
                    } else if let TokenKind::FloatLit(f) = self.peek_kind() {
                        w = *f;
                        self.advance();
                        self.match_token(&TokenKind::Percent);
                    }
                    weights.push((k, w));
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::GravityDecl { weights, span })
            }

            TokenKind::Bridge | TokenKind::Ident(_) if self.peek_kind() == &TokenKind::Bridge || (if let TokenKind::Ident(id) = self.peek_kind() { id == "bridge" } else { false }) => {
                self.advance();
                let from_mod = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Arrow);
                let to_mod = self.parse_identifier_or_keyword()?;
                let mut via = String::new();
                if self.peek_kind() == &TokenKind::Ident("via".to_string()) {
                    self.advance();
                    via = self.parse_identifier_or_keyword()?;
                }
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::BridgeDecl { from_mod, to_mod, via, span })
            }
            TokenKind::Analyze => {
                self.advance();
                let op_expr = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::AnalyzeOp { op_expr, span })
            }
            TokenKind::Feature => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut requirement = None;
                let mut skills = Vec::new();
                let mut tasks = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "requirement" || key == "requirements" {
                        requirement = Some(self.parse_identifier_or_string()?);
                    } else if key == "skills" || key == "skill" {
                        skills = self.parse_string_list()?;
                    } else if key == "tasks" || key == "task" {
                        tasks = self.parse_string_list()?;
                    } else {
                        let _ = self.parse_string_list();
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::FeatureDecl { name, requirement, skills, tasks, span })
            }
            TokenKind::Skill => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                let mut for_scope = None;
                if self.peek_kind() == &TokenKind::Ident("for".to_string()) || self.peek_kind() == &TokenKind::For {
                    self.advance();
                    for_scope = Some(self.parse_identifier_or_keyword()?);
                }
                let mut rules = Vec::new();
                let mut constraints = Vec::new();
                let mut structural = Vec::new();
                let mut semantic = Vec::new();
                let mut behavioral = Vec::new();
                let mut architectural = Vec::new();
                let mut performance = Vec::new();
                let mut security = Vec::new();
                let mut testing = Vec::new();
                let mut agent = Vec::new();
                let mut requires = Vec::new();
                let mut hard = Vec::new();
                let mut soft = Vec::new();
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        let list = self.parse_string_list()?;
                        if key == "rules" || key == "rule" {
                            rules.extend(list);
                        } else if key == "constraints" || key == "constraint" {
                            constraints.extend(list);
                        } else if key == "structural" {
                            structural.extend(list);
                        } else if key == "semantic" {
                            semantic.extend(list);
                        } else if key == "behavioral" {
                            behavioral.extend(list);
                        } else if key == "architectural" {
                            architectural.extend(list);
                        } else if key == "performance" {
                            performance.extend(list);
                        } else if key == "security" {
                            security.extend(list);
                        } else if key == "testing" {
                            testing.extend(list);
                        } else if key == "agent" {
                            agent.extend(list);
                        } else if key == "requires" || key == "require" {
                            requires.extend(list);
                        } else if key == "hard" {
                            hard.extend(list);
                        } else if key == "soft" {
                            soft.extend(list);
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::SkillDecl {
                    name, rules, constraints, structural, semantic, behavioral, architectural,
                    performance, security, testing, agent, requires, hard, soft, for_scope, span
                })
            }
            TokenKind::Satisfies => {
                self.advance();
                let entity = self.parse_identifier_or_keyword()?;
                let skills = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::SatisfiesDecl { entity, skills, span })
            }
            TokenKind::Requirement => {
                self.advance();
                let req_id = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::LBrace);
                let description = self.parse_identifier_or_string()?;
                self.match_token(&TokenKind::RBrace);
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::RequirementDecl { req_id, description, span })
            }
            TokenKind::Implements => {
                self.advance();
                let req_id = self.parse_identifier_or_keyword()?;
                let entities = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ImplementsDecl { req_id, entities, span })
            }
            TokenKind::Verifies => {
                self.advance();
                let req_id = self.parse_identifier_or_keyword()?;
                let entities = self.parse_string_list()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::VerifiesDecl { req_id, entities, span })
            }
            TokenKind::Claim => {
                self.advance();
                if self.check(&TokenKind::Task) {
                    self.advance();
                }
                let task_name = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ClaimTask { task_name, span })
            }
            TokenKind::Complete => {
                self.advance();
                if self.check(&TokenKind::Task) {
                    self.advance();
                }
                let task_name = self.parse_identifier_or_keyword()?;
                let mut result = "success".to_string();
                let mut confidence = None;
                let mut summary = None;
                let mut evidence = Vec::new();
                let mut risks = None;
                let mut recommendation = None;
                let mut notes = None;
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let key = self.parse_identifier_or_keyword()?;
                        self.match_token(&TokenKind::Colon);
                        if key == "result" {
                            result = self.parse_identifier_or_string()?;
                        } else if key == "confidence" {
                            if let TokenKind::FloatLit(f) = self.peek_kind() {
                                confidence = Some(*f);
                                self.advance();
                            } else if let TokenKind::IntLit(i) = self.peek_kind() {
                                confidence = Some(*i as f64);
                                self.advance();
                            }
                        } else if key == "summary" {
                            summary = Some(self.parse_identifier_or_string()?);
                        } else if key == "evidence" {
                            evidence = self.parse_string_list()?;
                        } else if key == "risks" || key == "risk" {
                            risks = Some(self.parse_identifier_or_string()?);
                        } else if key == "recommendation" {
                            recommendation = Some(self.parse_identifier_or_string()?);
                        } else if key == "notes" || key == "note" {
                            notes = Some(self.parse_identifier_or_string()?);
                        } else {
                            let _ = self.parse_string_list();
                        }
                        self.match_token(&TokenKind::Comma);
                        self.match_token(&TokenKind::SemiColon);
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    self.match_token(&TokenKind::SemiColon);
                }
                Ok(Statement::CompleteTask {
                    task_name, result, confidence, summary, evidence, risks, recommendation, notes, span
                })
            }
            TokenKind::Todo => {
                self.advance();
                let id = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut implement = String::new();
                let mut requires = Vec::new();
                let mut verify = Vec::new();
                let mut status = "planned".to_string();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "implement" {
                        implement = self.parse_identifier_or_string()?;
                    } else if key == "requires" || key == "require" {
                        requires = self.parse_string_list()?;
                    } else if key == "verify" {
                        verify = self.parse_string_list()?;
                    } else if key == "status" {
                        status = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::TodoDecl { id, implement, requires, verify, status, span })
            }
            TokenKind::Change => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut task = String::new();
                let mut intent = String::new();
                let mut satisfies = Vec::new();
                let mut evidence = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "task" {
                        task = self.parse_identifier_or_string()?;
                    } else if key == "intent" {
                        intent = self.parse_identifier_or_string()?;
                    } else if key == "satisfies" {
                        satisfies = self.parse_string_list()?;
                    } else if key == "evidence" {
                        evidence = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::SemanticCommitDecl { task, intent, satisfies, evidence, span })
            }
            TokenKind::Review => {
                self.advance();
                let task_id = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut summary = String::new();
                let mut completed = 0;
                let mut unresolved = 0;
                let mut risks = 0;
                let mut confidence = 1.0;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "summary" {
                        summary = self.parse_identifier_or_string()?;
                    } else if key == "completed" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            completed = *i as usize;
                            self.advance();
                        }
                    } else if key == "unresolved" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            unresolved = *i as usize;
                            self.advance();
                        }
                    } else if key == "risks" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            risks = *i as usize;
                            self.advance();
                        }
                    } else if key == "confidence" {
                        if let TokenKind::FloatLit(f) = self.peek_kind() {
                            confidence = *f;
                            self.advance();
                        } else if let TokenKind::IntLit(i) = self.peek_kind() {
                            confidence = *i as f64;
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentReviewDecl { task_id, summary, completed, unresolved, risks, confidence, span })
            }
            TokenKind::Approval => {
                self.advance();
                if self.peek_kind() == &TokenKind::Ident("required".to_string()) {
                    self.advance();
                }
                let required_items = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::ApprovalDecl { required_items, span })
            }
            TokenKind::Knowledge => {
                self.advance();
                let name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut decisions = Vec::new();
                let mut constraints = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "decisions" || key == "decision" {
                        decisions = self.parse_string_list()?;
                    } else if key == "constraints" || key == "constraint" {
                        constraints = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::KnowledgeDecl { name, decisions, constraints, span })
            }
            TokenKind::Decision => {
                self.advance();
                let id = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut choose = String::new();
                let mut because = String::new();
                let mut reject = String::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "choose" {
                        choose = self.parse_identifier_or_string()?;
                    } else if key == "because" {
                        because = self.parse_identifier_or_string()?;
                    } else if key == "reject" {
                        reject = self.parse_identifier_or_string()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::DecisionDecl { id, choose, because, reject, span })
            }
            TokenKind::AgentBoundary => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword().unwrap_or_default();
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::AgentBoundaryDecl { module_name, span })
            }
            TokenKind::AgentContext => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut expose = Vec::new();
                let mut hide = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "expose" {
                        expose = self.parse_string_list()?;
                    } else if key == "hide" {
                        hide = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentContextDecl { module_name, expose, hide, span })
            }
            TokenKind::ContextFirewall => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut deny = Vec::new();
                let mut expose = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "deny" {
                        deny = self.parse_string_list()?;
                    } else if key == "expose" {
                        expose = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ContextFirewallDecl { module_name, deny, expose, span })
            }
            TokenKind::AgentApi => {
                self.advance();
                let module_name = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::LBrace)?;
                let mut expose = Vec::new();
                let mut hide = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "expose" {
                        expose = self.parse_string_list()?;
                    } else if key == "hide" {
                        hide = self.parse_string_list()?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentApiDecl { module_name, expose, hide, span })
            }
            TokenKind::Agentability => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut max_context_tokens = 12000;
                let mut max_operation_complexity = "medium".to_string();
                let mut max_dependency_fanout = 8;
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let key = self.parse_identifier_or_keyword()?;
                    self.match_token(&TokenKind::Colon);
                    if key == "max_context_tokens" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            max_context_tokens = *i as usize;
                            self.advance();
                        }
                    } else if key == "max_operation_complexity" {
                        max_operation_complexity = self.parse_identifier_or_string()?;
                    } else if key == "max_dependency_fanout" {
                        if let TokenKind::IntLit(i) = self.peek_kind() {
                            max_dependency_fanout = *i as usize;
                            self.advance();
                        }
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::AgentabilityDecl { max_context_tokens, max_operation_complexity, max_dependency_fanout, span })
            }
            TokenKind::RegressionGuard => {
                self.advance();
                let items = if self.check(&TokenKind::LBrace) || self.check(&TokenKind::LBracket) {
                    self.parse_string_list()?
                } else {
                    vec![self.parse_identifier_or_string()?]
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::RegressionGuardDecl { items, span })
            }
            TokenKind::Ident(id) if id == "project" => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut profile = std::collections::HashMap::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    let k = self.parse_identifier_or_keyword()?;
                    if k == "skills" || k == "skill" {
                        self.expect(TokenKind::LBrace)?;
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let sk = self.parse_identifier_or_keyword()?;
                            self.match_token(&TokenKind::Colon);
                            let sv = self.parse_identifier_or_string()?;
                            profile.insert(sk, sv);
                            self.match_token(&TokenKind::Comma);
                            self.match_token(&TokenKind::SemiColon);
                        }
                        self.expect(TokenKind::RBrace)?;
                    }
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::ProjectSkillsDecl { profile, span })
            }


            _ => {
                let expr = self.parse_expression()?;
                if self.match_token(&TokenKind::Equal) {
                    let value = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::Assignment {
                        target: expr,
                        value,
                        span,
                    })
                } else if self.match_token(&TokenKind::LessPlusEqual) {
                    let value = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    if let Expression::Ident(target_name, _) = &expr {
                        Ok(Statement::AtomicOp {
                            target: target_name.clone(),
                            op: BinaryOp::Add,
                            value,
                            span,
                        })
                    } else {
                        Ok(Statement::Assignment {
                            target: expr.clone(),
                            value: Expression::Binary {
                                left: Box::new(expr),
                                op: BinaryOp::Add,
                                right: Box::new(value),
                                span: span.clone(),
                            },
                            span,
                        })
                    }
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::Expression(expr))
                }
            }
        }
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        let span = self.current_span();
        let pattern = self.parse_pattern()?;

        let mut guard = None;
        if self.match_token(&TokenKind::If) {
            guard = Some(self.parse_expression()?);
        }

        self.expect(TokenKind::FatArrow)?;

        let body = if self.check(&TokenKind::LBrace) {
            self.parse_block()?
        } else {
            let stmt = self.parse_statement()?;
            Block {
                statements: vec![stmt],
                span: span.clone(),
            }
        };

        Ok(MatchArm {
            pattern,
            guard,
            body,
            span,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        if self.match_token(&TokenKind::Underscore) {
            return Ok(Pattern::Wildcard);
        }

        if self.match_token(&TokenKind::Dot) {
            let variant_name = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected variant name after '.', found {:?}", other)),
            };

            let mut binding = None;
            if self.match_token(&TokenKind::LParen) {
                if let TokenKind::Ident(b) = self.advance().kind {
                    binding = Some(b);
                }
                self.expect(TokenKind::RParen)?;
            }

            return Ok(Pattern::Variant {
                enum_name: None,
                variant_name,
                binding,
            });
        }

        match self.peek_kind() {
            TokenKind::IntLit(n) => {
                let val = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(val)))
            }
            TokenKind::StringLit(s) => {
                let val = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(val)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            TokenKind::Ident(name) => {
                let id = name.clone();
                self.advance();
                let is_enum_variant = if self.match_token(&TokenKind::Dot) {
                    true
                } else if self.check(&TokenKind::Colon) {
                    self.advance();
                    self.match_token(&TokenKind::Colon)
                } else {
                    false
                };

                if is_enum_variant {
                    let vname = match self.advance().kind {
                        TokenKind::Ident(n) => n,
                        other => return Err(format!("Expected variant name, found {:?}", other)),
                    };
                    let mut binding = None;
                    if self.match_token(&TokenKind::LParen) {
                        if let TokenKind::Ident(b) = self.advance().kind {
                            binding = Some(b);
                        }
                        self.expect(TokenKind::RParen)?;
                    }
                    Ok(Pattern::Variant {
                        enum_name: Some(id),
                        variant_name: vname,
                        binding,
                    })
                } else {
                    Ok(Pattern::Ident(id))
                }
            }
            other => Err(format!("Invalid pattern syntax: {:?} at line {}", other, self.current_span().line)),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_pipe_expr()
    }

    fn parse_pipe_expr(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_catch_expr()?;
        while self.check(&TokenKind::PipeGreater)
            || self.check(&TokenKind::TildeArrow)
            || self.check(&TokenKind::Shr)
            || self.check(&TokenKind::Fallback)
            || self.check(&TokenKind::When)
            || (self.check(&TokenKind::Question) && !self.check(&TokenKind::QuestionQuestion))
        {
            if self.match_token(&TokenKind::PipeGreater) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::Pipe {
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                    span,
                };
            } else if self.match_token(&TokenKind::TildeArrow) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::NullCollapse {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    span,
                };
            } else if self.match_token(&TokenKind::Shr) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::Compose {
                    ops: vec![expr, rhs],
                    span,
                };
            } else if self.match_token(&TokenKind::Fallback) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::Alternative {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    span,
                };
            } else if self.match_token(&TokenKind::When) || self.match_token(&TokenKind::Question) {
                let span = self.current_span();
                let cond = self.parse_catch_expr()?;
                expr = Expression::ConditionalOp {
                    op: Box::new(expr),
                    condition: Box::new(cond),
                    span,
                };
            }
        }
        Ok(expr)
    }

    fn parse_catch_expr(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_logical_or()?;

        if self.match_token(&TokenKind::Catch) {
            let span = self.current_span();
            let mut err_name = "err".to_string();
            if let TokenKind::Ident(n) = self.peek_kind() {
                if n != "return" && n != "ret" {
                    err_name = n.clone();
                    self.advance();
                }
            }

            let handler = if self.check(&TokenKind::Return) {
                let stmt = self.parse_statement()?;
                Box::new(stmt)
            } else if self.check(&TokenKind::LBrace) {
                let blk = self.parse_block()?;
                Box::new(Statement::Expression(Expression::Block(blk)))
            } else {
                let sub_expr = self.parse_expression()?;
                Box::new(Statement::Expression(sub_expr))
            };

            expr = Expression::Catch {
                expr: Box::new(expr),
                error_name: err_name,
                handler,
                span,
            };
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_logical_and()?;
        while self.match_token(&TokenKind::PipePipe) {
            let span = self.current_span();
            let right = self.parse_logical_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_bitwise_or()?;
        while self.match_token(&TokenKind::AmpAmp) {
            let span = self.current_span();
            let right = self.parse_bitwise_or()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_bitwise_xor()?;
        while self.match_token(&TokenKind::Pipe) {
            let span = self.current_span();
            let right = self.parse_bitwise_xor()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::BitOr,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_bitwise_and()?;
        while self.match_token(&TokenKind::Caret) {
            let span = self.current_span();
            let right = self.parse_bitwise_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::BitXor,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_equality()?;
        while self.match_token(&TokenKind::Ampersand) {
            let span = self.current_span();
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::BitAnd,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;
        while self.check(&TokenKind::EqualEqual) || self.check(&TokenKind::BangEqual) {
            let op = if self.match_token(&TokenKind::EqualEqual) {
                BinaryOp::Equal
            } else {
                self.advance();
                BinaryOp::NotEqual
            };
            let span = self.current_span();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_shift()?;
        while self.check(&TokenKind::Less)
            || self.check(&TokenKind::LessEqual)
            || self.check(&TokenKind::Greater)
            || self.check(&TokenKind::GreaterEqual)
        {
            let op = if self.match_token(&TokenKind::Less) {
                BinaryOp::LessThan
            } else if self.match_token(&TokenKind::LessEqual) {
                BinaryOp::LessEqual
            } else if self.match_token(&TokenKind::Greater) {
                BinaryOp::GreaterThan
            } else {
                self.advance();
                BinaryOp::GreaterEqual
            };
            let span = self.current_span();
            let right = self.parse_shift()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_addition()?;
        while self.check(&TokenKind::Shl) || self.check(&TokenKind::Shr) {
            let op = if self.match_token(&TokenKind::Shl) {
                BinaryOp::Shl
            } else {
                self.advance();
                BinaryOp::Shr
            };
            let span = self.current_span();
            let right = self.parse_addition()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;
        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let op = if self.match_token(&TokenKind::Plus) {
                BinaryOp::Add
            } else {
                self.advance();
                BinaryOp::Sub
            };
            let span = self.current_span();
            let right = self.parse_multiplication()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;
        while self.check(&TokenKind::Star) || self.check(&TokenKind::Slash) || self.check(&TokenKind::Percent) {
            let op = if self.match_token(&TokenKind::Star) {
                BinaryOp::Mul
            } else if self.match_token(&TokenKind::Slash) {
                BinaryOp::Div
            } else {
                self.advance();
                BinaryOp::Mod
            };
            let span = self.current_span();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        let span = self.current_span();
        if self.match_token(&TokenKind::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Bang) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Tilde) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Ampersand) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::AddressOf,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Star) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Deref,
                expr: Box::new(expr),
                span,
            });
        }

        let mut expr = self.parse_postfix()?;
        while self.match_token(&TokenKind::As) {
            let span = self.current_span();
            let target_type = self.parse_type()?;
            expr = Expression::Cast {
                expr: Box::new(expr),
                target_type,
                span,
            };
        }
        Ok(expr)
    }

    fn parse_postfix(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            let span = self.current_span();
            if self.match_token(&TokenKind::Dot) {
                let member_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected member identifier after '.', found {:?}", other)),
                };
                expr = Expression::FieldAccess {
                    object: Box::new(expr),
                    field: member_name,
                    span,
                };
            } else if self.match_token(&TokenKind::LParen) {
                let mut args = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    args.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                expr = Expression::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            } else if self.match_token(&TokenKind::LBracket) {
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket)?;
                expr = Expression::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let span = self.current_span();

        if self.match_token(&TokenKind::Match) {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::LBrace)?;
            let mut arms = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let arm_span = self.current_span();
                let pattern = self.parse_pattern()?;
                let mut guard = None;
                if self.match_token(&TokenKind::If) {
                    guard = Some(self.parse_expression()?);
                }
                self.expect(TokenKind::FatArrow)?;
                let body = if self.check(&TokenKind::LBrace) {
                    self.parse_block()?
                } else {
                    let expr = self.parse_expression()?;
                    self.match_token(&TokenKind::Comma);
                    Block {
                        statements: vec![Statement::Expression(expr)],
                        span: arm_span.clone(),
                    }
                };
                self.match_token(&TokenKind::Comma);
                arms.push(MatchArm {
                    pattern,
                    guard,
                    body,
                    span: arm_span,
                });
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Expression::Match {
                expr: Box::new(expr),
                arms,
                span,
            });
        }

        if self.match_token(&TokenKind::Dot) {
            let vname = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected variant name after '.', found {:?}", other)),
            };

            let mut payload = None;
            if self.match_token(&TokenKind::LParen) {
                payload = Some(Box::new(self.parse_expression()?));
                self.expect(TokenKind::RParen)?;
            }

            return Ok(Expression::EnumInit {
                enum_name: None,
                variant_name: vname,
                payload,
                span,
            });
        }

        match self.peek_kind() {
            TokenKind::Operation => {
                let op = self.parse_operation(false)?;
                return Ok(Expression::OperationLiteral {
                    name: if op.name.is_empty() { None } else { Some(op.name) },
                    params: op.params,
                    return_type: op.return_type,
                    requires: op.requires,
                    guarantees: op.guarantees,
                    effects: op.effects,
                    emits: op.emits,
                    body: op.body,
                    span,
                });
            }
            TokenKind::Compose => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut ops = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    if self.match_token(&TokenKind::Then) {
                        continue;
                    }
                    ops.push(self.parse_expression()?);
                    self.match_token(&TokenKind::Then);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                return Ok(Expression::Compose { ops, span });
            }
            TokenKind::Retry => {
                self.advance();
                let op = self.parse_primary()?;
                let mut count = Expression::Lit(Literal::Int(3), span.clone());
                if self.peek_kind() == &TokenKind::Ident("up".to_string()) {
                    self.advance();
                    if self.peek_kind() == &TokenKind::To {
                        self.advance();
                    }
                }
                if let TokenKind::IntLit(_) = self.peek_kind() {
                    count = self.parse_primary()?;
                }
                return Ok(Expression::Repeat {
                    op: Box::new(op),
                    count: Box::new(count),
                    is_retry: true,
                    span,
                });
            }
            TokenKind::Repeat => {
                self.advance();
                let op = self.parse_primary()?;
                let mut count = Expression::Lit(Literal::Int(1), span.clone());
                if let TokenKind::IntLit(_) = self.peek_kind() {
                    count = self.parse_primary()?;
                }
                return Ok(Expression::Repeat {
                    op: Box::new(op),
                    count: Box::new(count),
                    is_retry: false,
                    span,
                });
            }
            TokenKind::Memoize => {
                self.advance();
                let op = self.parse_primary()?;
                return Ok(Expression::Memoize {
                    op: Box::new(op),
                    span,
                });
            }
            TokenKind::NameOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let mut target_name = String::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::Ident(n) => target_name.push_str(&n),
                        TokenKind::Dot => target_name.push('.'),
                        _ => {}
                    }
                }
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::NameOf { target: target_name, span });
            }
            TokenKind::PathOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let mut target_name = String::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::Ident(n) => target_name.push_str(&n),
                        TokenKind::Dot => target_name.push('.'),
                        _ => {}
                    }
                }
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::PathOf { target: target_name, span });
            }
            TokenKind::TypeOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::TypeOf { expr: Box::new(expr), span });
            }
            TokenKind::DocOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let target = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected identifier in docof!, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::DocOf { target, span });
            }
            TokenKind::CodeOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                let raw_code = "expr_source".to_string();
                return Ok(Expression::CodeOf { expr: Box::new(expr), code: raw_code, span });
            }
            TokenKind::Dbg => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::Dbg { expr: Box::new(expr), code: "dbg_expr".to_string(), span });
            }
            TokenKind::AssertDebug => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let cond = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::AssertDebug { condition: Box::new(cond), code: "assert_cond".to_string(), span });
            }
            TokenKind::Translate => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let key = match self.advance().kind {
                    TokenKind::StringLit(s) => s,
                    other => return Err(format!("Expected string key in t!, found {:?}", other)),
                };
                let mut args = Vec::new();
                while self.match_token(&TokenKind::Comma) {
                    if self.check(&TokenKind::RParen) {
                        break;
                    }
                    let arg_name = match self.advance().kind {
                        TokenKind::Ident(n) => n,
                        other => return Err(format!("Expected argument name in t!, found {:?}", other)),
                    };
                    self.expect(TokenKind::Equal)?;
                    let arg_val = self.parse_expression()?;
                    args.push((arg_name, arg_val));
                }
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::Translate { key, args, span });
            }
            TokenKind::FieldsOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let target = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected struct identifier in fields_of!, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::FieldsOf { target, span });
            }
            TokenKind::SqlExpr => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::SqlExpr { expr: Box::new(expr), span });
            }
            TokenKind::IntLit(n) => {
                let val = *n;
                self.advance();
                Ok(Expression::Lit(Literal::Int(val), span))
            }
            TokenKind::FloatLit(f) => {
                let val = *f;
                self.advance();
                Ok(Expression::Lit(Literal::Float(val), span))
            }
            TokenKind::UnitLit(val, unit) => {
                let v = *val;
                let u = unit.clone();
                self.advance();
                Ok(Expression::UnitLit { value: v, unit: u, span })
            }
            TokenKind::StringLit(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expression::Lit(Literal::String(val), span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Lit(Literal::Bool(true), span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Lit(Literal::Bool(false), span))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Lit(Literal::Null, span))
            }
            TokenKind::Struct => {
                self.advance();
                Ok(Expression::Ident("st".to_string(), span))
            }
            TokenKind::Val => {
                self.advance();
                Ok(Expression::Ident("val".to_string(), span))
            }
            TokenKind::Mut => {
                self.advance();
                Ok(Expression::Ident("mut".to_string(), span))
            }
            TokenKind::Target => {
                self.advance();
                Ok(Expression::Ident("target".to_string(), span))
            }
            TokenKind::Ident(name) => {
                let id = name.clone();
                self.advance();

                // Check for Region Promotion: `promote(temp, outer_scope)`
                if id == "promote" && self.match_token(&TokenKind::LParen) {
                    let expr = self.parse_expression()?;
                    self.expect(TokenKind::Comma)?;
                    let target_region = match self.advance().kind {
                        TokenKind::Ident(r) => r,
                        other => return Err(format!("Expected target region name in promote, found {:?}", other)),
                    };
                    self.expect(TokenKind::RParen)?;
                    return Ok(Expression::Promote {
                        expr: Box::new(expr),
                        target_region,
                        span,
                    });
                }

                // Check for Struct Initialization: `User { id: 1, name: "Ali" }`
                if self.check(&TokenKind::LBrace) && id.chars().next().map_or(false, |c| c.is_uppercase()) {
                    self.advance();
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let fname = match self.advance().kind {
                            TokenKind::Ident(n) => n,
                            TokenKind::Struct => "st".to_string(),
                            TokenKind::Val => "val".to_string(),
                            TokenKind::Mut => "mut".to_string(),
                            TokenKind::Target => "target".to_string(),
                            TokenKind::Match => "match".to_string(),
                            TokenKind::Fn => "fn".to_string(),
                            TokenKind::In => "in".to_string(),
                            TokenKind::Asm => "asm".to_string(),
                            TokenKind::Region => "region".to_string(),
                            other => return Err(format!("Expected field name in struct init, found {:?}", other)),
                        };
                        let mut fvalue = Expression::Ident(fname.clone(), self.current_span());
                        if self.match_token(&TokenKind::Colon) {
                            fvalue = self.parse_expression()?;
                        }
                        fields.push((fname, fvalue));
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    return Ok(Expression::StructInit {
                        name: id,
                        fields,
                        span,
                    });
                }

                // Check for Enum Qualified Init: `Status.Pending` or `Status::Ok`
                let is_enum_access = if self.enum_names.contains(&id) {
                    if self.match_token(&TokenKind::Dot) {
                        true
                    } else if self.check(&TokenKind::Colon) {
                        self.advance();
                        self.match_token(&TokenKind::Colon)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_enum_access {
                    let vname = match self.advance().kind {
                        TokenKind::Ident(n) => n,
                        other => return Err(format!("Expected enum variant name, found {:?}", other)),
                    };
                    let mut payload = None;
                    if self.match_token(&TokenKind::LParen) {
                        payload = Some(Box::new(self.parse_expression()?));
                        self.expect(TokenKind::RParen)?;
                    }
                    return Ok(Expression::EnumInit {
                        enum_name: Some(id),
                        variant_name: vname,
                        payload,
                        span,
                    });
                }

                Ok(Expression::Ident(id, span))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                let blk = self.parse_block()?;
                Ok(Expression::Block(blk))
            }
            TokenKind::Alloc => {
                self.advance();
                let has_paren = self.match_token(&TokenKind::LParen);
                let target_type = self.parse_type()?;
                if has_paren {
                    self.match_token(&TokenKind::RParen);
                }
                Ok(Expression::Alloc {
                    allocator: Box::new(Expression::Ident("default_allocator".into(), span.clone())),
                    target_type,
                    span,
                })
            }
            other => Err(format!(
                "Unexpected token in expression: {:?} at line {}, col {}",
                other, span.line, span.col
            )),
        }
    }
}





