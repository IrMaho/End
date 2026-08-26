use super::value::*;
use crate::ast::{EventHandlerDef, EventHubDef, FunctionDef, Module};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Interpreter {
    pub variables: Vec<HashMap<String, Value>>,
    pub functions: HashMap<String, FunctionDef>,
    pub operations: HashMap<String, Value>,
    pub event_hubs: HashMap<String, EventHubDef>,
    pub event_handlers: HashMap<String, Vec<EventHandlerDef>>,
    pub emitted_events: Vec<String>,
    pub traces: Vec<String>,
    pub memoized_cache: HashMap<String, Value>,
    pub snapshots: HashMap<String, Vec<HashMap<String, Value>>>,
    pub domain_ownership: HashMap<String, String>,
    pub features: HashMap<String, (Option<String>, Vec<String>, Vec<String>)>,
    pub skills: HashMap<String, SkillDefState>,
    pub requirements: HashMap<String, String>,
    pub requirement_implements: HashMap<String, Vec<String>>,
    pub requirement_verifies: HashMap<String, Vec<String>>,
    pub tasks_state: HashMap<String, TaskState>,
    pub todos_state: HashMap<String, TodoState>,
    pub agent_leases: HashMap<String, (String, String)>,
    pub knowledge_base: HashMap<String, (Vec<String>, Vec<String>)>,
    pub decision_records: HashMap<String, (String, String, String)>,
    pub agent_reports: Vec<AgentReportState>,
    pub verified_tasks: HashSet<String>,
    pub project_profile: HashMap<String, String>,
    pub stdout: String,
    pub capture_stdout: bool,
    pub db_engines: std::sync::Arc<std::sync::Mutex<HashMap<i64, crate::runtime::db::SqliteEngine>>>,
    pub next_db_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub pg_engines: std::sync::Arc<std::sync::Mutex<HashMap<i64, crate::runtime::db::PgEngine>>>,
    pub next_pg_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub http2_servers: std::sync::Arc<std::sync::Mutex<HashMap<i64, crate::runtime::net::Http2Server>>>,
    pub next_http2_server_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub http2_clients: std::sync::Arc<std::sync::Mutex<HashMap<i64, crate::runtime::net::Http2Client>>>,
    pub next_http2_client_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub atomics: std::sync::Arc<std::sync::Mutex<HashMap<i64, std::sync::Arc<std::sync::atomic::AtomicI64>>>>,
    pub next_atomic_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub mutexes: std::sync::Arc<std::sync::Mutex<HashMap<i64, std::sync::Arc<std::sync::Mutex<i64>>>>>,
    pub next_mutex_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub rwlocks: std::sync::Arc<std::sync::Mutex<HashMap<i64, std::sync::Arc<std::sync::RwLock<i64>>>>>,
    pub next_rwlock_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub raft_clusters: std::sync::Arc<std::sync::Mutex<HashMap<i64, std::sync::Arc<std::sync::Mutex<crate::runtime::raft::RaftCluster>>>>>,
    pub next_raft_cluster_handle: std::sync::Arc<std::sync::atomic::AtomicI64>,
    pub profiler_session: Option<std::sync::Arc<std::sync::Mutex<crate::profiler::ProfilerSession>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: vec![HashMap::new()],
            functions: HashMap::new(),
            operations: HashMap::new(),
            event_hubs: HashMap::new(),
            event_handlers: HashMap::new(),
            emitted_events: Vec::new(),
            traces: Vec::new(),
            memoized_cache: HashMap::new(),
            snapshots: HashMap::new(),
            domain_ownership: HashMap::new(),
            features: HashMap::new(),
            skills: HashMap::new(),
            requirements: HashMap::new(),
            requirement_implements: HashMap::new(),
            requirement_verifies: HashMap::new(),
            tasks_state: HashMap::new(),
            todos_state: HashMap::new(),
            agent_leases: HashMap::new(),
            knowledge_base: HashMap::new(),
            decision_records: HashMap::new(),
            agent_reports: Vec::new(),
            verified_tasks: HashSet::new(),
            project_profile: HashMap::new(),
            stdout: String::new(),
            capture_stdout: false,
            db_engines: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_db_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(100)),
            pg_engines: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_pg_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(500)),
            http2_servers: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_http2_server_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(800)),
            http2_clients: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_http2_client_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(900)),
            atomics: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_atomic_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(1000)),
            mutexes: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_mutex_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(2000)),
            rwlocks: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_rwlock_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(3000)),
            raft_clusters: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_raft_cluster_handle: std::sync::Arc::new(std::sync::atomic::AtomicI64::new(4000)),
            profiler_session: None,
        }
    }

    pub fn with_stdout_capture() -> Self {
        let mut interp = Self::new();
        interp.capture_stdout = true;
        interp
    }

    pub fn emit_stdout(&mut self, s: &str) {
        self.stdout.push_str(s);
    }

    pub fn run(&mut self, module: &Module) -> Result<Value, String> {
        for s in &module.statements {
            self.eval_statement(s)?;
        }

        for f in &module.functions {
            self.functions.insert(f.name.clone(), f.clone());
        }

        for m in &module.modules {
            for f in &m.functions {
                self.functions.insert(format!("{}_{}", m.name, f.name), f.clone());
                self.functions.insert(format!("{}.{}", m.name, f.name), f.clone());
                self.functions.insert(format!("{}::{}", m.name, f.name), f.clone());
            }
            for f in &m.overrides {
                self.functions.insert(format!("{}_{}", m.name, f.name), f.clone());
                self.functions.insert(format!("{}.{}", m.name, f.name), f.clone());
                self.functions.insert(format!("{}::{}", m.name, f.name), f.clone());
            }
        }

        for m in &module.modules {
            if let Some(ref parent_name) = m.parent {
                if let Some(parent_mod) = module.modules.iter().find(|p| &p.name == parent_name) {
                    for f in &parent_mod.functions {
                        let key1 = format!("{}_{}", m.name, f.name);
                        let key2 = format!("{}.{}", m.name, f.name);
                        let key3 = format!("{}::{}", m.name, f.name);
                        self.functions.entry(key1).or_insert_with(|| f.clone());
                        self.functions.entry(key2).or_insert_with(|| f.clone());
                        self.functions.entry(key3).or_insert_with(|| f.clone());
                    }
                }
            }
        }

        if let Some(main_fn) = self.functions.get("main").cloned() {
            self.eval_function(&main_fn, vec![])
        } else {
            Err("No 'main' function found in module".to_string())
        }
    }

    pub fn eval_named_function(&mut self, module: &Module, name: &str, args: Vec<Value>) -> Result<Value, String> {
        for f in &module.functions {
            self.functions.insert(f.name.clone(), f.clone());
        }

        if let Some(target_fn) = self.functions.get(name).cloned() {
            self.eval_function(&target_fn, args)
        } else {
            Err(format!("Function '{}' not found in module", name))
        }
    }

    pub(crate) fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.variables.pop();
    }

    pub(crate) fn set_var(&mut self, name: &str, val: Value) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), val);
        }
    }

    pub(crate) fn update_var(&mut self, name: &str, val: Value) -> Result<(), String> {
        for scope in self.variables.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        self.set_var(name, val);
        Ok(())
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        for scope in self.variables.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }
}
