use crate::ast::{Block, FunctionParam, Type};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Value {
    Void,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct(String, HashMap<String, Value>),
    Enum(Option<String>, String, Option<Box<Value>>),
    Pointer(usize),
    Array(Vec<Value>),
    Operation {
        name: Option<String>,
        params: Vec<FunctionParam>,
        return_type: Type,
        requires: Vec<String>,
        guarantees: Vec<String>,
        effects: Vec<String>,
        emits: Vec<String>,
        version: Option<usize>,
        body: Block,
    },
    ComposedOp(Box<Value>, Box<Value>),
    RepeatedOp(Box<Value>, usize, bool),
    AlternativeOp(Box<Value>, Box<Value>),
    ParallelOp(Box<Value>, Box<Value>),
    OperationResult {
        output: Box<Value>,
        status: String,
        duration_ns: u64,
        events: Vec<String>,
        logs: Vec<String>,
        effects: Vec<String>,
        errors: Vec<String>,
    },
    Event {
        name: String,
        data: HashMap<String, Value>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Struct(name, fields) => {
                write!(f, "{} {{ ", name)?;
                for (k, v) in fields {
                    write!(f, "{}: {}, ", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Enum(ename, vname, payload) => {
                if let Some(en) = ename {
                    write!(f, "{}.{}", en, vname)?;
                } else {
                    write!(f, ".{}", vname)?;
                }
                if let Some(p) = payload {
                    write!(f, "({})", p)?;
                }
                Ok(())
            }
            Value::Pointer(p) => write!(f, "*0x{:x}", p),
            Value::Array(items) => write!(f, "[{:?}]", items),
            Value::Operation { name, .. } => write!(f, "operation<{}>", name.as_deref().unwrap_or("anon")),
            Value::ComposedOp(op1, op2) => write!(f, "({} >> {})", op1, op2),
            Value::RepeatedOp(op, n, retry) => write!(f, "({} * {} (retry={}))", op, n, retry),
            Value::AlternativeOp(op1, op2) => write!(f, "({} | {})", op1, op2),
            Value::ParallelOp(op1, op2) => write!(f, "({} & {})", op1, op2),
            Value::OperationResult { output, status, duration_ns, .. } => {
                write!(f, "OperationResult(status: {}, output: {}, duration: {}ns)", status, output, duration_ns)
            }
            Value::Event { name, .. } => write!(f, "event<{}>", name),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillDefState {
    pub name: String,
    pub rules: Vec<String>,
    pub constraints: Vec<String>,
    pub structural: Vec<String>,
    pub semantic: Vec<String>,
    pub behavioral: Vec<String>,
    pub architectural: Vec<String>,
    pub performance: Vec<String>,
    pub security: Vec<String>,
    pub testing: Vec<String>,
    pub agent: Vec<String>,
    pub requires: Vec<String>,
    pub hard: Vec<String>,
    pub soft: Vec<String>,
    pub for_scope: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskState {
    pub name: String,
    pub owner: String,
    pub status: String,
    pub requirement: Option<String>,
    pub implementation: Option<String>,
    pub skills: Vec<String>,
    pub change_budget: Vec<String>,
    pub evidence: Vec<(String, String)>,
    pub result: Option<String>,
    pub confidence: Option<f64>,
    pub summary: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TodoState {
    pub id: String,
    pub implement: String,
    pub requires: Vec<String>,
    pub verify: Vec<String>,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentReportState {
    pub task_id: String,
    pub summary: String,
    pub completed: usize,
    pub unresolved: usize,
    pub risks: usize,
    pub confidence: f64,
}
