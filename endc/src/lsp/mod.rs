use crate::lexer::Lexer;
use crate::parser::Parser as EndParser;
use crate::semantic::SemanticAnalyzer;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

#[derive(Debug, Serialize, Deserialize)]
struct LspRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

pub struct LanguageServer {
    documents: std::collections::HashMap<String, String>,
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

    fn handle_request(&mut self, req: &LspRequest) -> Option<Value> {
        match req.method.as_str() {
            "initialize" => {
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1, // Full sync
                            "hoverProvider": true,
                            "definitionProvider": true
                        },
                        "serverInfo": {
                            "name": "endc-lsp",
                            "version": "0.1.0"
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
                            }
                        }
                    }
                }
                None
            }
            "textDocument/hover" => {
                if let Some(params) = &req.params {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or_default();
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize + 1;

                    if let Some(source) = self.documents.get(uri) {
                        let mut lexer = Lexer::new(uri, source);
                        if let Ok(tokens) = lexer.tokenize_all() {
                            let mut parser = EndParser::new(uri, tokens);
                            if let Ok(module) = parser.parse_module("main") {
                                let mut analyzer = SemanticAnalyzer::new(uri, source);
                                let _ = analyzer.analyze_module(&module);
                                if let Some(line_sem) = analyzer.graph.inspect_line(line) {
                                    let content = format!(
                                        "### 👑 End Semantic Introspection (Line {})\n```end\n{}\n```\n- **Memory Allocated:** {}\n- **IO Performed:** {}\n- **Side Effects:** {:?}",
                                        line, line_sem.code, line_sem.side_effects.memory_allocated, line_sem.side_effects.io_performed, line_sem.side_effects.effects
                                    );
                                    return Some(json!({
                                        "jsonrpc": "2.0",
                                        "id": req.id,
                                        "result": {
                                            "contents": {
                                                "kind": "markdown",
                                                "value": content
                                            }
                                        }
                                    }));
                                }
                            }
                        }
                    }
                }
                Some(json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "result": null
                }))
            }
            _ => None,
        }
    }
}
