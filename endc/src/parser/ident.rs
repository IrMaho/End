use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
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
            TokenKind::Partial => Ok("partial".to_string()),
            TokenKind::Augment => Ok("augment".to_string()),
            TokenKind::ExtensionOnly => Ok("extension_only".to_string()),
            TokenKind::ExtensionPoint => Ok("extension_point".to_string()),
            TokenKind::Replace => Ok("replace".to_string()),
            TokenKind::Migration => Ok("migration".to_string()),
            TokenKind::Overlay => Ok("overlay".to_string()),
            TokenKind::Open => Ok("open".to_string()),
            TokenKind::Closed => Ok("closed".to_string()),
            TokenKind::Syntax => Ok("syntax".to_string()),
            TokenKind::CompilerPlugin => Ok("compiler_plugin".to_string()),
            TokenKind::Lint => Ok("lint".to_string()),
            TokenKind::Analyzer => Ok("analyzer".to_string()),
            TokenKind::TypeRule => Ok("type_rule".to_string()),
            TokenKind::Optimizer => Ok("optimizer".to_string()),
            TokenKind::BuildPlugin => Ok("build_plugin".to_string()),
            TokenKind::Generator => Ok("generator".to_string()),
            TokenKind::Reflect => Ok("reflect".to_string()),
            TokenKind::Lock => Ok("lock".to_string()),
            TokenKind::AgentExtension => Ok("agent_extension".to_string()),
            TokenKind::Proposal => Ok("proposal".to_string()),
            TokenKind::Evolvable => Ok("evolvable".to_string()),
            TokenKind::OwnedBy => Ok("owned_by".to_string()),
            TokenKind::ArchitectureTest => Ok("architecture_test".to_string()),
            TokenKind::At => Ok("at".to_string()),
            TokenKind::Provides => Ok("provides".to_string()),
            TokenKind::Guarantees => Ok("guarantees".to_string()),
            TokenKind::Rename => Ok("rename".to_string()),
            TokenKind::Use => Ok("use".to_string()),
            TokenKind::Snapshot => Ok("snapshot".to_string()),
            TokenKind::Begin => Ok("begin".to_string()),
            TokenKind::Commit => Ok("commit".to_string()),
            TokenKind::ReplaceWith => Ok("replace_with".to_string()),
            TokenKind::Api => Ok("api".to_string()),
            TokenKind::Needs => Ok("needs".to_string()),
            TokenKind::Expose => Ok("expose".to_string()),
            TokenKind::Replaceable => Ok("replaceable".to_string()),
            TokenKind::Lifecycle => Ok("lifecycle".to_string()),
            TokenKind::Decorate => Ok("decorate".to_string()),
            TokenKind::Impact => Ok("impact".to_string()),
            TokenKind::Must => Ok("must".to_string()),
            TokenKind::Reason => Ok("reason".to_string()),
            TokenKind::Internal => Ok("internal".to_string()),
            TokenKind::Private => Ok("private".to_string()),
            TokenKind::Extends => Ok("extends".to_string()),
            TokenKind::Extension => Ok("extension".to_string()),
            TokenKind::Implementation => Ok("implementation".to_string()),
            TokenKind::Test => Ok("test".to_string()),
            TokenKind::Access => Ok("access".to_string()),
            TokenKind::Grant => Ok("grant".to_string()),
            TokenKind::Adopt => Ok("adopt".to_string()),
            TokenKind::Implement => Ok("implement".to_string()),
            TokenKind::Attach => Ok("attach".to_string()),
            TokenKind::Detach => Ok("detach".to_string()),
            TokenKind::Mixin => Ok("mixin".to_string()),
            TokenKind::Capability => Ok("capability".to_string()),
            TokenKind::Provide => Ok("provide".to_string()),
            TokenKind::Require => Ok("require".to_string()),
            TokenKind::Resolve => Ok("resolve".to_string()),
            TokenKind::Select => Ok("select".to_string()),
            TokenKind::Project => Ok("project".to_string()),
            TokenKind::Delegate => Ok("delegate".to_string()),
            TokenKind::Proxy => Ok("proxy".to_string()),
            TokenKind::Intercept => Ok("intercept".to_string()),
            TokenKind::Hook => Ok("hook".to_string()),
            TokenKind::Enable => Ok("enable".to_string()),
            TokenKind::Disable => Ok("disable".to_string()),
            TokenKind::Scope => Ok("scope".to_string()),
            TokenKind::FeatureSwitch => Ok("feature_switch".to_string()),
            TokenKind::Traitify => Ok("traitify".to_string()),
            TokenKind::Equip => Ok("equip".to_string()),
            TokenKind::Fuse => Ok("fuse".to_string()),
            TokenKind::Shape => Ok("shape".to_string()),
            TokenKind::Only => Ok("only".to_string()),
            TokenKind::Section => Ok("section".to_string()),
            TokenKind::Before => Ok("before".to_string()),
            other => Err(format!("Expected identifier, found {:?} at line {}", other, tok.span.line)),
        }
    }

    pub fn parse_identifier_or_keyword_or_int(&mut self) -> Result<String, String> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(n) | TokenKind::StringLit(n) => Ok(n),
            TokenKind::IntLit(i) => Ok(i.to_string()),
            TokenKind::FloatLit(f) => Ok(f.to_string()),
            TokenKind::Feature => Ok("feature".to_string()),
            TokenKind::Contract => Ok("contract".to_string()),
            TokenKind::Api => Ok("api".to_string()),
            TokenKind::Needs => Ok("needs".to_string()),
            TokenKind::Pub => Ok("pub".to_string()),
            TokenKind::Mod => Ok("mod".to_string()),
            other => Ok(format!("{:?}", other)),
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
        } else {
            while !self.check(&TokenKind::LBrace) && !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::EOF) {
                let k = self.parse_identifier_or_keyword()?;
                self.match_token(&TokenKind::Colon);
                let v = self.parse_identifier_or_string()?;
                pairs.push((k, v));
                self.match_token(&TokenKind::Comma);
                self.match_token(&TokenKind::SemiColon);
            }
        }
        Ok(pairs)
    }


}
