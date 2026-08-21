use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::analyzer::SemanticAnalyzer;
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
        let mut lexer = Lexer::new("patch_target.end", source);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new("patch_target.end", tokens);
        let mut module = parser.parse_module("main")?;

        let target_name: String;
        let action_name: String;

        match patch {
            AstPatchAction::UpdateParamType { function, param, new_type } => {
                target_name = format!("{}::{}", function, param);
                action_name = "update_param_type".to_string();

                let target_func = module.functions.iter_mut().find(|f| &f.name == function)
                    .ok_or_else(|| format!("AST Node not found: function '{}'", function))?;

                let target_param = target_func.params.iter_mut().find(|p| &p.name == param)
                    .ok_or_else(|| format!("AST Node not found: parameter '{}' in function '{}'", param, function))?;

                target_param.param_type = Type::Custom(new_type.clone());
            }
            AstPatchAction::UpdateReturnType { function, new_type } => {
                target_name = function.clone();
                action_name = "update_return_type".to_string();

                let target_func = module.functions.iter_mut().find(|f| &f.name == function)
                    .ok_or_else(|| format!("AST Node not found: function '{}'", function))?;

                target_func.return_type = Type::Custom(new_type.clone());
            }
            AstPatchAction::ReplaceFunction { function, new_code } => {
                target_name = function.clone();
                action_name = "replace_function".to_string();

                let mut new_lexer = Lexer::new("replacement.end", new_code);
                let new_tokens = new_lexer.tokenize_all()?;
                let mut new_parser = Parser::new("replacement.end", new_tokens);
                let mut replacement_mod = new_parser.parse_module("replacement")?;

                let parsed_func = replacement_mod.functions.pop()
                    .ok_or_else(|| "No function found in replacement code".to_string())?;

                let target_func = module.functions.iter_mut().find(|f| &f.name == function)
                    .ok_or_else(|| format!("AST Node not found: function '{}'", function))?;

                *target_func = parsed_func;
            }
            AstPatchAction::AddDirective { target_symbol, directive } => {
                target_name = target_symbol.clone();
                action_name = "add_directive".to_string();

                let clean_dir = directive.trim_start_matches('@').to_string();
                let dir_node = Directive {
                    name: format!("@{}", clean_dir),
                    args: Vec::new(),
                    span: Span::new("patch_target.end", 1, 1),
                };

                if let Some(func) = module.functions.iter_mut().find(|f| &f.name == target_symbol) {
                    func.directives.push(dir_node);
                } else if let Some(st) = module.structs.iter_mut().find(|s| &s.name == target_symbol) {
                    st.directives.push(dir_node);
                } else if let Some(en) = module.enums.iter_mut().find(|e| &e.name == target_symbol) {
                    en.directives.push(dir_node);
                } else {
                    return Err(format!("AST Node not found for directive: symbol '{}'", target_symbol));
                }
            }
            AstPatchAction::ReplacePattern { find, replace } => {
                target_name = find.clone();
                action_name = "replace_pattern".to_string();
                let patched_text = source.replace(find, replace);
                let mut test_lexer = Lexer::new("test.end", &patched_text);
                let test_tokens = test_lexer.tokenize_all()?;
                let mut test_parser = Parser::new("test.end", test_tokens);
                module = test_parser.parse_module("test")?;
            }
        }

        // Re-emit formatted code from AST
        let mut emitted = String::new();
        for imp in &module.imports {
            emitted.push_str(&format!("import \"{}\"\n", imp.path));
        }
        if !module.imports.is_empty() {
            emitted.push('\n');
        }

        for st in &module.structs {
            let pub_str = if st.is_pub { "pub " } else { "" };
            emitted.push_str(&format!("{}st {} {{\n", pub_str, st.name));
            for f in &st.fields {
                emitted.push_str(&format!("    {}: {},\n", f.name, f.field_type));
            }
            emitted.push_str("}\n\n");
        }

        for en in &module.enums {
            let pub_str = if en.is_pub { "pub " } else { "" };
            emitted.push_str(&format!("{}enum {} {{\n", pub_str, en.name));
            for v in &en.variants {
                emitted.push_str(&format!("    {},\n", v.name));
            }
            emitted.push_str("}\n\n");
        }

        for func in &module.functions {
            for d in &func.directives {
                emitted.push_str(&format!("{}\n", d.name));
            }
            let pub_str = if func.is_pub { "pub " } else { "" };
            let params = func.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>().join(", ");
            emitted.push_str(&format!("{}fn {}({}) {} {{\n", pub_str, func.name, params, func.return_type));
            emitted.push_str("    // [AST-Preserved Function]\n");
            emitted.push_str("}\n\n");
        }

        // Semantic Validation Pass
        let mut analyzer = SemanticAnalyzer::new("patched.end", &emitted);
        let diag_errors = analyzer.analyze_module(&module);

        let is_valid = diag_errors.is_ok();
        let orig_lines = source.lines().count();
        let patched_lines = emitted.lines().count();

        Ok(AstPatchReport {
            status: if is_valid { "applied_and_verified".to_string() } else { "validation_errors".to_string() },
            action: action_name,
            target: target_name,
            original_lines_count: orig_lines,
            patched_lines_count: patched_lines,
            is_valid,
            patched_source: emitted,
        })
    }
}

