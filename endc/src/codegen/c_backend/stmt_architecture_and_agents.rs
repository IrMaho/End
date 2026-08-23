use super::state::CBackend;
use crate::ast::Statement;

impl CBackend {
    pub(crate) fn gen_architecture_and_agents_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::AgentContract { name, scope, goal, constraints, body, .. } => {
                self.output.push_str(&format!("{}/* 🤖 [AGENT CONTRACT '{}']: scope='{}', goal='{}', constraints=[{}] */\n", self.indent(), name, scope, goal, constraints.join(", ")));
                if let Some(b) = body {
                    self.output.push_str(&format!("{}{{\n", self.indent()));
                    self.indent_level += 1;
                    for s in &b.statements {
                        self.gen_statement(s);
                    }
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                true
            }
            Statement::TaskDecl { name, body, .. } => {
                self.output.push_str(&format!("{}/* 📋 [TASK DECLARATION: {}] */\n", self.indent(), name));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::AcceptBlock { conditions, .. } => {
                self.output.push_str(&format!("{}/* ✅ [ACCEPT CONDITIONS]: [{}] */\n", self.indent(), conditions.join(", ")));
                true
            }
            Statement::RejectBlock { conditions, .. } => {
                self.output.push_str(&format!("{}/* ❌ [REJECT CONDITIONS]: [{}] */\n", self.indent(), conditions.join(", ")));
                true
            }
            Statement::BaselineBlock { metrics, .. } => {
                let m_str = metrics.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");
                self.output.push_str(&format!("{}/* 📊 [BASELINE PERFORMANCE METRICS]: {} */\n", self.indent(), m_str));
                true
            }
            Statement::RegressionCheck { condition, .. } => {
                self.output.push_str(&format!("{}/* 📉 [REGRESSION CHECK]: {} */\n", self.indent(), condition));
                true
            }
            Statement::ExplainBlock { topic, rationale, .. } => {
                self.output.push_str(&format!("{}/* 💡 [EXPLAIN '{}']: {} */\n", self.indent(), topic, rationale));
                true
            }
            Statement::ContextBlock { name, includes, excludes, body, .. } => {
                self.output.push_str(&format!("{}/* 🌐 [CONTEXT '{}']: include=[{}], exclude=[{}] */\n", self.indent(), name, includes.join(", "), excludes.join(", ")));
                if let Some(b) = body {
                    self.output.push_str(&format!("{}{{\n", self.indent()));
                    self.indent_level += 1;
                    for s in &b.statements {
                        self.gen_statement(s);
                    }
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                true
            }
            Statement::SliceDecl { name, from_target, includes, excludes, .. } => {
                self.output.push_str(&format!("{}/* 🔪 [CODE SLICE '{}']: from='{}', include=[{}], exclude=[{}] */\n", self.indent(), name, from_target, includes.join(", "), excludes.join(", ")));
                true
            }
            Statement::PatchDecl { target, body, .. } => {
                self.output.push_str(&format!("{}/* 🩹 [PATCH DECLARATION ON '{}'] */\n", self.indent(), target));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::EvolveBlock { target, intent, preserve, budget, allow, reject, verify, accept, body, .. } => {
                let b_str = budget.as_deref().unwrap_or("default");
                self.output.push_str(&format!("{}/* 🧬 [EVOLVE '{}']: intent='{}', preserve=[{}], budget='{}', allow=[{}], reject=[{}], verify=[{}], accept=[{}] */\n",
                    self.indent(), target, intent, preserve.join(", "), b_str, allow.join(", "), reject.join(", "), verify.join(", "), accept.join(", ")));
                if let Some(b) = body {
                    self.output.push_str(&format!("{}{{\n", self.indent()));
                    self.indent_level += 1;
                    for s in &b.statements {
                        self.gen_statement(s);
                    }
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                true
            }
            Statement::BoundaryDecl { name, allows, denies, is_sealed, .. } => {
                self.output.push_str(&format!("{}/* 🏰 [ARCHITECTURAL BOUNDARY: '{}'] sealed={}, allow=[{}], deny=[{}] */\n", self.indent(), name, is_sealed, allows.join(", "), denies.join(", ")));
                true
            }
            Statement::ResponsibilityDecl { module_name, description, .. } => {
                self.output.push_str(&format!("{}/* 🎯 [MODULE RESPONSIBILITY: '{}']: \"{}\" */\n", self.indent(), module_name, description));
                true
            }
            Statement::OwnsDecl { module_name, symbols, .. } => {
                self.output.push_str(&format!("{}/* 👑 [MODULE OWNS: '{}']: [{}] */\n", self.indent(), module_name, symbols.join(", ")));
                true
            }
            Statement::ExposesDecl { module_name, symbols, .. } => {
                self.output.push_str(&format!("{}/* 🚪 [MODULE EXPOSES: '{}']: [{}] */\n", self.indent(), module_name, symbols.join(", ")));
                true
            }
            Statement::DependsDecl { from_module, target_module, is_only, .. } => {
                let only_str = if *is_only { " (depends_only)" } else { "" };
                self.output.push_str(&format!("{}/* 🔗 [DEPENDENCY: '{}' -> '{}'{} ] */\n", self.indent(), from_module, target_module, only_str));
                true
            }
            Statement::ForbidDecl { from, to, .. } => {
                self.output.push_str(&format!("{}/* 🚫 [FORBID DEPENDENCY: '{}' -> '{}'] */\n", self.indent(), from, to));
                true
            }
            Statement::LayerDecl { name, forbid_depends, .. } => {
                self.output.push_str(&format!("{}/* 🧱 [ARCHITECTURAL LAYER: '{}'] forbid_depends=[{}] */\n", self.indent(), name, forbid_depends.join(", ")));
                true
            }
            Statement::DirectionDecl { from, to, .. } => {
                self.output.push_str(&format!("{}/* 🧭 [FLOW DIRECTION: '{}' -> '{}'] */\n", self.indent(), from, to));
                true
            }
            Statement::SplitDecl { entity, parts, .. } => {
                self.output.push_str(&format!("{}/* ✂️ [SPLIT ENTITY '{}']: into [{}] */\n", self.indent(), entity, parts.join(", ")));
                true
            }
            Statement::PartitionDecl { entity, by, parts, .. } => {
                self.output.push_str(&format!("{}/* 📦 [PARTITION ENTITY '{}' by '{}']: [{}] */\n", self.indent(), entity, by, parts.join(", ")));
                true
            }
            Statement::ExtractDecl { symbols, into_module, .. } => {
                self.output.push_str(&format!("{}/* ⛏️ [EXTRACT TO '{}']: symbols=[{}] */\n", self.indent(), into_module, symbols.join(", ")));
                true
            }
            Statement::ClusterDecl { by, predicate, .. } => {
                self.output.push_str(&format!("{}/* 🌌 [CLUSTER BY '{}']: predicate='{}' */\n", self.indent(), by, predicate));
                true
            }
            Statement::SeparateDecl { left, right, .. } => {
                self.output.push_str(&format!("{}/* ↔️ [SEPARATE MODULES]: '{}' from '{}' */\n", self.indent(), left, right));
                true
            }
            Statement::ModuleContractDecl { module_name, accepts, returns, guarantees, .. } => {
                self.output.push_str(&format!("{}/* 📜 [MODULE CONTRACT '{}']: accepts=[{}], returns=[{}], guarantees=[{}] */\n", self.indent(), module_name, accepts.join(", "), returns.join(", "), guarantees.join(", ")));
                true
            }
            Statement::ContractDefinition(ctr) => {
                self.output.push_str(&format!("{}/* 📜 [MODULE CONTRACT '{}']: clauses=[{}] */\n", self.indent(), ctr.name, ctr.clauses.join(", ")));
                true
            }
            Statement::FeatureStatement(f) => {
                let req = f.contracts.first().map(|c| c.rule.clone());
                let skills = f.requires_capabilities.join(", ");
                let tasks: Vec<String> = f.decisions.iter().map(|d| d.target.clone()).collect();
                self.output.push_str(&format!("{}/* 🎯 [FEATURE '{}']: req={:?}, skills=[{}], tasks=[{}] */\n", self.indent(), f.name, req, skills, tasks.join(", ")));
                true
            }
            Statement::PortDecl { name, methods, .. } => {
                self.output.push_str(&format!("{}/* 🔌 [PORT '{}']: methods=[{}] */\n", self.indent(), name, methods.join(", ")));
                true
            }
            Statement::AdapterDecl { name, port, body, .. } => {
                self.output.push_str(&format!("{}/* 🔌 [ADAPTER '{}' for port '{}'] */\n", self.indent(), name, port));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::FacadeDecl { name, exposes, .. } => {
                self.output.push_str(&format!("{}/* 🏛️ [FACADE '{}']: exposes=[{}] */\n", self.indent(), name, exposes.join(", ")));
                true
            }
            Statement::GatewayDecl { from_mod, to_mod, allowed_calls, .. } => {
                self.output.push_str(&format!("{}/* 🌉 [GATEWAY '{}' -> '{}']: allowed_calls=[{}] */\n", self.indent(), from_mod, to_mod, allowed_calls.join(", ")));
                true
            }
            Statement::ArchInvariantDecl { rule, .. } => {
                self.output.push_str(&format!("{}/* ⚖️ [ARCH INVARIANT]: {} */\n", self.indent(), rule));
                true
            }
            Statement::PreserveRefactorDecl { preserves, body, .. } => {
                self.output.push_str(&format!("{}/* 🛡️ [PRESERVE REFACTOR INVARIANTS: {}] */\n", self.indent(), preserves.join(", ")));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::CompatDecl { module_name, version, body, .. } => {
                self.output.push_str(&format!("{}/* 🔄 [COMPATIBILITY REGION for '{}': v{}] */\n", self.indent(), module_name, version));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::StableDecl { api_name, .. } => {
                self.output.push_str(&format!("{}/* ⚓ [STABLE API SYMBOLS]: {} */\n", self.indent(), api_name));
                true
            }
            Statement::SealedDecl { boundary_name, .. } => {
                self.output.push_str(&format!("{}/* 🔒 [SEALED MODULE: '{}'] */\n", self.indent(), boundary_name));
                true
            }
            Statement::FriendDecl { module_name, friend_module, .. } => {
                self.output.push_str(&format!("{}/* 🤝 [FRIEND MODULE: '{}' grants internal access to '{}'] */\n", self.indent(), module_name, friend_module));
                true
            }
            Statement::PrivateToDecl { symbol, module_name, .. } => {
                self.output.push_str(&format!("{}/* 🙈 [PRIVATE SYMBOL: '{}' is private_to '{}'] */\n", self.indent(), symbol, module_name));
                true
            }
            Statement::SurfaceDecl { name, exposes, hides, .. } => {
                self.output.push_str(&format!("{}/* 🛡️ [SURFACE OF '{}']: exposes=[{}], hides=[{}] */\n", self.indent(), name, exposes.join(", "), hides.join(", ")));
                true
            }
            Statement::LeakCheckDecl { module_name, symbol, through, .. } => {
                self.output.push_str(&format!("{}/* 💧 [LEAK CHECK on '{}']: forbid '{}' leaking through '{}' */\n", self.indent(), module_name, symbol, through));
                true
            }
            Statement::PurityDecl { module_name, level, .. } => {
                self.output.push_str(&format!("{}/* ✨ [MODULE PURITY: '{}']: level='{}' */\n", self.indent(), module_name, level));
                true
            }
            Statement::ViewDecl { name, includes, .. } => {
                self.output.push_str(&format!("{}/* 🔭 [VIEW '{}']: includes=[{}] */\n", self.indent(), name, includes.join(", ")));
                true
            }
            Statement::LensDecl { name, focus, hide, .. } => {
                self.output.push_str(&format!("{}/* 🔍 [LENS '{}']: focus='{}', hide='{}' */\n", self.indent(), name, focus, hide));
                true
            }
            Statement::AgentScopeDecl { name, modules, forbid, .. } => {
                self.output.push_str(&format!("{}/* 🤖 [AGENT SCOPE '{}']: modules=[{}], forbid=[{}] */\n", self.indent(), name, modules.join(", "), forbid.join(", ")));
                true
            }
            Statement::BudgetContextDecl { name, token_budget, priority, .. } => {
                self.output.push_str(&format!("{}/* 💰 [BUDGET CONTEXT '{}']: tokens={}, priority=[{}] */\n", self.indent(), name, token_budget, priority.join(", ")));
                true
            }
            Statement::MoveDecl { symbol, from_mod, to_mod, .. } => {
                self.output.push_str(&format!("{}/* 🚚 [MOVE SYMBOL]: '{}' from '{}' -> '{}' */\n", self.indent(), symbol, from_mod, to_mod));
                true
            }
            Statement::MigrateDecl { entity, from_mod, to_mod, .. } => {
                self.output.push_str(&format!("{}/* 🏗️ [MIGRATE ENTITY]: '{}' from '{}' -> '{}' */\n", self.indent(), entity, from_mod, to_mod));
                true
            }
            Statement::BridgeDecl { from_mod, to_mod, via, .. } => {
                self.output.push_str(&format!("{}/* 🌉 [BRIDGE]: '{}' -> '{}' via '{}' */\n", self.indent(), from_mod, to_mod, via));
                true
            }
            Statement::RedirectDecl { from_api, to_api, .. } => {
                self.output.push_str(&format!("{}/* 🔀 [REDIRECT API]: '{}' -> '{}' */\n", self.indent(), from_api, to_api));
                true
            }
            Statement::DeprecateDecl { target_api, after_cond, remove_cond, .. } => {
                self.output.push_str(&format!("{}/* ⏳ [DEPRECATE '{}']: after='{}', remove_when='{}' */\n", self.indent(), target_api, after_cond, remove_cond));
                true
            }
            Statement::CycleFreeDecl { .. } => {
                self.output.push_str(&format!("{}/* 🔄 [ARCHITECTURE INVARIANT: cycle_free = true] */\n", self.indent()));
                true
            }
            Statement::FanoutDecl { module_name, limit, .. } => {
                self.output.push_str(&format!("{}/* 🌲 [MAX FANOUT on '{}']: limit={} */\n", self.indent(), module_name, limit));
                true
            }
            Statement::FaninDecl { module_name, limit, .. } => {
                self.output.push_str(&format!("{}/* 🌲 [MAX FANIN on '{}']: limit={} */\n", self.indent(), module_name, limit));
                true
            }
            Statement::DepthDecl { limit, .. } => {
                self.output.push_str(&format!("{}/* 📏 [MAX DEPTH]: limit={} */\n", self.indent(), limit));
                true
            }
            Statement::CohesionDecl { module_name, min_threshold, .. } => {
                self.output.push_str(&format!("{}/* 🧲 [COHESION REQUIREMENT on '{}']: min_threshold={:.2} */\n", self.indent(), module_name, min_threshold));
                true
            }
            Statement::ModularizeDecl { target, target_files_min, target_files_max, preserve, .. } => {
                self.output.push_str(&format!("{}/* 🧩 [MODULARIZE '{}']: target_files={}..{}, preserve=[{}] */\n", self.indent(), target, target_files_min, target_files_max, preserve.join(", ")));
                true
            }
            Statement::DecomposeDecl { target, target_modules, optimize, preserve, gravity, .. } => {
                let tm_str = target_modules.map(|n| n.to_string()).unwrap_or_else(|| "auto".to_string());
                let g_str = gravity.as_deref().unwrap_or("semantic");
                self.output.push_str(&format!("{}/* 💥 [DECOMPOSE MONOLITH '{}']: target_modules={}, optimize=[{}], preserve=[{}], gravity='{}' */\n",
                    self.indent(), target, tm_str, optimize.join(", "), preserve.join(", "), g_str));
                true
            }
            Statement::ArchitectureDecl { name, layers, rules, invariants, .. } => {
                self.output.push_str(&format!("{}/* 🏛️ [ARCHITECTURE SPECIFICATION '{}']: layers=[{}], rules=[{}], invariants=[{}] */\n",
                    self.indent(), name, layers.join(", "), rules.join(", "), invariants.join(", ")));
                true
            }
            Statement::RepairDecl { target, .. } => {
                self.output.push_str(&format!("{}/* 🔧 [ARCHITECTURE REPAIR on '{}'] */\n", self.indent(), target));
                true
            }
            Statement::EvolveArchDecl { from, toward, target_modules, preserve, optimize, reject_if, verify, .. } => {
                self.output.push_str(&format!("{}/* 🚀 [EVOLVE ARCHITECTURE]: from='{}' toward='{}', target_modules={}, preserve=[{}], optimize=[{}], reject_if=[{}], verify=[{}] */\n",
                    self.indent(), from, toward, target_modules, preserve.join(", "), optimize.join(", "), reject_if.join(", "), verify.join(", ")));
                true
            }
            Statement::GravityDecl { weights, .. } => {
                let w_str = weights.iter().map(|(k, v)| format!("{}: {:.2}", k, v)).collect::<Vec<_>>().join(", ");
                self.output.push_str(&format!("{}/* 🌌 [MODULE GRAVITY MATRIX]: {} */\n", self.indent(), w_str));
                true
            }
            Statement::FeatureDecl { name, requirement, skills, tasks, .. } => {
                self.output.push_str(&format!("{}/* 🎯 [FEATURE '{}']: req={:?}, skills=[{}], tasks=[{}] */\n", self.indent(), name, requirement, skills.join(", "), tasks.join(", ")));
                true
            }
            Statement::SkillDecl { name, rules, constraints, requires, for_scope, .. } => {
                self.output.push_str(&format!("{}/* 🧠 [SKILL '{}' FOR {:?}]: rules=[{}], constraints=[{}], requires=[{}] */\n", self.indent(), name, for_scope, rules.join(", "), constraints.join(", "), requires.join(", ")));
                true
            }
            Statement::SatisfiesDecl { entity, skills, .. } => {
                self.output.push_str(&format!("{}/* ✅ [SATISFIES]: '{}' -> [{}] */\n", self.indent(), entity, skills.join(", ")));
                true
            }
            Statement::ProjectSkillsDecl { profile, .. } => {
                let p_str = profile.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");
                self.output.push_str(&format!("{}/* 🏛️ [PROJECT SKILLS PROFILE]: {} */\n", self.indent(), p_str));
                true
            }
            Statement::AgentTaskContractDecl { name, owner, status, requirement, implementation, skills, .. } => {
                self.output.push_str(&format!("{}/* 📋 [AGENT TASK CONTRACT '{}']: owner={:?}, status={:?}, req={:?}, impl={:?}, skills=[{}] */\n", self.indent(), name, owner, status, requirement, implementation, skills.join(", ")));
                true
            }
            Statement::ClaimTask { task_name, .. } => {
                self.output.push_str(&format!("{}/* 🙋 [CLAIM TASK]: '{}' */\n", self.indent(), task_name));
                true
            }
            Statement::CompleteTask { task_name, result, confidence, evidence, .. } => {
                self.output.push_str(&format!("{}/* 🏁 [COMPLETE TASK '{}']: result={}, confidence={:?}, evidence=[{}] */\n", self.indent(), task_name, result, confidence, evidence.join(", ")));
                true
            }
            Statement::VerifyTask { target, is_adversarial, skill, .. } => {
                self.output.push_str(&format!("{}/* 🔍 [VERIFY TASK '{}']: adversarial={}, skill={:?} */\n", self.indent(), target, is_adversarial, skill));
                true
            }
            Statement::RequirementDecl { req_id, description, .. } => {
                self.output.push_str(&format!("{}/* 📜 [REQUIREMENT '{}']: \"{}\" */\n", self.indent(), req_id, description));
                true
            }
            Statement::ImplementsDecl { req_id, entities, .. } => {
                self.output.push_str(&format!("{}/* 🔨 [IMPLEMENTS '{}']: [{}] */\n", self.indent(), req_id, entities.join(", ")));
                true
            }
            Statement::VerifiesDecl { req_id, entities, .. } => {
                self.output.push_str(&format!("{}/* 🛡️ [VERIFIES '{}']: [{}] */\n", self.indent(), req_id, entities.join(", ")));
                true
            }
            Statement::TodoDecl { id, implement, requires, verify, status, .. } => {
                self.output.push_str(&format!("{}/* 📝 [EXECUTABLE TODO '{}']: implement=\"{}\", status=\"{}\", requires=[{}], verify=[{}] */\n", self.indent(), id, implement, status, requires.join(", "), verify.join(", ")));
                true
            }
            Statement::AgentBoundaryDecl { module_name, .. } => {
                self.output.push_str(&format!("{}/* 🧱 [AGENT BOUNDARY]: '{}' */\n", self.indent(), module_name));
                true
            }
            Statement::AgentContextDecl { module_name, expose, hide, .. } => {
                self.output.push_str(&format!("{}/* 👁️ [AGENT CONTEXT '{}']: expose=[{}], hide=[{}] */\n", self.indent(), module_name, expose.join(", "), hide.join(", ")));
                true
            }
            Statement::ContextFirewallDecl { module_name, deny, expose, .. } => {
                self.output.push_str(&format!("{}/* 🧱🔥 [CONTEXT FIREWALL '{}']: deny=[{}], expose=[{}] */\n", self.indent(), module_name, deny.join(", "), expose.join(", ")));
                true
            }
            Statement::AgentApiDecl { module_name, expose, hide, .. } => {
                self.output.push_str(&format!("{}/* 🤖 [AGENT API '{}']: expose=[{}], hide=[{}] */\n", self.indent(), module_name, expose.join(", "), hide.join(", ")));
                true
            }
            Statement::AgentabilityDecl { max_context_tokens, max_operation_complexity, max_dependency_fanout, .. } => {
                self.output.push_str(&format!("{}/* ⚙️ [AGENTABILITY BUDGET]: max_context_tokens={}, complexity={}, fanout={} */\n", self.indent(), max_context_tokens, max_operation_complexity, max_dependency_fanout));
                true
            }
            Statement::IntentDecl { goal, preserve, optimize, .. } => {
                self.output.push_str(&format!("{}/* 🎯 [INTENT]: goal=\"{}\", preserve=[{}], optimize=[{}] */\n", self.indent(), goal, preserve.join(", "), optimize.join(", ")));
                true
            }
            Statement::SemanticCommitDecl { task, intent, satisfies, evidence, .. } => {
                self.output.push_str(&format!("{}/* 💾 [SEMANTIC COMMIT]: task=\"{}\", intent=\"{}\", satisfies=[{}], evidence=[{}] */\n", self.indent(), task, intent, satisfies.join(", "), evidence.join(", ")));
                true
            }
            Statement::AgentReviewDecl { task_id, summary, completed, unresolved, risks, confidence, .. } => {
                self.output.push_str(&format!("{}/* 🧐 [AGENT REVIEW '{}']: summary=\"{}\", completed={}, unresolved={}, risks={}, confidence={:.2} */\n", self.indent(), task_id, summary, completed, unresolved, risks, confidence));
                true
            }
            Statement::ApprovalDecl { required_items, .. } => {
                self.output.push_str(&format!("{}/* ✍️ [APPROVAL REQUIRED]: [{}] */\n", self.indent(), required_items.join(", ")));
                true
            }
            Statement::AgentLeaseDecl { module_name, owner, duration, .. } => {
                self.output.push_str(&format!("{}/* 🔑 [AGENT LEASE on '{}']: owner=\"{}\", duration=\"{}\" */\n", self.indent(), module_name, owner, duration));
                true
            }
            Statement::KnowledgeDecl { name, decisions, constraints, .. } => {
                self.output.push_str(&format!("{}/* 📚 [KNOWLEDGE '{}']: decisions=[{}], constraints=[{}] */\n", self.indent(), name, decisions.join(", "), constraints.join(", ")));
                true
            }
            Statement::DecisionDecl { id, choose, because, reject, .. } => {
                self.output.push_str(&format!("{}/* ⚖️ [DECISION ADR '{}']: choose=\"{}\", because=\"{}\", reject=\"{}\" */\n", self.indent(), id, choose, because, reject));
                true
            }
            Statement::AgentCapabilityDecl { capabilities, cannot, .. } => {
                self.output.push_str(&format!("{}/* 🛡️ [AGENT CAPABILITIES]: can=[{}], cannot=[{}] */\n", self.indent(), capabilities.join(", "), cannot.join(", ")));
                true
            }
            Statement::RegressionGuardDecl { items, .. } => {
                self.output.push_str(&format!("{}/* 🛡️ [REGRESSION GUARD]: [{}] */\n", self.indent(), items.join(", ")));
                true
            }
            _ => false,
        }
    }
}
