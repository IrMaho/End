use colored::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast;
use crate::diagnostics::Diagnostic;
use crate::lexer::Lexer;
use crate::parser::Parser as EndParser;
use crate::semantic::analyzer::SemanticAnalyzer;

pub fn parse_file_line(target: &str) -> (PathBuf, usize) {
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        eprintln!("{} Target must be in format <file.end>:<line_number>", "Error:".red().bold());
        std::process::exit(1);
    }
    (PathBuf::from(parts[0]), parts[1].parse().unwrap_or(0))
}

pub fn find_all_end_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dirname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Skip hidden folders and target/dist/build
                if !dirname.starts_with('.') && dirname != "target" && dirname != "dist" && dirname != "build" && dirname != "ui_build" {
                    files.extend(find_all_end_files(&path));
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("end") {
                files.push(path);
            }
        }
    }
    files
}

pub fn resolve_import_file(base_dir: &std::path::Path, path_str: &str) -> Option<PathBuf> {
    // 1. Direct path relative to base_dir
    let direct = base_dir.join(path_str);
    if direct.exists() && direct.is_file() {
        return Some(direct);
    }
    let with_ext = base_dir.join(format!("{}.end", path_str.trim_end_matches(".end")));
    if with_ext.exists() && with_ext.is_file() {
        return Some(with_ext);
    }

    // 2. Direct path relative to workspace root
    let root_direct = std::path::Path::new(path_str);
    if root_direct.exists() && root_direct.is_file() {
        return Some(root_direct.to_path_buf());
    }
    let root_with_ext = std::path::Path::new(&format!("{}.end", path_str.trim_end_matches(".end"))).to_path_buf();
    if root_with_ext.exists() && root_with_ext.is_file() {
        return Some(root_with_ext);
    }

    // 3. Dot notation: modules.hardware -> modules/hardware.end
    let stripped = path_str.trim_end_matches(".end");
    let dot_path = stripped.replace('.', "/").replace("::", "/");
    let dot_file = base_dir.join(format!("{}.end", dot_path));
    if dot_file.exists() && dot_file.is_file() {
        return Some(dot_file);
    }

    // 4. Workspace std root check
    let std_candidate = std::path::Path::new("std").join(format!("{}.end", dot_path.trim_start_matches("std/")));
    if std_candidate.exists() && std_candidate.is_file() {
        return Some(std_candidate);
    }
    None
}

pub fn load_and_analyze(file: &PathBuf) -> Result<(ast::Module, SemanticAnalyzer), String> {
    let mut visited = std::collections::HashSet::new();
    let mut merged_module = ast::Module {
        name: "main".to_string(),
        imports: Vec::new(),
        enums: Vec::new(),
        structs: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
        modules: Vec::new(),
        extensions: Vec::new(),
        features: Vec::new(),
        contracts: Vec::new(),
        architecture_templates: Vec::new(),
        architecture_rules: Vec::new(),
        feature_migrations: Vec::new(),
        statements: Vec::new(),
        span: ast::Span::new(file.to_string_lossy().to_string(), 1, 1),
    };

    let mut full_source = String::new();
    load_module_recursive(file, &mut merged_module, &mut full_source, &mut visited)?;

    let file_str = file.to_string_lossy().to_string();
    let mut analyzer = SemanticAnalyzer::new(&file_str, &full_source);
    if let Err(errs) = analyzer.analyze_module(&merged_module) {
        for err in &errs {
            let mut diag = Diagnostic::error(&err.code, &err.message, &file_str, err.line, err.col);
            if let Some(ref h) = err.repair_suggestion {
                diag = diag.with_help(h);
            }
            eprintln!("{}", diag.render(&full_source));
        }
        return Err(format!("Found {} semantic errors", errs.len()));
    }

    Ok((merged_module, analyzer))
}

pub fn load_module_recursive(
    file: &PathBuf,
    merged: &mut ast::Module,
    full_source: &mut String,
    visited: &mut std::collections::HashSet<PathBuf>,
) -> Result<(), String> {
    let canonical = file.canonicalize().unwrap_or_else(|_| file.clone());
    if visited.contains(&canonical) {
        return Ok(());
    }
    visited.insert(canonical);

    let file_str = file.to_string_lossy().to_string();
    let source = fs::read_to_string(file).map_err(|e| format!("Failed to read file '{}': {}", file_str, e))?;
    full_source.push_str(&source);
    full_source.push('\n');

    let mut lexer = Lexer::new(&file_str, &source);
    let tokens = match lexer.tokenize_all() {
        Ok(t) => t,
        Err(e) => {
            let diag = Diagnostic::error("E0001", &e, &file_str, 1, 1);
            eprintln!("{}", diag.render(&source));
            return Err(format!("Lexing failed for '{}'", file_str));
        }
    };

    let mut parser = EndParser::new(&file_str, tokens);
    let module = match parser.parse_module("main") {
        Ok(m) => m,
        Err(e) => {
            let diag = Diagnostic::error("E0100", &e, &file_str, parser.current_span().line, parser.current_span().col);
            eprintln!("{}", diag.render(&source));
            return Err(format!("Parsing failed for '{}'", file_str));
        }
    };

    // end.config.toml rules are enforced during `end lint` and `end build`

    let base_dir = file.parent().unwrap_or_else(|| std::path::Path::new("."));

    // Process file imports
    for imp in &module.imports {
        merged.imports.push(imp.clone());
        if let ast::ImportKind::Standard = imp.kind {
            if let Some(child_path) = resolve_import_file(base_dir, &imp.path) {
                load_module_recursive(&child_path, merged, full_source, visited)?;
            }
        }
    }

    merged.enums.extend(module.enums);
    merged.structs.extend(module.structs);
    merged.traits.extend(module.traits);
    merged.impls.extend(module.impls);
    merged.functions.extend(module.functions);
    merged.modules.extend(module.modules);
    merged.extensions.extend(module.extensions);
    merged.statements.extend(module.statements);

    Ok(())
}

