use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: usize,
    pub line: usize,
    pub verified: bool,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: usize,
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub file: String,
}

pub struct DapServer {
    breakpoints: HashMap<String, Vec<Breakpoint>>,
    current_frame: usize,
    is_running: bool,
}

impl DapServer {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            current_frame: 0,
            is_running: false,
        }
    }

    pub fn handle_dap_request(&mut self, request: &Value) -> Value {
        let command = request["command"].as_str().unwrap_or("");
        let seq = request["seq"].as_i64().unwrap_or(1);

        match command {
            "initialize" => json!({
                "seq": seq,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "initialize",
                "body": {
                    "supportsConfigurationDoneRequest": true,
                    "supportsFunctionBreakpoints": true,
                    "supportsConditionalBreakpoints": true,
                    "supportsEvaluateForHovers": true,
                    "supportsStepBack": false,
                    "supportsSetVariable": true
                }
            }),
            "setBreakpoints" => {
                let file = request["arguments"]["source"]["path"].as_str().unwrap_or("").to_string();
                let lines = request["arguments"]["lines"].as_array();
                let mut bps = Vec::new();
                if let Some(arr) = lines {
                    for (i, l) in arr.iter().enumerate() {
                        let line_num = l.as_u64().unwrap_or(1) as usize;
                        bps.push(Breakpoint {
                            id: i + 1,
                            line: line_num,
                            verified: true,
                            source_path: file.clone(),
                        });
                    }
                }
                self.breakpoints.insert(file, bps.clone());
                json!({
                    "seq": seq,
                    "type": "response",
                    "request_seq": seq,
                    "success": true,
                    "command": "setBreakpoints",
                    "body": {
                        "breakpoints": bps
                    }
                })
            }
            "threads" => json!({
                "seq": seq,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "threads",
                "body": {
                    "threads": [
                        { "id": 1, "name": "End Main Worker Fiber" }
                    ]
                }
            }),
            "stackTrace" => json!({
                "seq": seq,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "stackTrace",
                "body": {
                    "stackFrames": [
                        {
                            "id": 1,
                            "name": "main",
                            "line": 1,
                            "column": 1,
                            "source": { "name": "app.end", "path": "app.end" }
                        }
                    ],
                    "totalFrames": 1
                }
            }),
            "scopes" => json!({
                "seq": seq,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "scopes",
                "body": {
                    "scopes": [
                        { "name": "Locals", "variablesReference": 1, "expensive": false },
                        { "name": "Globals", "variablesReference": 2, "expensive": false }
                    ]
                }
            }),
            "variables" => json!({
                "seq": seq,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "variables",
                "body": {
                    "variables": [
                        { "name": "counter", "value": "42", "type": "i64", "variablesReference": 0 }
                    ]
                }
            }),
            _ => json!({
                "seq": seq,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": command
            }),
        }
    }
}
