use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentScopeDef {
    pub name: String,
    pub scope_pattern: String,
    pub allow_actions: Vec<String>,
    pub deny_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentScopeAuditReport {
    pub agent_name: String,
    pub target_file: String,
    pub requested_action: String,
    pub is_authorized: bool,
    pub within_scope: bool,
    pub granted_permissions: Vec<String>,
    pub denied_violations: Vec<String>,
    pub status_message: String,
}

pub struct AgentScopeEngine;

impl AgentScopeEngine {
    pub fn check_permission(
        agent: &AgentScopeDef,
        target_file: &str,
        requested_action: &str,
        requested_capability: Option<&str>,
    ) -> AgentScopeAuditReport {
        let normalized_target = target_file.replace('\\', "/");
        let normalized_scope = agent.scope_pattern.replace('\\', "/");

        // 1. Check Scope Pattern Matching
        let mut within_scope = false;
        if normalized_scope == "**" || normalized_scope == "*" || normalized_scope == "." {
            within_scope = true;
        } else {
            let scope_prefix = normalized_scope.trim_end_matches("/**").trim_end_matches("/*").trim_end_matches('*');
            if normalized_target.starts_with(scope_prefix) {
                within_scope = true;
            }
        }

        let mut denied_violations = Vec::new();

        // 2. Check Explicit Deny Rules
        for deny_rule in &agent.deny_patterns {
            let clean_rule = deny_rule.trim();

            // Check modify(...) or write(...) file path denials
            if clean_rule.starts_with("modify(") && clean_rule.ends_with(')') {
                let denied_path = &clean_rule[7..clean_rule.len() - 1].replace('\\', "/");
                let denied_prefix = denied_path.trim_end_matches("/**").trim_end_matches("/*").trim_end_matches('*');
                if normalized_target.starts_with(denied_prefix) {
                    denied_violations.push(format!("Access Denied: Agent `{}` is strictly forbidden from modifying `{}` (Deny Rule: `{}`)", agent.name, target_file, deny_rule));
                }
            } else if clean_rule.starts_with("deny_path:") {
                let denied_path = clean_rule.trim_start_matches("deny_path:").trim().replace('\\', "/");
                if normalized_target.starts_with(&denied_path) {
                    denied_violations.push(format!("Access Denied: Path `{}` is restricted by policy `{}`", target_file, deny_rule));
                }
            } else if clean_rule == requested_action {
                denied_violations.push(format!("Action Denied: Agent `{}` is forbidden from performing `{}`", agent.name, requested_action));
            }

            // Check capability restrictions
            if let Some(cap) = requested_capability {
                if clean_rule == cap || clean_rule == format!("capability:{}", cap) {
                    denied_violations.push(format!("Capability Denied: Agent `{}` lacks elevation for restricted capability `{}`", agent.name, cap));
                }
            }
        }

        // 3. Check Action Allow List
        let action_allowed = agent.allow_actions.iter().any(|a| a == "*" || a == requested_action);
        if !action_allowed {
            denied_violations.push(format!("Action Not Granted: Requested action `{}` is not in agent allow list: {:?}", requested_action, agent.allow_actions));
        }

        if !within_scope {
            denied_violations.push(format!("Out of Scope: Target file `{}` is outside the permitted scope `{}`", target_file, agent.scope_pattern));
        }

        let is_auth = within_scope && denied_violations.is_empty();

        let status_msg = if is_auth {
            format!("✔ Permitted: Agent `{}` is authorized to perform `{}` on `{}`", agent.name, requested_action, target_file)
        } else {
            format!("✖ Access Blocked: Agent `{}` violates {} permission/scope boundaries", agent.name, denied_violations.len())
        };

        AgentScopeAuditReport {
            agent_name: agent.name.clone(),
            target_file: target_file.to_string(),
            requested_action: requested_action.to_string(),
            is_authorized: is_auth,
            within_scope,
            granted_permissions: agent.allow_actions.clone(),
            denied_violations,
            status_message: status_msg,
        }
    }
}
