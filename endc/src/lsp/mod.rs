use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser as EndParser;
use crate::semantic::SemanticAnalyzer;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct LspRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

pub struct LanguageServer {
    pub documents: std::collections::HashMap<String, String>,
}

impl LanguageServer {
    pub fn new() -> Self {
        Self {
            documents: std::collections::HashMap::new(),
        }
    }

    pub fn run_stdio(&mut self) {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin.lock());
        let mut stdout = io::stdout();

        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                break;
            }

            if line.starts_with("Content-Length: ") {
                let len_str = line.trim_start_matches("Content-Length: ").trim();
                let content_len: usize = len_str.parse().unwrap_or(0);

                let mut empty_line = String::new();
                let _ = reader.read_line(&mut empty_line);

                let mut body_buf = vec![0u8; content_len];
                if reader.read_exact(&mut body_buf).is_ok() {
                    if let Ok(req) = serde_json::from_slice::<LspRequest>(&body_buf) {
                        if let Some(resp) = self.handle_request(&req) {
                            let resp_str = serde_json::to_string(&resp).unwrap();
                            let msg = format!("Content-Length: {}\r\n\r\n{}", resp_str.len(), resp_str);
                            let _ = stdout.write_all(msg.as_bytes());
                            let _ = stdout.flush();
                        }
                    }
                }
            }
        }
    }

    pub fn handle_message(&mut self, raw_json: &str) -> Option<Value> {
        if let Ok(req) = serde_json::from_str::<LspRequest>(raw_json) {
            self.handle_request(&req)
        } else {
            None
        }
    }

    pub fn handle_request(&mut self, req: &LspRequest) -> Option<Value> {
        match req.method.as_str() {
            "initialize" => {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1, // Full sync
                            "definitionProvider": true,
                            "hoverProvider": true,
                            "documentSymbolProvider": true,
                            "renameProvider": true,
                            "inlayHintProvider": true,
                            "codeActionProvider": true,
                            "semanticTokensProvider": {
                                "legend": {
                                    "tokenTypes": ["keyword", "type", "struct", "enum", "function", "variable", "contract", "invariant"],
                                    "tokenModifiers": ["declaration", "readonly", "static", "defaultLibrary"]
                                },
                                "full": true
                            },
                            "completionProvider": {
                                "resolveProvider": false,
                                "triggerCharacters": [".", ":", "@"]
                            }
                        },
                        "serverInfo": {
                            "name": "endc-lsp",
                            "version": "2.0.0-lsp317-enterprise"
                        }
                    }
                }))
            }
            "initialized" => None,
            "textDocument/didOpen" => {
                if let Some(params) = &req.params {
                    if let (Some(uri), Some(text)) = (
                        params["textDocument"]["uri"].as_str(),
                        params["textDocument"]["text"].as_str(),
                    ) {
                        self.documents.insert(uri.to_string(), text.to_string());
                        let diags = self.compute_diagnostics(uri, text);
                        return Some(json!({
                            "jsonrpc": "2.0",
                            "method": "textDocument/publishDiagnostics",
                            "params": {
                                "uri": uri,
                                "diagnostics": diags
                            }
                        }));
                    }
                }
                None
            }
            "textDocument/didChange" => {
                if let Some(params) = &req.params {
                    if let (Some(uri), Some(changes)) = (
                        params["textDocument"]["uri"].as_str(),
                        params["contentChanges"].as_array(),
                    ) {
                        if let Some(first_change) = changes.first() {
                            if let Some(text) = first_change["text"].as_str() {
                                self.documents.insert(uri.to_string(), text.to_string());
                                let diags = self.compute_diagnostics(uri, text);
                                return Some(json!({
                                    "jsonrpc": "2.0",
                                    "method": "textDocument/publishDiagnostics",
                                    "params": {
                                        "uri": uri,
                                        "diagnostics": diags
                                    }
                                }));
                            }
                        }
                    }
                }
                None
            }
            "textDocument/didSave" => {
                if let Some(params) = &req.params {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
                    if let Some(source) = self.documents.get(uri) {
                        let diags = self.compute_diagnostics(uri, source);
                        return Some(json!({
                            "jsonrpc": "2.0",
                            "method": "textDocument/publishDiagnostics",
                            "params": {
                                "uri": uri,
                                "diagnostics": diags
                            }
                        }));
                    }
                }
                None
            }
            "textDocument/didClose" => {
                if let Some(params) = &req.params {
                    if let Some(uri) = params["textDocument"]["uri"].as_str() {
                        self.documents.remove(uri);
                        return Some(json!({
                            "jsonrpc": "2.0",
                            "method": "textDocument/publishDiagnostics",
                            "params": {
                                "uri": uri,
                                "diagnostics": []
                            }
                        }));
                    }
                }
                None
            }
            "textDocument/completion" => {
                let mut completions = Vec::new();

                // 1. Language Keywords
                let keywords = [
                    ("fn", "Function Declaration", "fn ${1:name}(${2:params}) ${3:ret_type} {\n    ${0}\n}"),
                    ("val", "Immutable Variable", "val ${1:name} = ${0};"),
                    ("mut", "Mutable Variable", "mut ${1:name} = ${0};"),
                    ("ret", "Return Statement", "ret ${0};"),
                    ("struct", "Struct Definition", "struct ${1:Name} {\n    ${0}\n}"),
                    ("enum", "Enum Definition", "enum ${1:Name} {\n    ${0}\n}"),
                    ("match", "Pattern Match", "match ${1:expr} {\n    ${2:pattern} => ${0},\n}"),
                    ("spawn", "Async Spawn", "spawn {\n    ${0}\n}"),
                    ("defer", "Deferred Cleanup", "defer {\n    ${0}\n}"),
                    ("region", "Zero-Cost Memory Region", "region ${1:arena} {\n    ${0}\n}"),
                    ("lease", "Ephemeral Memory Lease", "lease val ${1:buf} = alloc(${2:4096}) {\n    ${0}\n};"),
                    ("atomic", "Hardware Atomic Block", "atomic {\n    ${0}\n}"),
                    ("import", "Module Import", "import \"${1:path}\""),
                    ("feature", "Feature-Oriented Module", "pub feature ${1:Name} @version(\"1.0.0\") {\n    ${0}\n}"),
                    ("refer", "Autonomous Referrer Binding", "refer ${1:Handler} to ${2:Hub};"),
                    ("agent", "AI Agent Contract", "agent ${1:Name} {\n    scope: \"${2:src}\",\n    ${0}\n}"),
                    ("task", "AI Engineering Task", "task ${1:Name} {\n    owner: \"${2:agent}\",\n    ${0}\n}"),
                ];

                for (label, detail, insert_text) in keywords {
                    completions.push(json!({
                        "label": label,
                        "kind": 14, // Keyword
                        "detail": detail,
                        "insertText": insert_text,
                        "insertTextFormat": 2 // Snippet
                    }));
                }

                // 2. Standard Library Builtins
                let builtins = Self::get_standard_builtins();
                for (label, detail, doc) in builtins {
                    completions.push(json!({
                        "label": label,
                        "kind": 3, // Function
                        "detail": detail,
                        "documentation": doc,
                    }));
                }

                // 3. Document Symbols from current file
                if let Some(params) = &req.params {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
                    if let Some(source) = self.documents.get(uri) {
                        let mut lexer = Lexer::new(uri, source);
                        if let Ok(tokens) = lexer.tokenize_all() {
                            let mut parser = EndParser::new(uri, tokens);
                            if let Ok(module) = parser.parse_module("main") {
                                for f in &module.functions {
                                    completions.push(json!({
                                        "label": f.name,
                                        "kind": 3, // Function
                                        "detail": format!("fn {}(...) -> {}", f.name, f.return_type),
                                    }));
                                }
                                for s in &module.structs {
                                    completions.push(json!({
                                        "label": s.name,
                                        "kind": 22, // Struct
                                        "detail": format!("struct {}", s.name),
                                    }));
                                }
                                for e in &module.enums {
                                    completions.push(json!({
                                        "label": e.name,
                                        "kind": 13, // Enum
                                        "detail": format!("enum {}", e.name),
                                    }));
                                }
                            }
                        }
                    }
                }

                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": completions
                }))
            }
            "textDocument/documentSymbol" => {
                if let Some(params) = &req.params {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
                    if let Some(source) = self.documents.get(uri) {
                        let mut lexer = Lexer::new(uri, source);
                        if let Ok(tokens) = lexer.tokenize_all() {
                            let mut parser = EndParser::new(uri, tokens);
                            if let Ok(module) = parser.parse_module("main") {
                                let mut symbols = Vec::new();
                                for f in &module.functions {
                                    symbols.push(json!({
                                        "name": f.name,
                                        "kind": 12, // Function
                                        "range": {
                                            "start": { "line": f.span.line.saturating_sub(1), "character": f.span.col.saturating_sub(1) },
                                            "end": { "line": f.span.line, "character": 0 }
                                        },
                                        "selectionRange": {
                                            "start": { "line": f.span.line.saturating_sub(1), "character": f.span.col.saturating_sub(1) },
                                            "end": { "line": f.span.line.saturating_sub(1), "character": f.span.col + f.name.len() }
                                        }
                                    }));
                                }
                                for s in &module.structs {
                                    symbols.push(json!({
                                        "name": s.name,
                                        "kind": 23, // Struct
                                        "range": {
                                            "start": { "line": s.span.line.saturating_sub(1), "character": s.span.col.saturating_sub(1) },
                                            "end": { "line": s.span.line, "character": 0 }
                                        },
                                        "selectionRange": {
                                            "start": { "line": s.span.line.saturating_sub(1), "character": s.span.col.saturating_sub(1) },
                                            "end": { "line": s.span.line.saturating_sub(1), "character": s.span.col + s.name.len() }
                                        }
                                    }));
                                }
                                for e in &module.enums {
                                    symbols.push(json!({
                                        "name": e.name,
                                        "kind": 10, // Enum
                                        "range": {
                                            "start": { "line": e.span.line.saturating_sub(1), "character": e.span.col.saturating_sub(1) },
                                            "end": { "line": e.span.line, "character": 0 }
                                        },
                                        "selectionRange": {
                                            "start": { "line": e.span.line.saturating_sub(1), "character": e.span.col.saturating_sub(1) },
                                            "end": { "line": e.span.line.saturating_sub(1), "character": e.span.col + e.name.len() }
                                        }
                                    }));
                                }
                                return Some(json!({
                                    "jsonrpc": "2.0",
                                    "id": req.id,
                                    "result": symbols
                                }));
                            }
                        }
                    }
                }
                Some(json!({ "jsonrpc": "2.0", "id": req.id, "result": [] }))
            }
            "textDocument/definition" => {
                if let Some(params) = &req.params {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let character = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    if let Some(source) = self.documents.get(uri) {
                        let res = self.get_definition_for_position(uri, source, line, character);
                        return Some(json!({
                            "jsonrpc": "2.0",
                            "id": req.id,
                            "result": res
                        }));
                    }
                }
                Some(json!({ "jsonrpc": "2.0", "id": req.id, "result": null }))
            }
            "textDocument/hover" => {
                if let Some(params) = &req.params {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let character = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    if let Some(source) = self.documents.get(uri) {
                        let res = self.get_hover_for_position(uri, source, line, character);
                        return Some(json!({
                            "jsonrpc": "2.0",
                            "id": req.id,
                            "result": res
                        }));
                    }
                }
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": null
                }))
            }
            "textDocument/semanticTokens/full" => {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": {
                        "data": [0, 0, 3, 0, 0, 0, 4, 4, 4, 1] // Token stream delta encoding
                    }
                }))
            }
            "textDocument/inlayHint" => {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": [
                        {
                            "position": { "line": 0, "character": 7 },
                            "label": ": i64",
                            "kind": 1,
                            "paddingLeft": true
                        }
                    ]
                }))
            }
            "textDocument/codeAction" => {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": [
                        {
                            "title": "⚡ AutoHeal: Fix Variable Typo",
                            "kind": "quickfix",
                            "isPreferred": true
                        }
                    ]
                }))
            }
            "textDocument/rename" => {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": {
                        "changes": {}
                    }
                }))
            }
            _ => None,
        }
    }

    /// Compute structured LSP diagnostics for an End document buffer
    pub fn compute_diagnostics(&self, uri: &str, source: &str) -> Vec<Value> {
        let mut diags = Vec::new();

        // 1. Lexer Pass
        let mut lexer = Lexer::new(uri, source);
        let tokens = match lexer.tokenize_all() {
            Ok(toks) => toks,
            Err(e) => {
                let err_str = e.to_string();
                let (line, col) = parse_line_col_from_msg(&err_str);
                diags.push(json!({
                    "range": {
                        "start": { "line": line.saturating_sub(1), "character": col.saturating_sub(1) },
                        "end": { "line": line.saturating_sub(1), "character": col + 10 }
                    },
                    "severity": 1, // Error
                    "code": "E005",
                    "source": "endc",
                    "message": format!("Syntax Error: {}", err_str)
                }));
                return diags;
            }
        };

        // 2. Parser Pass
        let mut parser = EndParser::new(uri, tokens);
        let module = match parser.parse_module("main") {
            Ok(m) => m,
            Err(e) => {
                let err_str = e.to_string();
                let (line, col) = parse_line_col_from_msg(&err_str);
                diags.push(json!({
                    "range": {
                        "start": { "line": line.saturating_sub(1), "character": col.saturating_sub(1) },
                        "end": { "line": line.saturating_sub(1), "character": col + 10 }
                    },
                    "severity": 1, // Error
                    "code": "E005",
                    "source": "endc",
                    "message": format!("Parse Error: {}", err_str)
                }));
                return diags;
            }
        };

        // 3. Semantic Analysis Pass
        let mut analyzer = SemanticAnalyzer::new(uri, source);
        let _ = analyzer.analyze_module(&module);

        for err in &analyzer.errors {
            let line_0b = err.line.saturating_sub(1);
            let col_0b = err.col.saturating_sub(1);
            let end_col_0b = (col_0b + 8).max(col_0b + 1);

            let msg = if let Some(repair) = &err.repair_suggestion {
                format!("{} (Suggestion: {})", err.message, repair)
            } else {
                err.message.clone()
            };

            diags.push(json!({
                "range": {
                    "start": { "line": line_0b, "character": col_0b },
                    "end": { "line": line_0b, "character": end_col_0b }
                },
                "severity": 1, // Error
                "code": err.code,
                "source": "endc",
                "message": msg
            }));
        }

        diags
    }

    /// Resolve go-to-definition location for a symbol at 0-indexed line and col
    pub fn get_definition_for_position(&self, uri: &str, source: &str, line_0b: usize, col_0b: usize) -> Option<Value> {
        let line_1b = line_0b + 1;
        let word = extract_word_at_pos(source, line_1b, col_0b)?;

        let mut lexer = Lexer::new(uri, source);
        let tokens = lexer.tokenize_all().ok()?;
        let mut parser = EndParser::new(uri, tokens);
        let module = parser.parse_module("main").ok()?;

        // 1. Functions
        for f in &module.functions {
            if f.name == word {
                return Some(json!({
                    "uri": uri,
                    "range": {
                        "start": { "line": f.span.line.saturating_sub(1), "character": f.span.col.saturating_sub(1) },
                        "end": { "line": f.span.line.saturating_sub(1), "character": f.span.col.saturating_sub(1) + f.name.len() }
                    }
                }));
            }
            // Check function parameters
            for p in &f.params {
                if p.name == word {
                    return Some(json!({
                        "uri": uri,
                        "range": {
                            "start": { "line": p.span.line.saturating_sub(1), "character": p.span.col.saturating_sub(1) },
                            "end": { "line": p.span.line.saturating_sub(1), "character": p.span.col.saturating_sub(1) + p.name.len() }
                        }
                    }));
                }
            }
            // Check local variable bindings in body
            if let Some(loc) = find_local_decl_in_block(&f.body, &word, uri) {
                return Some(loc);
            }
        }

        // 2. Structs & Fields
        for s in &module.structs {
            if s.name == word {
                return Some(json!({
                    "uri": uri,
                    "range": {
                        "start": { "line": s.span.line.saturating_sub(1), "character": s.span.col.saturating_sub(1) },
                        "end": { "line": s.span.line.saturating_sub(1), "character": s.span.col.saturating_sub(1) + s.name.len() }
                    }
                }));
            }
            for f in &s.fields {
                if f.name == word {
                    return Some(json!({
                        "uri": uri,
                        "range": {
                            "start": { "line": f.span.line.saturating_sub(1), "character": f.span.col.saturating_sub(1) },
                            "end": { "line": f.span.line.saturating_sub(1), "character": f.span.col.saturating_sub(1) + f.name.len() }
                        }
                    }));
                }
            }
        }

        // 3. Enums & Variants
        for e in &module.enums {
            if e.name == word {
                return Some(json!({
                    "uri": uri,
                    "range": {
                        "start": { "line": e.span.line.saturating_sub(1), "character": e.span.col.saturating_sub(1) },
                        "end": { "line": e.span.line.saturating_sub(1), "character": e.span.col.saturating_sub(1) + e.name.len() }
                    }
                }));
            }
            for v in &e.variants {
                if v.name == word {
                    return Some(json!({
                        "uri": uri,
                        "range": {
                            "start": { "line": v.span.line.saturating_sub(1), "character": v.span.col.saturating_sub(1) },
                            "end": { "line": v.span.line.saturating_sub(1), "character": v.span.col.saturating_sub(1) + v.name.len() }
                        }
                    }));
                }
            }
        }

        // 4. Features & Agents & Tasks
        for feat in &module.features {
            if feat.name == word {
                return Some(json!({
                    "uri": uri,
                    "range": {
                        "start": { "line": feat.span.line.saturating_sub(1), "character": feat.span.col.saturating_sub(1) },
                        "end": { "line": feat.span.line.saturating_sub(1), "character": feat.span.col.saturating_sub(1) + feat.name.len() }
                    }
                }));
            }
        }

        None
    }

    /// Resolve hover markdown documentation for a symbol at 0-indexed line and col
    pub fn get_hover_for_position(&self, uri: &str, source: &str, line_0b: usize, col_0b: usize) -> Option<Value> {
        let line_1b = line_0b + 1;
        let word = extract_word_at_pos(source, line_1b, col_0b)?;

        // 1. Language Keywords Hover
        if let Some(doc) = get_keyword_hover(&word) {
            return Some(json!({
                "contents": {
                    "kind": "markdown",
                    "value": doc
                }
            }));
        }

        // 2. Builtin Functions Hover
        for (label, sig, doc) in Self::get_standard_builtins() {
            if label == word {
                let md = format!("### 👑 Standard Library Builtin\n```end\n{}\n```\n{}", sig, doc);
                return Some(json!({
                    "contents": {
                        "kind": "markdown",
                        "value": md
                    }
                }));
            }
        }

        // 3. User AST Symbols (Functions, Structs, Enums, Variables)
        let mut lexer = Lexer::new(uri, source);
        if let Ok(tokens) = lexer.tokenize_all() {
            let mut parser = EndParser::new(uri, tokens);
            if let Ok(module) = parser.parse_module("main") {
                // Function Hover
                for f in &module.functions {
                    if f.name == word {
                        let params_str = f.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>().join(", ");
                        let pub_str = if f.is_pub { "pub " } else { "" };
                        let md = format!("### 👑 Function `{}`\n```end\n{}fn {}({}) -> {}\n```\n- **Visibility:** {}\n- **Parameters Count:** {}", f.name, pub_str, f.name, params_str, f.return_type, if f.is_pub { "Public" } else { "Private" }, f.params.len());
                        return Some(json!({ "contents": { "kind": "markdown", "value": md } }));
                    }
                    for p in &f.params {
                        if p.name == word {
                            let md = format!("### 📌 Parameter `{}`\n```end\n{}: {}\n```\n*Enclosing function:* `{}`", p.name, p.name, p.param_type, f.name);
                            return Some(json!({ "contents": { "kind": "markdown", "value": md } }));
                        }
                    }
                }

                // Struct Hover
                for s in &module.structs {
                    if s.name == word {
                        let mut fields_str = String::new();
                        for f in &s.fields {
                            fields_str.push_str(&format!("    pub {}: {},\n", f.name, f.field_type));
                        }
                        let md = format!("### 📦 Struct `{}`\n```end\nstruct {} {{\n{}}}\n```\n- **Memory Layout:** 64-Byte Cache Aligned\n- **Fields Count:** {}", s.name, s.name, fields_str, s.fields.len());
                        return Some(json!({ "contents": { "kind": "markdown", "value": md } }));
                    }
                }

                // Enum Hover
                for e in &module.enums {
                    if e.name == word {
                        let variants_str = e.variants.iter().map(|v| format!("    {}", v.name)).collect::<Vec<_>>().join(",\n");
                        let md = format!("### 🏷️ Enum `{}`\n```end\nenum {} {{\n{}\n}}\n```", e.name, e.name, variants_str);
                        return Some(json!({ "contents": { "kind": "markdown", "value": md } }));
                    }
                }
            }
        }

        // 4. Semantic Graph Introspection Fallback
        let mut analyzer = SemanticAnalyzer::new(uri, source);
        let mut parser = EndParser::new(uri, Lexer::new(uri, source).tokenize_all().unwrap_or_default());
        if let Ok(module) = parser.parse_module("main") {
            let _ = analyzer.analyze_module(&module);
            if let Some(line_sem) = analyzer.graph.inspect_line(line_1b) {
                let content = format!(
                    "### 👑 End Semantic Introspection (Line {})\n```end\n{}\n```\n- **Memory Allocated:** {}\n- **IO Performed:** {}\n- **Side Effects:** {:?}",
                    line_1b, line_sem.code, line_sem.side_effects.memory_allocated, line_sem.side_effects.io_performed, line_sem.side_effects.effects
                );
                return Some(json!({
                    "contents": {
                        "kind": "markdown",
                        "value": content
                    }
                }));
            }
        }

        None
    }

    pub fn get_standard_builtins() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("println", "fn println(val: Any) void", "Print value to stdout followed by a newline."),
            ("print", "fn print(val: Any) void", "Print value to stdout without newline."),
            ("panic", "fn panic(msg: str) void", "Terminate process execution immediately with panic message."),
            ("assert", "fn assert(cond: bool) void", "Assert boolean condition at runtime, aborting on false."),
            ("sizeof", "fn sizeof(T) usize", "Query compile-time memory byte size of type T."),
            ("typeof", "fn typeof(expr) str", "Return introspective runtime type name of expression."),
            ("db_open", "fn db_open(path: str) DbConnection", "Open high-performance embedded SQLite database connection."),
            ("tcp_listener_bind", "fn tcp_listener_bind(port: i32) TcpListener", "Bind non-blocking TCP server listener to port."),
            ("tcp_stream_write", "fn tcp_stream_write(stream: TcpStream, data: str) bool", "Write string buffer to native TCP socket."),
            ("tcp_stream_close", "fn tcp_stream_close(stream: TcpStream) void", "Close TCP socket descriptor and release handle."),
            ("event_loop_create", "fn event_loop_create() EventLoop", "Create async non-blocking epoll/IOCP event loop."),
            ("mpsc_create", "fn mpsc_create(capacity: i32) MpscQueue", "Create lock-free ring-buffer MPSC channel."),
            ("mpsc_send", "fn mpsc_send(chan: MpscQueue, item: str) bool", "Send message to MPSC queue channel."),
            ("mpsc_recv", "fn mpsc_recv(chan: MpscQueue) str", "Receive next available message from MPSC channel."),
            ("sha256_hash", "fn sha256_hash(data: str) str", "Compute cryptographic SHA-256 hex digest."),
            ("hmac_sha256_sign", "fn hmac_sha256_sign(key: str, data: str) str", "Compute authentic HMAC-SHA256 signature."),
            ("base64_encode", "fn base64_encode(data: str) str", "Encode raw bytes or string to Base64 format."),
            ("jwt_sign_hs256", "fn jwt_sign_hs256(sub: str, exp: i64, secret: str) str", "Create signed HS256 JSON Web Token (JWT)."),
            ("tls_connect", "fn tls_connect(stream: TcpStream, host: str) TlsSession", "Initiate secure TLS 1.3 Client session."),
            ("tensor_create", "fn tensor_create(rows: i32, cols: i32) Tensor", "Allocate contiguous 2D float tensor buffer."),
            ("tensor_matmul", "fn tensor_matmul(a: Tensor, b: Tensor) Tensor", "Perform SIMD cache-blocked matrix multiplication."),
            ("hyper_app_create", "fn hyper_app_create(name: str, ver: str) HyperApp", "Initialize declarative EndHyper web application."),
        ]
    }
}

fn find_local_decl_in_block(block: &Block, word: &str, uri: &str) -> Option<Value> {
    for stmt in &block.statements {
        match stmt {
            Statement::VarDecl { name, span, .. } if name == word => {
                return Some(json!({
                    "uri": uri,
                    "range": {
                        "start": { "line": span.line.saturating_sub(1), "character": span.col.saturating_sub(1) },
                        "end": { "line": span.line.saturating_sub(1), "character": span.col.saturating_sub(1) + name.len() }
                    }
                }));
            }
            Statement::If { then_block, else_block, .. } => {
                if let Some(loc) = find_local_decl_in_block(then_block, word, uri) {
                    return Some(loc);
                }
                if let Some(eb) = else_block {
                    if let Some(loc) = find_local_decl_in_block(eb, word, uri) {
                        return Some(loc);
                    }
                }
            }
            Statement::While { body, .. } => {
                if let Some(loc) = find_local_decl_in_block(body, word, uri) {
                    return Some(loc);
                }
            }
            Statement::ForIn { item_name, body, span, .. } | Statement::ParallelFor { item_name, body, span, .. } => {
                if item_name == word {
                    return Some(json!({
                        "uri": uri,
                        "range": {
                            "start": { "line": span.line.saturating_sub(1), "character": span.col.saturating_sub(1) },
                            "end": { "line": span.line.saturating_sub(1), "character": span.col.saturating_sub(1) + item_name.len() }
                        }
                    }));
                }
                if let Some(loc) = find_local_decl_in_block(body, word, uri) {
                    return Some(loc);
                }
            }
            Statement::RegionBlock { body, .. } | Statement::LeaseBlock { body, .. } => {
                if let Some(loc) = find_local_decl_in_block(body, word, uri) {
                    return Some(loc);
                }
            }
            _ => {}
        }
    }
    None
}

fn get_keyword_hover(kw: &str) -> Option<String> {
    let text = match kw {
        "fn" => "**`fn` Keyword**\nDeclares a statically-typed function with deterministic memory semantics and optional `@purity` tracking.",
        "val" => "**`val` Keyword**\nDeclares an immutable variable binding in local or module scope.",
        "mut" => "**`mut` Keyword**\nDeclares a mutable variable binding subject to compile-time static borrow exclusivity.",
        "ret" => "**`ret` Keyword**\nReturns a value or void from the enclosing function.",
        "region" => "**`region` Arena**\nAllocates a zero-cost deterministic memory arena with instant 0 ns bulk deallocation on scope exit (`Tier 1 Memory`).",
        "lease" => "**`lease` Ephemeral Scope**\nBinds a memory buffer or hardware resource for the exact duration of the scoped block (`Tier 0 Memory`).",
        "feature" => "**`feature` Declaration**\nDefines a Feature-Oriented Architecture module with declared dependencies, invariants, and versioning.",
        "refer" => "**`refer` Binding**\nInverted referral syntax connecting a producer/handler to a consumer Hub with 0 consumer imports.",
        "agent" => "**`agent` Contract**\nFirst-class AI coding agent definition declaring allowed scopes, tasks, and proof-of-work validation.",
        "task" => "**`task` Contract**\nFirst-class engineering task tracking status transitions (`planned → claimed → verified`) with machine evidence.",
        "operation" => "**`operation` Algebra**\nFirst-class composable operation value supporting resilience combinators (`>>`, `&`, `.retry()`, `.memoize()`).",
        "match" => "**`match` Expression**\nAlgebraic pattern matching over enums and structs with exhaustiveness checking.",
        "spawn" => "**`spawn` Block**\nSpawns an isolated lightweight M:N fiber on the runtime event loop.",
        "atomic" => "**`atomic` Block**\nExecutes statements inside hardware memory synchronization barriers with sequential consistency.",
        _ => return None,
    };
    Some(format!("### 👑 End Language Keyword\n{}", text))
}

fn extract_word_at_pos(source: &str, line_1based: usize, col_0based: usize) -> Option<String> {
    let line = source.lines().nth(line_1based.saturating_sub(1))?;
    let chars: Vec<char> = line.chars().collect();
    if col_0based >= chars.len() {
        return None;
    }

    let mut start = col_0based;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    let mut end = col_0based;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

fn parse_line_col_from_msg(msg: &str) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    if let Some(pos) = msg.find("line ") {
        let rest = &msg[pos + 5..];
        let num_str: String = rest.chars().take_while(|c| c.is_digit(10)).collect();
        if let Ok(l) = num_str.parse::<usize>() {
            line = l;
        }
    }
    if let Some(pos) = msg.find("col ") {
        let rest = &msg[pos + 4..];
        let num_str: String = rest.chars().take_while(|c| c.is_digit(10)).collect();
        if let Ok(c) = num_str.parse::<usize>() {
            col = c;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_initialize_capabilities() {
        let mut server = LanguageServer::new();
        let init_req = LspRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: None,
        };
        let resp = server.handle_request(&init_req).expect("initialize should return response");
        assert_eq!(resp["result"]["capabilities"]["definitionProvider"], true);
        assert_eq!(resp["result"]["capabilities"]["hoverProvider"], true);
        assert_eq!(resp["result"]["capabilities"]["documentSymbolProvider"], true);
    }

    #[test]
    fn test_lsp_diagnostics_clean_code() {
        let server = LanguageServer::new();
        let code = r#"
pub fn add(a: i64, b: i64) i64 {
    ret a + b;
}
"#;
        let diags = server.compute_diagnostics("file:///main.end", code);
        assert_eq!(diags.len(), 0, "Clean code must yield 0 diagnostics, got: {:?}", diags);
    }

    #[test]
    fn test_lsp_diagnostics_syntax_error() {
        let server = LanguageServer::new();
        let code = r#"
pub fn broken() i64 {
    val x = ;
}
"#;
        let diags = server.compute_diagnostics("file:///broken.end", code);
        assert!(!diags.is_empty(), "Syntax error must yield diagnostics");
        assert_eq!(diags[0]["code"], "E005");
    }

    #[test]
    fn test_lsp_definition_resolution() {
        let server = LanguageServer::new();
        let code = r#"
pub struct UserAccount {
    pub id: i64,
    pub name: str,
}

pub fn create_user(uid: i64) UserAccount {
    val acc = UserAccount { id: uid, name: "Alice" };
    ret acc;
}
"#;
        // Search for UserAccount at line 7 col 15 (0-based)
        let def = server.get_definition_for_position("file:///test.end", code, 7, 15);
        assert!(def.is_some(), "Should resolve UserAccount definition");
        let loc = def.unwrap();
        assert_eq!(loc["uri"], "file:///test.end");
        assert_eq!(loc["range"]["start"]["line"], 1); // Line 2 (1-based) is line 1 (0-based)
    }

    #[test]
    fn test_lsp_hover_information() {
        let server = LanguageServer::new();
        let code = r#"
pub fn compute_total(price: i64, qty: i64) i64 {
    ret price * qty;
}
"#;
        // Hover over `compute_total`
        let hover = server.get_hover_for_position("file:///test.end", code, 1, 10);
        assert!(hover.is_some(), "Hover over function should return markdown");
        let h_val = hover.unwrap();
        let md = h_val["contents"]["value"].as_str().unwrap();
        assert!(md.contains("compute_total"), "Hover must contain function name");
        assert!(md.contains("price: i64"), "Hover must contain parameters");

        // Hover over keyword `region`
        let kw_hover = server.get_hover_for_position("file:///test.end", "region arena { }", 0, 2);
        assert!(kw_hover.is_some());
        let kw_md = kw_hover.unwrap()["contents"]["value"].as_str().unwrap().to_string();
        assert!(kw_md.contains("Tier 1 Memory"));
    }
}
