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
                            "definitionProvider": true,
                            "documentSymbolProvider": true,
                            "completionProvider": {
                                "resolveProvider": false,
                                "triggerCharacters": [".", ":", "@"]
                            }
                        },
                        "serverInfo": {
                            "name": "endc-lsp",
                            "version": "0.4.0-enterprise"
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
                    ("atomic", "Hardware Atomic Block", "atomic {\n    ${0}\n}"),
                    ("import", "Module Import", "import \"${1:path}\""),
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
                let builtins = [
                    ("println", "fn println(val: Any) void", "Print to stdout with newline"),
                    ("print", "fn print(val: Any) void", "Print to stdout without newline"),
                    ("panic", "fn panic(msg: str) void", "Terminate execution with panic"),
                    ("assert", "fn assert(cond: bool) void", "Runtime assertion check"),
                    ("sizeof", "fn sizeof(T) usize", "Compile-time type size inspection"),
                    ("typeof", "fn typeof(expr) str", "Introspective type string query"),
                    ("db_open", "fn db_open(path: str) DbConnection", "Open embedded high-speed database"),
                    ("tcp_listener_bind", "fn tcp_listener_bind(port: i32) TcpListener", "Bind native TCP server"),
                    ("tcp_listener_set_nonblocking", "fn tcp_listener_set_nonblocking(listener: TcpListener, nonblock: bool) bool", "Set socket non-blocking mode"),
                    ("event_loop_create", "fn event_loop_create() EventLoop", "Create async non-blocking event loop"),
                    ("event_loop_poll", "fn event_loop_poll(ev_loop: EventLoop, timeout_ms: i32) i32", "Poll ready socket descriptors"),
                    ("mpsc_create", "fn mpsc_create(capacity: i32) MpscQueue", "Create thread-safe MPSC ring buffer channel"),
                    ("mpsc_send", "fn mpsc_send(chan: MpscQueue, item: str) bool", "Send item to MPSC channel"),
                    ("mpsc_recv", "fn mpsc_recv(chan: MpscQueue) str", "Receive item from MPSC channel"),
                    ("json_parse", "fn json_parse(raw: str) JsonObject", "Parse RFC 8259 JSON string"),
                    ("json_get_string", "fn json_get_string(json: str, key: str) str", "Query string key from JSON"),
                    ("json_get_int", "fn json_get_int(json: str, key: str) i64", "Query int key from JSON"),
                    ("sha256_hash", "fn sha256_hash(data: str) str", "Compute cryptographic SHA-256 hex digest"),
                    ("hmac_sha256_sign", "fn hmac_sha256_sign(key: str, data: str) str", "Compute HMAC-SHA256 signature"),
                    ("base64_encode", "fn base64_encode(data: str) str", "Encode data to standard Base64"),
                    ("base64url_encode", "fn base64url_encode(data: str) str", "Encode data to URL-safe Base64"),
                    ("jwt_sign_hs256", "fn jwt_sign_hs256(sub: str, exp: i64, secret: str) str", "Generate authentic HS256 JWT Token"),
                    ("jwt_verify", "fn jwt_verify(token: str, secret: str) JwtValidationResult", "Verify and inspect JWT Token"),
                    ("tls_connect", "fn tls_connect(stream: TcpStream, host: str) TlsSession", "Establish native TLS 1.3 Client session"),
                    ("tls_accept", "fn tls_accept(stream: TcpStream, cert: str, key: str) TlsSession", "Accept native TLS 1.3 Server session"),
                    ("acme_create_order", "fn acme_create_order(domain: str, tok: str, thumb: str) AcmeCertificateOrder", "Create automated ACME certificate order"),
                    ("tensor_create", "fn tensor_create(rows: i32, cols: i32) Tensor", "Allocate hardware-accelerated 2D Tensor"),
                    ("tensor_matmul", "fn tensor_matmul(a: Tensor, b: Tensor) Tensor", "SIMD cache-optimized matrix multiplication"),
                    ("tensor_relu", "fn tensor_relu(t: Tensor) void", "In-place ReLU activation function"),
                    ("gguf_parse_header", "fn gguf_parse_header(magic: i64, ver: i32, tensors: i64, kv: i64) GgufHeader", "Parse GGUF model header"),
                    ("llm_format_request_json", "fn llm_format_request_json(model: str, prompt: str, tokens: i32, stream: bool) str", "Format streaming LLM JSON payload"),
                    ("canvas_create", "fn canvas_create(width: i32, height: i32) Canvas", "Create 120 FPS direct canvas compositor"),
                    ("canvas_draw_rect", "fn canvas_draw_rect(c: Canvas, r: Rect, col: Color) void", "Render filled rectangle on canvas"),
                    ("canvas_draw_circle", "fn canvas_draw_circle(c: Canvas, cx: i32, cy: i32, radius: i32, col: Color) void", "Render anti-aliased circle on canvas"),
                    ("gpu_create_buffer", "fn gpu_create_buffer(size_bytes: i64) GpuBuffer", "Allocate Vulkan/DX12 GPU buffer"),
                    ("gpu_dispatch_compute", "fn gpu_dispatch_compute(pipeline: ComputePipeline, x: i32, y: i32, z: i32) i64", "Dispatch GPU compute workgroups"),
                    ("hyper_app_create", "fn hyper_app_create(name: str, ver: str) HyperApp", "Initialize EndHyper Web App"),
                ];

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
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize + 1;
                    let character = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    if let Some(source) = self.documents.get(uri) {
                        // Extract word at position
                        if let Some(word) = extract_word_at_pos(source, line, character) {
                            let mut lexer = Lexer::new(uri, source);
                            if let Ok(tokens) = lexer.tokenize_all() {
                                let mut parser = EndParser::new(uri, tokens);
                                if let Ok(module) = parser.parse_module("main") {
                                    for f in &module.functions {
                                        if f.name == word {
                                            return Some(json!({
                                                "jsonrpc": "2.0",
                                                "id": req.id,
                                                "result": {
                                                    "uri": uri,
                                                    "range": {
                                                        "start": { "line": f.span.line.saturating_sub(1), "character": f.span.col.saturating_sub(1) },
                                                        "end": { "line": f.span.line.saturating_sub(1), "character": f.span.col + f.name.len() }
                                                    }
                                                }
                                            }));
                                        }
                                    }
                                    for s in &module.structs {
                                        if s.name == word {
                                            return Some(json!({
                                                "jsonrpc": "2.0",
                                                "id": req.id,
                                                "result": {
                                                    "uri": uri,
                                                    "range": {
                                                        "start": { "line": s.span.line.saturating_sub(1), "character": s.span.col.saturating_sub(1) },
                                                        "end": { "line": s.span.line.saturating_sub(1), "character": s.span.col + s.name.len() }
                                                    }
                                                }
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(json!({ "jsonrpc": "2.0", "id": req.id, "result": null }))
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

