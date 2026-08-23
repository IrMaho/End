use super::state::Interpreter;
use super::value::{AgentReportState, SkillDefState, TaskState, TodoState, Value};
use crate::ast::Statement;

impl Interpreter {
    pub(crate) fn eval_agent_contracts_and_oop_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, String> {
        match stmt {
            Statement::FeatureDecl { name, requirement, skills, tasks, .. } => {
                self.features.insert(name.clone(), (requirement.clone(), skills.clone(), tasks.clone()));
                self.set_var(&format!("__feature_{}", name), Value::String(format!("Feature {}: req={:?}, skills={:?}, tasks={:?}", name, requirement, skills, tasks)));
                Ok(None)
            }
            Statement::FeatureStatement(f) => {
                let req = f.contracts.first().map(|c| c.rule.clone());
                let skills = f.requires_capabilities.clone();
                let tasks: Vec<String> = f.decisions.iter().map(|d| d.target.clone()).collect();
                self.features.insert(f.name.clone(), (req.clone(), skills.clone(), tasks.clone()));
                self.set_var(&format!("__feature_{}", f.name), Value::String(format!("Feature {}: req={:?}, skills={:?}, tasks={:?}", f.name, req, skills, tasks)));
                Ok(None)
            }
            Statement::SkillDecl { name, rules, constraints, structural, semantic, behavioral, architectural, performance, security, testing, agent, requires, hard, soft, for_scope, .. } => {
                let state = SkillDefState {
                    name: name.clone(),
                    rules: rules.clone(),
                    constraints: constraints.clone(),
                    structural: structural.clone(),
                    semantic: semantic.clone(),
                    behavioral: behavioral.clone(),
                    architectural: architectural.clone(),
                    performance: performance.clone(),
                    security: security.clone(),
                    testing: testing.clone(),
                    agent: agent.clone(),
                    requires: requires.clone(),
                    hard: hard.clone(),
                    soft: soft.clone(),
                    for_scope: for_scope.clone(),
                };
                self.skills.insert(name.clone(), state);
                self.set_var(&format!("__skill_{}", name), Value::String(format!("Skill {}: scope={:?}, rules={:?}, requires={:?}", name, for_scope, rules, requires)));
                Ok(None)
            }
            Statement::SatisfiesDecl { entity, skills, .. } => {
                self.set_var(&format!("__satisfies_{}", entity), Value::String(skills.join(", ")));
                Ok(None)
            }
            Statement::ProjectSkillsDecl { profile, .. } => {
                for (k, v) in profile {
                    self.project_profile.insert(k.clone(), v.clone());
                }
                self.set_var("__project_skills", Value::String(format!("{:?}", self.project_profile)));
                Ok(None)
            }
            Statement::AgentTaskContractDecl { name, owner, status, requirement, implementation, skills, change_budget, evidence, .. } => {
                let t_owner = owner.clone().unwrap_or_else(|| "agent".to_string());
                let t_status = status.clone().unwrap_or_else(|| "planned".to_string());
                let state = TaskState {
                    name: name.clone(),
                    owner: t_owner,
                    status: t_status,
                    requirement: requirement.clone(),
                    implementation: implementation.clone(),
                    skills: skills.clone(),
                    change_budget: change_budget.clone(),
                    evidence: evidence.clone(),
                    result: None,
                    confidence: None,
                    summary: None,
                    notes: None,
                };
                self.tasks_state.insert(name.clone(), state);
                self.set_var(&format!("__task_{}", name), Value::String(format!("Task {}: req={:?}, impl={:?}, skills={:?}", name, requirement, implementation, skills)));
                Ok(None)
            }
            Statement::ClaimTask { task_name, .. } => {
                if let Some(task) = self.tasks_state.get_mut(task_name) {
                    task.status = "claimed".to_string();
                    task.owner = "agent".to_string();
                } else {
                    let state = TaskState {
                        name: task_name.clone(),
                        owner: "agent".to_string(),
                        status: "claimed".to_string(),
                        requirement: None,
                        implementation: None,
                        skills: Vec::new(),
                        change_budget: Vec::new(),
                        evidence: Vec::new(),
                        result: None,
                        confidence: None,
                        summary: None,
                        notes: None,
                    };
                    self.tasks_state.insert(task_name.clone(), state);
                }
                self.set_var(&format!("__claim_task_{}", task_name), Value::String("claimed".to_string()));
                Ok(None)
            }
            Statement::CompleteTask { task_name, result, confidence, summary, evidence, notes, .. } => {
                if let Some(task) = self.tasks_state.get_mut(task_name) {
                    task.status = "completed".to_string();
                    task.result = Some(result.clone());
                    task.confidence = *confidence;
                    task.summary = summary.clone();
                    task.notes = notes.clone();
                } else {
                    let state = TaskState {
                        name: task_name.clone(),
                        owner: "agent".to_string(),
                        status: "completed".to_string(),
                        requirement: None,
                        implementation: None,
                        skills: Vec::new(),
                        change_budget: Vec::new(),
                        evidence: evidence.iter().map(|e| ("evidence".to_string(), e.clone())).collect(),
                        result: Some(result.clone()),
                        confidence: *confidence,
                        summary: summary.clone(),
                        notes: notes.clone(),
                    };
                    self.tasks_state.insert(task_name.clone(), state);
                }
                self.set_var(&format!("__complete_task_{}", task_name), Value::String(format!("result={}, confidence={:?}, evidence={:?}", result, confidence, evidence)));
                Ok(None)
            }
            Statement::VerifyTask { target, is_adversarial, skill, .. } => {
                self.verified_tasks.insert(target.clone());
                if let Some(task) = self.tasks_state.get_mut(target) {
                    task.status = "accepted".to_string();
                }
                self.set_var(&format!("__verify_task_{}", target), Value::String(format!("verified: adversarial={}, skill={:?}", is_adversarial, skill)));
                Ok(None)
            }
            Statement::RequirementDecl { req_id, description, .. } => {
                self.requirements.insert(req_id.clone(), description.clone());
                self.set_var(&format!("__requirement_{}", req_id), Value::String(description.clone()));
                Ok(None)
            }
            Statement::ImplementsDecl { req_id, entities, .. } => {
                self.requirement_implements.insert(req_id.clone(), entities.clone());
                self.set_var(&format!("__implements_{}", req_id), Value::String(entities.join(", ")));
                Ok(None)
            }
            Statement::VerifiesDecl { req_id, entities, .. } => {
                self.requirement_verifies.insert(req_id.clone(), entities.clone());
                self.set_var(&format!("__verifies_{}", req_id), Value::String(entities.join(", ")));
                Ok(None)
            }
            Statement::TodoDecl { id, implement, requires, verify, status, .. } => {
                let state = TodoState {
                    id: id.clone(),
                    implement: implement.clone(),
                    requires: requires.clone(),
                    verify: verify.clone(),
                    status: status.clone(),
                };
                self.todos_state.insert(id.clone(), state);
                self.set_var(&format!("__todo_{}", id), Value::String(format!("Todo {}: implement={}, status={}", id, implement, status)));
                Ok(None)
            }
            Statement::AgentBoundaryDecl { module_name, .. } => {
                self.set_var(&format!("__agent_boundary_{}", module_name), Value::Bool(true));
                Ok(None)
            }
            Statement::AgentContextDecl { module_name, expose, hide, .. } => {
                self.set_var(&format!("__agent_context_{}", module_name), Value::String(format!("expose: {:?}, hide: {:?}", expose, hide)));
                Ok(None)
            }
            Statement::ContextFirewallDecl { module_name, deny, expose, .. } => {
                self.set_var(&format!("__context_firewall_{}", module_name), Value::String(format!("deny: {:?}, expose: {:?}", deny, expose)));
                Ok(None)
            }
            Statement::AgentApiDecl { module_name, expose, hide, .. } => {
                self.set_var(&format!("__agent_api_{}", module_name), Value::String(format!("expose: {:?}, hide: {:?}", expose, hide)));
                Ok(None)
            }
            Statement::AgentabilityDecl { max_context_tokens, max_operation_complexity, max_dependency_fanout, .. } => {
                self.set_var("__agentability", Value::String(format!("max_tokens={}, complexity={}, fanout={}", max_context_tokens, max_operation_complexity, max_dependency_fanout)));
                Ok(None)
            }
            Statement::IntentDecl { goal, preserve, optimize, .. } => {
                self.set_var("__intent", Value::String(format!("goal={}, preserve={:?}, optimize={:?}", goal, preserve, optimize)));
                Ok(None)
            }
            Statement::SemanticCommitDecl { task, intent, satisfies, evidence, .. } => {
                self.set_var(&format!("__semantic_commit_{}", task), Value::String(format!("intent={}, satisfies={:?}, evidence={:?}", intent, satisfies, evidence)));
                Ok(None)
            }
            Statement::AgentReviewDecl { task_id, summary, completed, unresolved, risks, confidence, .. } => {
                let report = AgentReportState {
                    task_id: task_id.clone(),
                    summary: summary.clone(),
                    completed: *completed,
                    unresolved: *unresolved,
                    risks: *risks,
                    confidence: *confidence,
                };
                self.agent_reports.push(report);
                self.set_var(&format!("__review_{}", task_id), Value::String(format!("summary={}, completed={}, confidence={}", summary, completed, confidence)));
                Ok(None)
            }
            Statement::ApprovalDecl { required_items, .. } => {
                self.set_var("__approval_required", Value::String(required_items.join(", ")));
                Ok(None)
            }
            Statement::AgentLeaseDecl { module_name, owner, duration, .. } => {
                self.agent_leases.insert(module_name.clone(), (owner.clone(), duration.clone()));
                self.set_var(&format!("__lease_{}", module_name), Value::String(format!("owner={}, duration={}", owner, duration)));
                Ok(None)
            }
            Statement::KnowledgeDecl { name, decisions, constraints, .. } => {
                self.knowledge_base.insert(name.clone(), (decisions.clone(), constraints.clone()));
                self.set_var(&format!("__knowledge_{}", name), Value::String(format!("decisions={:?}, constraints={:?}", decisions, constraints)));
                Ok(None)
            }
            Statement::DecisionDecl { id, choose, because, reject, .. } => {
                self.decision_records.insert(id.clone(), (choose.clone(), because.clone(), reject.clone()));
                self.set_var(&format!("__decision_{}", id), Value::String(format!("choose={}, because={}, reject={}", choose, because, reject)));
                Ok(None)
            }
            Statement::AgentCapabilityDecl { capabilities, cannot, .. } => {
                self.set_var("__agent_capabilities", Value::String(format!("can={:?}, cannot={:?}", capabilities, cannot)));
                Ok(None)
            }
            Statement::RegressionGuardDecl { items, .. } => {
                self.set_var("__regression_guard", Value::String(items.join(", ")));
                Ok(None)
            }
            Statement::ClassDecl(c) => {
                self.set_var(&format!("__class_{}", c.name), Value::String(format!("extends={:?}, mixins={:?}, implements={:?}", c.extends, c.mixins, c.implements)));
                for m in &c.methods {
                    self.functions.insert(format!("{}::{}", c.name, m.name), m.clone());
                }
                Ok(None)
            }
            Statement::TraitDecl(t) => {
                self.set_var(&format!("__trait_{}", t.name), Value::String(format!("extends={:?}", t.extends)));
                Ok(None)
            }
            Statement::InheritStmt(i) => {
                if let Some(b) = &i.body {
                    self.push_scope();
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                    self.pop_scope();
                }
                self.set_var(&format!("__inherit_{}_{}", i.target, i.parent), Value::String(format!("kind={:?}, is_contractual={}", i.kind, i.is_contractual)));
                Ok(None)
            }
            Statement::SuperCallStmt(s) => {
                let mut evaluated_args = Vec::new();
                for arg in &s.args {
                    evaluated_args.push(self.eval_expression(arg)?);
                }
                let target_fn = match &s.target_parent {
                    Some(parent) => format!("{}::{}", parent, s.method),
                    None => s.method.clone(),
                };
                if let Some(func) = self.functions.get(&target_fn).cloned() {
                    let res = self.eval_function(&func, evaluated_args)?;
                    Ok(Some(res))
                } else {
                    self.set_var(&format!("__super_{}", s.method), Value::String(format!("parent={:?}, args={:?}", s.target_parent, evaluated_args)));
                    Ok(None)
                }
            }
            Statement::ConflictStmt(c) => {
                self.set_var(&format!("__conflict_{}_{}", c.left.replace('.', "_"), c.right.replace('.', "_")), Value::Bool(true));
                Ok(None)
            }
            Statement::ResolveConflictStmt(r) => {
                self.set_var(&format!("__resolve_{}", r.preferred.replace('.', "_")), Value::String(r.over.clone().unwrap_or_default()));
                Ok(None)
            }
            Statement::InspectInheritanceStmt(i) => {
                self.set_var(&format!("__inspect_inheritance_{}", i.target), Value::Bool(true));
                Ok(None)
            }
            Statement::ImpactInheritanceStmt(i) => {
                self.set_var(&format!("__impact_inheritance_{}", i.target), Value::Bool(true));
                Ok(None)
            }
            Statement::RefactorSessionStmt(s) => {
                self.set_var(&format!("__refactor_session_{}", s.agent_name), Value::String(format!("target={}, scope={:?}, forbid={:?}", s.target, s.scope, s.forbid)));
                Ok(None)
            }
            Statement::DecompositionPlanStmt(d) => {
                self.set_var(&format!("__decomposition_plan_{}", d.source.replace(['/', '.', '\\'], "_")), Value::String(format!("submodules_count={}, target_arch={}", d.submodules.len(), d.target_architecture)));
                Ok(None)
            }
            Statement::ConservationAuditStmt(c) => {
                self.set_var(&format!("__conservation_audit_{}", c.original_source.replace(['/', '.', '\\'], "_")), Value::String(format!("orig_loc={}, new_loc={}, unaccounted={}", c.original_loc, c.new_loc, c.unaccounted_count)));
                Ok(None)
            }
            Statement::SolidAuditStmt(s) => {
                self.set_var(&format!("__solid_audit_{}", s.module_name), Value::String(format!("srp={}, ocp={}, lsp={}, isp={}, dip={}", s.verify_srp, s.verify_ocp, s.verify_lsp, s.verify_isp, s.verify_dip)));
                Ok(None)
            }
            Statement::RefactoringTxStmt(r) => {
                self.set_var(&format!("__refactoring_tx_{}", r.tx_name), Value::String(format!("checkpoint={}, steps_count={}, rollback={}", r.checkpoint, r.steps.len(), r.auto_rollback)));
                Ok(None)
            }
            Statement::SymbolInventoryStmt(s) => {
                self.set_var(&format!("__symbol_inventory_{}", s.module_name), Value::String(format!("classes={}, functions={}, types={}", s.classes.len(), s.functions.len(), s.types.len())));
                Ok(None)
            }
            Statement::TraceableMapStmt(t) => {
                self.set_var(&format!("__traceable_map_{}", t.source_module.replace(['/', '.', '\\'], "_")), Value::String(format!("mappings_count={}", t.mappings.len())));
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
