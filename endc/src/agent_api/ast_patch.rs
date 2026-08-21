use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AstPatchAction {
    #[serde(rename = "update_param_type")]
    UpdateParamType {
        function: String,
        param: String,
        new_type: String,
    },
    #[serde(rename = "update_return_type")]
    UpdateReturnType {
        function: String,
        new_type: String,
    },
    #[serde(rename = "replace_function")]
    ReplaceFunction {
        function: String,
        new_code: String,
    },
    #[serde(rename = "add_directive")]
    AddDirective {
        target_symbol: String,
        directive: String,
    },
    #[serde(rename = "replace_pattern")]
    ReplacePattern {
        find: String,
        replace: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstPatchReport {
    pub status: String,
    pub action: String,
    pub target: String,
    pub original_lines_count: usize,
    pub patched_lines_count: usize,
    pub is_valid: bool,
    pub patched_source: String,
}

pub struct StructuredAstPatcher;

impl StructuredAstPatcher {
    pub fn apply_patch_json(source: &str, patch_json: &str) -> Result<AstPatchReport, String> {
        let action: AstPatchAction = serde_json::from_str(patch_json)
            .map_err(|e| format!("Invalid AST patch JSON schema: {}", e))?;
        Self::apply_patch(source, &action)
    }

    pub fn apply_patch(source: &str, patch: &AstPatchAction) -> Result<AstPatchReport, String> {
        let mut patched = source.to_string();
        let target_name: String;
        let action_name: String;

        match patch {
            AstPatchAction::UpdateParamType { function, param, new_type } => {
                target_name = format!("{}::{}", function, param);
                action_name = "update_param_type".to_string();

                let mut modified = false;
                let lines: Vec<&str> = patched.lines().collect();
                let mut new_lines = Vec::new();

                for line in lines {
                    if line.contains(&format!("fn {}", function)) || (line.contains("fn ") && line.contains(function)) {
                        // Look for `param: old_type`
                        let pattern = format!("{}: ", param);
                        if let Some(pos) = line.find(&pattern) {
                            let after_param = &line[pos + pattern.len()..];
                            let type_end = after_param.find(|c: char| c == ',' || c == ')' || c.is_whitespace()).unwrap_or(after_param.len());
                            let old_type = &after_param[..type_end];

                            let replaced_line = line.replace(
                                &format!("{}: {}", param, old_type),
                                &format!("{}: {}", param, new_type)
                            );
                            new_lines.push(replaced_line);
                            modified = true;
                            continue;
                        }
                    }
                    new_lines.push(line.to_string());
                }

                if !modified {
                    return Err(format!("Could not locate function '{}' with parameter '{}'", function, param));
                }
                patched = new_lines.join("\n");
            }
            AstPatchAction::UpdateReturnType { function, new_type } => {
                target_name = function.clone();
                action_name = "update_return_type".to_string();

                let mut modified = false;
                let lines: Vec<&str> = patched.lines().collect();
                let mut new_lines = Vec::new();

                for line in lines {
                    if line.contains(&format!("fn {}", function)) || (line.contains("fn ") && line.contains(function)) {
                        if let Some(paren_close) = line.rfind(')') {
                            if let Some(brace_open) = line.rfind('{') {
                                if brace_open > paren_close {
                                    let before = &line[..paren_close + 1];
                                    let after = &line[brace_open..];
                                    let replaced_line = format!("{} {} {}", before, new_type, after);
                                    new_lines.push(replaced_line);
                                    modified = true;
                                    continue;
                                }
                            }
                        }
                    }
                    new_lines.push(line.to_string());
                }

                if !modified {
                    return Err(format!("Could not locate function signature for '{}'", function));
                }
                patched = new_lines.join("\n");
            }
            AstPatchAction::ReplaceFunction { function, new_code } => {
                target_name = function.clone();
                action_name = "replace_function".to_string();

                let fn_pattern = format!("fn {}", function);
                let pub_fn_pattern = format!("pub fn {}", function);

                let lines: Vec<&str> = patched.lines().collect();
                let mut start_idx = None;
                let mut end_idx = None;
                let mut depth = 0;
                let mut in_fn = false;

                for (i, line) in lines.iter().enumerate() {
                    if !in_fn && (line.contains(&fn_pattern) || line.contains(&pub_fn_pattern)) {
                        start_idx = Some(i);
                        in_fn = true;
                    }

                    if in_fn {
                        depth += line.matches('{').count();
                        depth -= line.matches('}').count();
                        if depth == 0 && line.contains('}') {
                            end_idx = Some(i);
                            break;
                        }
                    }
                }

                if let (Some(s), Some(e)) = (start_idx, end_idx) {
                    let mut result_lines = Vec::new();
                    result_lines.extend_from_slice(&lines[..s]);
                    result_lines.push(new_code.trim());
                    result_lines.extend_from_slice(&lines[e + 1..]);
                    patched = result_lines.join("\n");
                } else {
                    return Err(format!("Could not isolate complete function body for '{}'", function));
                }
            }
            AstPatchAction::AddDirective { target_symbol, directive } => {
                target_name = target_symbol.clone();
                action_name = "add_directive".to_string();

                let lines: Vec<&str> = patched.lines().collect();
                let mut new_lines = Vec::new();
                let mut added = false;

                let dir_str = if directive.starts_with('@') { directive.clone() } else { format!("@{}", directive) };

                for line in lines {
                    if !added && (line.contains(&format!("fn {}", target_symbol)) || line.contains(&format!("st {}", target_symbol))) {
                        new_lines.push(dir_str.clone());
                        added = true;
                    }
                    new_lines.push(line.to_string());
                }

                if !added {
                    return Err(format!("Could not locate target symbol '{}' to attach directive", target_symbol));
                }
                patched = new_lines.join("\n");
            }
            AstPatchAction::ReplacePattern { find, replace } => {
                target_name = find.clone();
                action_name = "replace_pattern".to_string();
                if !patched.contains(find) {
                    return Err(format!("Pattern '{}' not found in source", find));
                }
                patched = patched.replace(find, replace);
            }
        }

        Ok(AstPatchReport {
            status: "success".to_string(),
            action: action_name,
            target: target_name,
            original_lines_count: source.lines().count(),
            patched_lines_count: patched.lines().count(),
            is_valid: true,
            patched_source: patched,
        })
    }
}
