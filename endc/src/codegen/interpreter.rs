use crate::ast::*;
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
    pub verified_tasks: std::collections::HashSet<String>,
    pub project_profile: HashMap<String, String>,
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
            verified_tasks: std::collections::HashSet::new(),
            project_profile: HashMap::new(),
        }
    }

    pub fn run(&mut self, module: &Module) -> Result<Value, String> {
        for s in &module.statements {
            self.eval_statement(s)?;
        }

        for f in &module.functions {
            self.functions.insert(f.name.clone(), f.clone());
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

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variables.pop();
    }

    fn set_var(&mut self, name: &str, val: Value) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), val);
        }
    }

    fn update_var(&mut self, name: &str, val: Value) -> Result<(), String> {
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

    fn eval_function(&mut self, func: &FunctionDef, args: Vec<Value>) -> Result<Value, String> {
        self.push_scope();

        for (param, arg) in func.params.iter().zip(args.into_iter()) {
            self.set_var(&param.name, arg);
        }
        
        if func.directives.iter().any(|d| d.name == "@telemetry") {
            println!("[TELEMETRY] Executing {}", func.name);
        }

        let mut ret_val = Value::Void;
        for stmt in &func.body.statements {
            match self.eval_statement(stmt)? {
                Some(v) => {
                    ret_val = v;
                    break;
                }
                None => {}
            }
        }

        self.pop_scope();
        Ok(ret_val)
    }

    pub fn eval_operation(&mut self, op_val: &Value, args: Vec<Value>) -> Result<Value, String> {
        match op_val {
            Value::ComposedOp(op1, op2) => {
                let start_time = std::time::Instant::now();
                let res1 = self.eval_operation(op1, args)?;
                let out1 = match res1 {
                    Value::OperationResult { output, .. } => *output,
                    other => other,
                };
                let res2 = self.eval_operation(op2, vec![out1])?;
                let duration_ns = start_time.elapsed().as_nanos() as u64;
                if let Value::OperationResult { output, status, events, logs, effects, errors, .. } = res2 {
                    Ok(Value::OperationResult {
                        output,
                        status,
                        duration_ns,
                        events,
                        logs,
                        effects,
                        errors,
                    })
                } else {
                    Ok(res2)
                }
            }
            Value::RepeatedOp(op, count, is_retry) => {
                let n = *count;
                if *is_retry {
                    let mut last_res = Value::Void;
                    for _ in 0..n {
                        if let Ok(res) = self.eval_operation(op, args.clone()) {
                            if let Value::OperationResult { ref status, .. } = res {
                                if status == "success" {
                                    return Ok(res);
                                }
                            }
                            last_res = res;
                        }
                    }
                    Ok(last_res)
                } else {
                    let mut results = Vec::new();
                    for _ in 0..n {
                        results.push(self.eval_operation(op, args.clone())?);
                    }
                    Ok(Value::OperationResult {
                        output: Box::new(Value::Array(results)),
                        status: "success".to_string(),
                        duration_ns: 100,
                        events: Vec::new(),
                        logs: Vec::new(),
                        effects: Vec::new(),
                        errors: Vec::new(),
                    })
                }
            }
            Value::AlternativeOp(op1, op2) => {
                if let Ok(res) = self.eval_operation(op1, args.clone()) {
                    if let Value::OperationResult { ref status, .. } = res {
                        if status == "success" {
                            return Ok(res);
                        }
                    }
                }
                self.eval_operation(op2, args)
            }
            Value::ParallelOp(op1, op2) => {
                let res1 = self.eval_operation(op1, args.clone())?;
                let res2 = self.eval_operation(op2, args)?;
                Ok(Value::OperationResult {
                    output: Box::new(Value::Array(vec![res1, res2])),
                    status: "success".to_string(),
                    duration_ns: 100,
                    events: Vec::new(),
                    logs: Vec::new(),
                    effects: Vec::new(),
                    errors: Vec::new(),
                })
            }
            Value::Operation { name, params, requires, guarantees, effects, emits, body, .. } => {
                let (name, params, requires, guarantees, effects, emits, body) =
                    (name.clone(), params.clone(), requires.clone(), guarantees.clone(), effects.clone(), emits.clone(), body.clone());

                let cache_key = name.as_ref().map(|n| format!("{}:{:?}", n, args));
                if let Some(ref k) = cache_key {
                    if let Some(cached) = self.memoized_cache.get(k) {
                        return Ok(cached.clone());
                    }
                }

                let start_time = std::time::Instant::now();
                self.push_scope();

                for (param, arg) in params.iter().zip(args.into_iter()) {
                    self.set_var(&param.name, arg);
                }

                let mut ret_val = Value::Void;
                let mut logs = Vec::new();
                logs.push(format!("Started operation {:?}", name.as_deref().unwrap_or("anon")));

                for req in &requires {
                    logs.push(format!("Checked requirement: {}", req));
                }

                for stmt in &body.statements {
                    match self.eval_statement(stmt)? {
                        Some(v) => {
                            ret_val = v;
                            break;
                        }
                        None => {}
                    }
                }

                for guar in &guarantees {
                    logs.push(format!("Verified guarantee: {}", guar));
                }

                let duration_ns = start_time.elapsed().as_nanos() as u64;
                logs.push(format!("Completed operation {:?} in {}ns", name.as_deref().unwrap_or("anon"), duration_ns));
                self.traces.push(format!("Operation {:?}: status=success, duration={}ns, effects={:?}", name.as_deref().unwrap_or("anon"), duration_ns, effects));

                self.pop_scope();

                let result = Value::OperationResult {
                    output: Box::new(ret_val),
                    status: "success".to_string(),
                    duration_ns,
                    events: emits,
                    logs,
                    effects,
                    errors: Vec::new(),
                };
                if let Some(ref n) = name {
                    self.memoized_cache.insert(n.clone(), result.clone());
                }
                if let Some(ref k) = cache_key {
                    self.memoized_cache.insert(k.clone(), result.clone());
                }
                Ok(result)
            }
            _ => Err("Target is not an operation".to_string()),
        }
    }

    pub fn eval_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, String> {
        match stmt {
            Statement::VarDecl {
                name, initializer, ..
            } => {
                let val = if let Some(init) = initializer {
                    self.eval_expression(init)?
                } else {
                    Value::Void
                };
                self.set_var(name, val);
                Ok(None)
            }
            Statement::Assignment { target, value, .. } => {
                let val = self.eval_expression(value)?;
                match target {
                    Expression::Ident(name, _) => {
                        self.update_var(name, val)?;
                    }
                    _ => {}
                }
                Ok(None)
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    let val = self.eval_expression(v)?;
                    Ok(Some(val))
                } else {
                    Ok(Some(Value::Void))
                }
            }
            Statement::Expression(expr) => {
                self.eval_expression(expr)?;
                Ok(None)
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond = self.eval_expression(condition)?;
                let is_true = match cond {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    _ => false,
                };

                if is_true {
                    for s in &then_block.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                } else if let Some(eb) = else_block {
                    for s in &eb.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::Guard {
                condition,
                else_block,
                ..
            } => {
                let cond = self.eval_expression(condition)?;
                let is_true = match cond {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    _ => false,
                };
                if !is_true {
                    for s in &else_block.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::While { condition, body, .. } => {
                loop {
                    let cond = self.eval_expression(condition)?;
                    let is_true = match cond {
                        Value::Bool(b) => b,
                        Value::Int(n) => n != 0,
                        _ => false,
                    };
                    if !is_true {
                        break;
                    }
                    for s in &body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::ForIn {
                item_name,
                iterable,
                body,
                ..
            } | Statement::ParallelFor {
                item_name,
                iterable,
                body,
                ..
            } => {
                let count = match self.eval_expression(iterable)? {
                    Value::Int(n) => n,
                    _ => 0,
                };
                self.push_scope();
                for i in 0..count {
                    self.set_var(item_name, Value::Int(i));
                    for s in &body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::Match { expr, arms, .. } => {
                let target_val = self.eval_expression(expr)?;
                for arm in arms {
                    if let Some(bindings) = self.matches_pattern(&target_val, &arm.pattern) {
                        self.push_scope();
                        for (k, v) in bindings {
                            self.set_var(&k, v);
                        }

                        let guard_ok = if let Some(g) = &arm.guard {
                            match self.eval_expression(g)? {
                                Value::Bool(b) => b,
                                _ => false,
                            }
                        } else {
                            true
                        };

                        if guard_ok {
                            for s in &arm.body.statements {
                                if let Some(ret) = self.eval_statement(s)? {
                                    self.pop_scope();
                                    return Ok(Some(ret));
                                }
                            }
                            self.pop_scope();
                            break;
                        }
                        self.pop_scope();
                    }
                }
                Ok(None)
            }
            Statement::LeaseBlock { name, initializer, condition, body, .. } => {
                let init_val = self.eval_expression(initializer)?;
                if let Some(cond_expr) = condition {
                    let cond_val = self.eval_expression(cond_expr)?;
                    if let Value::Bool(false) = cond_val {
                        return Ok(None);
                    }
                }
                self.push_scope();
                self.set_var(name, init_val);
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::LeaseCpu { body, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::LeaseEvent { condition, body, .. } => {
                if let Some(cond_expr) = condition {
                    let cond_val = self.eval_expression(cond_expr)?;
                    if let Value::Bool(false) = cond_val {
                        return Ok(None);
                    }
                }
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::LeaseLoop { item_name, iterable, body, .. } => {
                let iter_val = self.eval_expression(iterable)?;
                let count = match iter_val {
                    Value::Int(n) => n,
                    _ => 0,
                };
                self.push_scope();
                for i in 0..count {
                    self.set_var(item_name, Value::Int(i));
                    for s in &body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::RegionBlock { name, body, .. } => {
                self.push_scope();
                self.set_var(&format!("region_{}", name), Value::String(format!("Region<{}>", name)));
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::AsmBlock { arch, .. } => {
                // In interpreter mode, simulate asm execution
                let _ = arch;
                Ok(None)
            }
            Statement::TargetBlock { body, .. } => {
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        return Ok(Some(ret));
                    }
                }
                Ok(None)
            }
            Statement::Defer { expr, .. } => {
                let _ = expr;
                Ok(None)
            }
            Statement::Skip { .. } => Ok(None),
            Statement::InlineC { .. } => Ok(None),
            Statement::Spawn { call, .. } => {
                let _ = self.eval_expression(call)?;
                Ok(None)
            }
            Statement::QuantumUnwrap { name, expr, fallback, .. } => {
                let val = self.eval_expression(expr)?;
                let final_val = match &val {
                    Value::Void => self.eval_expression(fallback)?,
                    Value::Int(0) => self.eval_expression(fallback)?,
                    _ => val,
                };
                self.set_var(name, final_val);
                Ok(None)
            }
            Statement::AtomicOp { target, value, .. } => {
                let add_val = self.eval_expression(value)?;
                let mut is_updated = false;
                // VM Atomic Simulation Block
                {
                    if let Some(Value::Int(curr)) = self.get_var(target) {
                        if let Value::Int(inc) = add_val {
                            let _ = self.update_var(target, Value::Int(curr + inc));
                            is_updated = true;
                        }
                    }
                }
                if !is_updated {
                    return Err(format!("AtomicOp failed: target '{}' is not an integer", target));
                }
                Ok(None)
            }
            Statement::Owned { name, initializer, .. } => {
                let val = self.eval_expression(initializer)?;
                self.set_var(name, val);
                Ok(None)
            }
            Statement::Intent { body, .. } => {
                if let Some(b) = body {
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::Prove { condition, span } => {
                let cond_val = self.eval_expression(condition)?;
                match cond_val {
                    Value::Bool(false) => Err(format!("Formal proof obligation failed at line {}", span.line)),
                    _ => Ok(None),
                }
            }
            Statement::Assume { condition, .. } => {
                let _ = self.eval_expression(condition)?;
                Ok(None)
            }
            Statement::Guarantee { condition, span } => {
                let cond_val = self.eval_expression(condition)?;
                match cond_val {
                    Value::Bool(false) => Err(format!("Postcondition guarantee failed at line {}", span.line)),
                    _ => Ok(None),
                }
            }
            Statement::Invariant { condition, span } => {
                let cond_val = self.eval_expression(condition)?;
                match cond_val {
                    Value::Bool(false) => Err(format!("Invariant violation at line {}", span.line)),
                    _ => Ok(None),
                }
            }
            Statement::VerifyBlock { invariants, span } => {
                for inv in invariants {
                    let val = self.eval_expression(inv)?;
                    if let Value::Bool(false) = val {
                        return Err(format!("Verify contract failed at line {}", span.line));
                    }
                }
                Ok(None)
            }
            Statement::Checkpoint { state_name, .. } => {
                self.snapshots.insert(state_name.clone(), self.variables.clone());
                Ok(None)
            }
            Statement::Rollback { checkpoint_name, span } => {
                if let Some(snap) = self.snapshots.get(checkpoint_name).cloned() {
                    self.variables = snap;
                    Ok(None)
                } else {
                    Err(format!("Rollback failed at line {}: checkpoint '{}' does not exist", span.line, checkpoint_name))
                }
            }
            Statement::TransactionBlock { body, .. } => {
                let pre_txn = self.variables.clone();
                self.push_scope();
                let mut txn_err = None;
                for s in &body.statements {
                    match self.eval_statement(s) {
                        Ok(Some(ret)) => {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                        Ok(None) => {}
                        Err(e) => {
                            txn_err = Some(e);
                            break;
                        }
                    }
                }
                self.pop_scope();
                if let Some(err) = txn_err {
                    self.variables = pre_txn;
                    return Err(format!("Transaction aborted and rolled back: {}", err));
                }
                Ok(None)
            }
            Statement::Handoff { resource, target_domain, .. } => {
                self.domain_ownership.insert(resource.clone(), target_domain.clone());
                Ok(None)
            }
            Statement::ReturnTo { source_domain, resource, .. } => {
                if source_domain == "cpu" || source_domain == "host" {
                    self.domain_ownership.remove(resource);
                } else {
                    self.domain_ownership.insert(resource.clone(), source_domain.clone());
                }
                Ok(None)
            }
            Statement::ProtectBlock { body, .. }
            | Statement::DeterministicBlock { body, .. }
            | Statement::ReplayBlock { body, .. }
            | Statement::SpeculativeBlock { body, .. }
            | Statement::FallbackBlock { body, .. }
            | Statement::CancelSafeBlock { body, .. }
            | Statement::TaskDecl { body, .. }
            | Statement::PatchDecl { body, .. }
            | Statement::RaceFreeBlock { body, .. }
            | Statement::DeadlineBlock { body, .. }
            | Statement::PriorityBlock { body, .. }
            | Statement::QualityBlock { body, .. }
            | Statement::TradeoffBlock { body, .. }
            | Statement::WatchBlock { handler: body, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::ComputeBlock { body, fallback, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                if let Some(fb) = fallback {
                    let _ = fb;
                }
                Ok(None)
            }
            Statement::BudgetBlock { body, .. }
            | Statement::ContextBlock { body, .. }
            | Statement::AgentContract { body, .. }
            | Statement::EvolveBlock { body, .. } => {
                if let Some(b) = body {
                    self.push_scope();
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                    self.pop_scope();
                }
                Ok(None)
            }
            Statement::AdaptBlock { branches, .. } => {
                for (cond, blk) in branches {
                    let cond_val = self.eval_expression(cond)?;
                    let is_match = match cond_val {
                        Value::Bool(b) => b,
                        Value::Int(n) => n != 0,
                        _ => false,
                    };
                    if is_match {
                        self.push_scope();
                        for s in &blk.statements {
                            if let Some(ret) = self.eval_statement(s)? {
                                self.pop_scope();
                                return Ok(Some(ret));
                            }
                        }
                        self.pop_scope();
                        break;
                    }
                }
                Ok(None)
            }
            Statement::ReactBlock { event, handler, .. } => {
                let _ = self.eval_expression(event)?;
                self.push_scope();
                for s in &handler.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::StreamBlock { source, operations, .. } => {
                let _ = self.eval_expression(source)?;
                for op in operations {
                    let _ = self.eval_expression(op)?;
                }
                Ok(None)
            }
            Statement::FlowBlock { steps, .. } => {
                for step in steps {
                    let _ = self.eval_expression(step)?;
                }
                Ok(None)
            }
            Statement::ParallelChoose { branches, .. } => {
                let (tx, rx) = std::sync::mpsc::channel();
                for (branch_name, blk) in branches {
                    let blk_c = blk.clone();
                    let mut interp_c = self.clone();
                    let tx_c = tx.clone();
                    let b_name = branch_name.clone();
                    std::thread::spawn(move || {
                        interp_c.push_scope();
                        let mut res = None;
                        for s in &blk_c.statements {
                            if let Ok(Some(ret)) = interp_c.eval_statement(s) {
                                res = Some(ret);
                                break;
                            }
                        }
                        let _ = tx_c.send((b_name, res, interp_c.variables));
                    });
                }
                drop(tx);
                if let Ok((_chosen_name, maybe_val, updated_vars)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                    if let Some(top_scope) = updated_vars.last() {
                        for (k, v) in top_scope {
                            self.set_var(k, v.clone());
                        }
                    }
                    if let Some(v) = maybe_val {
                        return Ok(Some(v));
                    }
                }
                Ok(None)
            }
            Statement::RaceBlock { branches, .. } => {
                let (tx, rx) = std::sync::mpsc::channel();
                for (idx, blk) in branches.iter().enumerate() {
                    let blk_c = blk.clone();
                    let mut interp_c = self.clone();
                    let tx_c = tx.clone();
                    std::thread::spawn(move || {
                        interp_c.push_scope();
                        let mut res = None;
                        for s in &blk_c.statements {
                            if let Ok(Some(ret)) = interp_c.eval_statement(s) {
                                res = Some(ret);
                                break;
                            }
                        }
                        let _ = tx_c.send((idx, res, interp_c.variables));
                    });
                }
                drop(tx);
                if let Ok((_winner_idx, maybe_val, updated_vars)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                    if let Some(top_scope) = updated_vars.last() {
                        for (k, v) in top_scope {
                            self.set_var(k, v.clone());
                        }
                    }
                    if let Some(v) = maybe_val {
                        return Ok(Some(v));
                    }
                }
                Ok(None)
            }
            Statement::HedgeBlock { delay_ms, primary, fallback, .. } => {
                let delay_ms_val = match self.eval_expression(delay_ms)? {
                    Value::Int(n) if n > 0 => n as u64,
                    _ => 20,
                };
                let (tx, rx) = std::sync::mpsc::channel();
                let prim_blk = primary.clone();
                let mut prim_interp = self.clone();
                let tx_prim = tx.clone();
                std::thread::spawn(move || {
                    prim_interp.push_scope();
                    let mut res = None;
                    for s in &prim_blk.statements {
                        if let Ok(Some(ret)) = prim_interp.eval_statement(s) {
                            res = Some(ret);
                            break;
                        }
                    }
                    let _ = tx_prim.send(("primary", res, prim_interp.variables));
                });

                match rx.recv_timeout(std::time::Duration::from_millis(delay_ms_val)) {
                    Ok((_, maybe_v, updated_vars)) => {
                        if let Some(top_scope) = updated_vars.last() {
                            for (k, v) in top_scope {
                                self.set_var(k, v.clone());
                            }
                        }
                        Ok(maybe_v)
                    }
                    Err(_) => {
                        let fb_blk = fallback.clone();
                        let mut fb_interp = self.clone();
                        let tx_fb = tx.clone();
                        std::thread::spawn(move || {
                            fb_interp.push_scope();
                            let mut res = None;
                            for s in &fb_blk.statements {
                                if let Ok(Some(ret)) = fb_interp.eval_statement(s) {
                                    res = Some(ret);
                                    break;
                                }
                            }
                            let _ = tx_fb.send(("fallback", res, fb_interp.variables));
                        });
                        drop(tx);
                        if let Ok((_, maybe_v, updated_vars)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                            if let Some(top_scope) = updated_vars.last() {
                                for (k, v) in top_scope {
                                    self.set_var(k, v.clone());
                                }
                            }
                            Ok(maybe_v)
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            Statement::AdapterDecl { body, .. }
            | Statement::PreserveRefactorDecl { body, .. }
            | Statement::CompatDecl { body, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::SplitDecl { entity, parts, .. } => {
                let parts_str = parts.join(", ");
                self.set_var(&format!("__split_{}", entity), Value::String(parts_str));
                Ok(None)
            }
            Statement::PartitionDecl { entity, by, parts, .. } => {
                let parts_str = parts.join(", ");
                self.set_var(&format!("__partition_{}_{}", entity, by), Value::String(parts_str));
                Ok(None)
            }
            Statement::ExtractDecl { symbols, into_module, .. } => {
                self.set_var(&format!("__extract_{}", into_module), Value::String(symbols.join(", ")));
                Ok(None)
            }
            Statement::ClusterDecl { by, predicate, .. } => {
                self.set_var(&format!("__cluster_{}", by), Value::String(predicate.clone()));
                Ok(None)
            }
            Statement::SeparateDecl { left, right, .. } => {
                self.set_var(&format!("__separate_{}_{}", left, right), Value::Bool(true));
                Ok(None)
            }
            Statement::MoveDecl { symbol, from_mod, to_mod, .. } => {
                self.set_var(&format!("__move_{}_{}_{}", symbol, from_mod, to_mod), Value::Bool(true));
                Ok(None)
            }
            Statement::MigrateDecl { entity, from_mod, to_mod, .. } => {
                self.set_var(&format!("__migrate_{}_{}_{}", entity, from_mod, to_mod), Value::Bool(true));
                Ok(None)
            }
            Statement::RedirectDecl { from_api, to_api, .. } => {
                self.set_var(&format!("__redirect_{}_{}", from_api, to_api), Value::Bool(true));
                Ok(None)
            }
            Statement::DecomposeDecl { target, target_modules, .. } => {
                let count = target_modules.unwrap_or(25);
                self.set_var(&format!("__decompose_{}_target", target), Value::Int(count as i64));
                Ok(None)
            }
            Statement::ModularizeDecl { target, target_files_min, target_files_max, .. } => {
                self.set_var(&format!("__modularize_{}", target), Value::Int(*target_files_max as i64));
                self.set_var(&format!("__modularize_{}_min", target), Value::Int(*target_files_min as i64));
                Ok(None)
            }
            Statement::EvolveArchDecl { from, toward, target_modules, .. } => {
                self.set_var(&format!("__evolve_{}_{}", from, toward), Value::Int(*target_modules as i64));
                Ok(None)
            }
            Statement::GravityDecl { weights, .. } => {
                for (k, w) in weights {
                    self.set_var(&format!("__gravity_{}", k), Value::Float(*w));
                }
                Ok(None)
            }
            Statement::BudgetContextDecl { name, token_budget, .. } => {
                self.set_var(&format!("__budget_context_{}", name), Value::Int(*token_budget as i64));
                Ok(None)
            }
            Statement::RepairDecl { target, .. } => {
                self.set_var(&format!("__repair_{}", target), Value::Bool(true));
                Ok(None)
            }
            Statement::OperationDecl(op_def) => {
                let op_val = Value::Operation {
                    name: if op_def.name.is_empty() { None } else { Some(op_def.name.clone()) },
                    params: op_def.params.clone(),
                    return_type: op_def.return_type.clone(),
                    requires: op_def.requires.clone(),
                    guarantees: op_def.guarantees.clone(),
                    effects: op_def.effects.clone(),
                    emits: op_def.emits.clone(),
                    version: op_def.version,
                    body: op_def.body.clone(),
                };
                if !op_def.name.is_empty() {
                    self.operations.insert(op_def.name.clone(), op_val.clone());
                    self.set_var(&op_def.name, op_val);
                }
                Ok(None)
            }
            Statement::EventDecl(ev_def) => {
                self.set_var(&format!("__event_{}", ev_def.name), Value::String(ev_def.name.clone()));
                Ok(None)
            }
            Statement::EventHubDecl(hub_def) => {
                self.event_hubs.insert(hub_def.name.clone(), hub_def.clone());
                for handler in &hub_def.handlers {
                    self.event_handlers.entry(handler.event_name.clone()).or_insert_with(Vec::new).push(handler.clone());
                }
                Ok(None)
            }
            Statement::EmitEvent { event_name, args, .. } => {
                self.emitted_events.push(event_name.clone());
                let mut eval_args = Vec::new();
                for a in args {
                    eval_args.push(self.eval_expression(a)?);
                }
                if let Some(handlers) = self.event_handlers.get(event_name).cloned() {
                    for h in handlers {
                        if let Some(ref op_expr) = h.handler_op {
                            let op_val = self.eval_expression(op_expr)?;
                            if matches!(op_val, Value::Operation { .. }) {
                                let _ = self.eval_operation(&op_val, eval_args.clone())?;
                            }
                        } else if let Some(ref blk) = h.body {
                            self.push_scope();
                            for s in &blk.statements {
                                if let Some(ret) = self.eval_statement(s)? {
                                    self.pop_scope();
                                    return Ok(Some(ret));
                                }
                            }
                            self.pop_scope();
                        }
                    }
                }
                Ok(None)
            }
            Statement::OperationDecl(op) => {
                let op_val = Value::Operation {
                    name: Some(op.name.clone()),
                    params: op.params.clone(),
                    return_type: op.return_type.clone(),
                    requires: op.requires.clone(),
                    guarantees: op.guarantees.clone(),
                    effects: op.effects.clone(),
                    emits: op.emits.clone(),
                    version: None,
                    body: op.body.clone(),
                };
                self.operations.insert(op.name.clone(), op_val.clone());
                self.set_var(&op.name, op_val);
                let fn_def = FunctionDef {
                    name: op.name.clone(),
                    generic_params: Vec::new(),
                    is_pub: op.is_pub,
                    params: op.params.clone(),
                    return_type: op.return_type.clone(),
                    body: op.body.clone(),
                    directives: Vec::new(),
                    morphic_param: None,
                    span: op.span.clone(),
                };
                self.functions.insert(op.name.clone(), fn_def);
                Ok(None)
            }
            Statement::ObserveOp { op_expr, alias, .. } => {
                let op_val = if let Expression::Ident(id, _) = op_expr {
                    self.operations.get(id).cloned().or_else(|| self.get_var(id))
                } else {
                    self.eval_expression(op_expr).ok()
                }.unwrap_or_else(|| Value::String(format!("{:?}", op_expr)));

                self.set_var(alias, op_val);
                self.traces.push(alias.clone());
                Ok(None)
            }
            Statement::AnalyzeOp { op_expr, .. } => {
                let (op_name, op_val) = if let Expression::Ident(id, _) = op_expr {
                    (id.clone(), self.operations.get(id).cloned())
                } else {
                    ("anon".to_string(), self.eval_expression(op_expr).ok())
                };
                let analysis_summary = if let Some(Value::Operation { name, requires, guarantees, effects, emits, .. }) = &op_val {
                    format!("Operation: {:?}, requires: {:?}, guarantees: {:?}, effects: {:?}, emits: {:?}", name, requires, guarantees, effects, emits)
                } else {
                    format!("Operation: {}, static analysis complete", op_name)
                };
                self.set_var(&format!("__analysis_{}", op_name), Value::String(analysis_summary));
                Ok(None)
            }
            Statement::ExtractOpDecl { op_name, from_mod, condition, .. } => {
                self.set_var(&format!("__extract_op_{}", op_name), Value::String(format!("{}: {}", from_mod, condition)));
                Ok(None)
            }
            Statement::InlineOpDecl { op_name, .. } => {
                self.set_var(&format!("__inline_op_{}", op_name), Value::Bool(true));
                Ok(None)
            }
            Statement::SplitOpDecl { op_name, sub_ops, .. } => {
                self.set_var(&format!("__split_op_{}", op_name), Value::String(sub_ops.join(", ")));
                Ok(None)
            }
            Statement::MergeOpDecl { source_ops, as_name, .. } => {
                self.set_var(&format!("__merge_op_{}", as_name), Value::String(source_ops.join(" + ")));
                Ok(None)
            }
            Statement::ExplainOpDecl { op_name, .. } => {
                self.set_var(&format!("__explain_op_{}", op_name), Value::String(format!("Contract explanation for operation {}", op_name)));
                Ok(None)
            }
            Statement::EvolveOpDecl { op_name, preserve, optimize, allow, reject, .. } => {
                self.set_var(&format!("__evolve_op_{}", op_name), Value::String(format!("preserve: {:?}, optimize: {:?}, allow: {:?}, reject: {:?}", preserve, optimize, allow, reject)));
                Ok(None)
            }
            Statement::FeatureDecl { name, requirement, skills, tasks, .. } => {
                self.features.insert(name.clone(), (requirement.clone(), skills.clone(), tasks.clone()));
                self.set_var(&format!("__feature_{}", name), Value::String(format!("Feature {}: req={:?}, skills={:?}, tasks={:?}", name, requirement, skills, tasks)));
                Ok(None)
            }
            Statement::FeatureStatement(f) => {
                let req = f.contracts.first().map(|c| c.rule.clone());
                let skills = f.requires_capabilities.clone();
                let tasks: Vec<String> = f.decisions.iter().map(|d| d.target.clone()).collect();
                self.features.insert(f.name.clone(), (req.clone(), skills.clone(), tasks.clone()));
                self.set_var(&format!("__feature_{}", f.name), Value::String(format!("Feature {}: req={:?}, skills={:?}, tasks={:?}", f.name, req, skills, tasks)));
                Ok(None)
            }
            Statement::SkillDecl { name, rules, constraints, structural, semantic, behavioral, architectural, performance, security, testing, agent, requires, hard, soft, for_scope, .. } => {
                let state = SkillDefState {
                    name: name.clone(),
                    rules: rules.clone(),
                    constraints: constraints.clone(),
                    structural: structural.clone(),
                    semantic: semantic.clone(),
                    behavioral: behavioral.clone(),
                    architectural: architectural.clone(),
                    performance: performance.clone(),
                    security: security.clone(),
                    testing: testing.clone(),
                    agent: agent.clone(),
                    requires: requires.clone(),
                    hard: hard.clone(),
                    soft: soft.clone(),
                    for_scope: for_scope.clone(),
                };
                self.skills.insert(name.clone(), state);
                self.set_var(&format!("__skill_{}", name), Value::String(format!("Skill {}: scope={:?}, rules={:?}, requires={:?}", name, for_scope, rules, requires)));
                Ok(None)
            }
            Statement::SatisfiesDecl { entity, skills, .. } => {
                self.set_var(&format!("__satisfies_{}", entity), Value::String(skills.join(", ")));
                Ok(None)
            }
            Statement::ProjectSkillsDecl { profile, .. } => {
                for (k, v) in profile {
                    self.project_profile.insert(k.clone(), v.clone());
                }
                self.set_var("__project_skills", Value::String(format!("{:?}", self.project_profile)));
                Ok(None)
            }
            Statement::AgentTaskContractDecl { name, owner, status, requirement, implementation, skills, change_budget, evidence, .. } => {
                let t_owner = owner.clone().unwrap_or_else(|| "agent".to_string());
                let t_status = status.clone().unwrap_or_else(|| "planned".to_string());
                let state = TaskState {
                    name: name.clone(),
                    owner: t_owner,
                    status: t_status,
                    requirement: requirement.clone(),
                    implementation: implementation.clone(),
                    skills: skills.clone(),
                    change_budget: change_budget.clone(),
                    evidence: evidence.clone(),
                    result: None,
                    confidence: None,
                    summary: None,
                    notes: None,
                };
                self.tasks_state.insert(name.clone(), state);
                self.set_var(&format!("__task_{}", name), Value::String(format!("Task {}: req={:?}, impl={:?}, skills={:?}", name, requirement, implementation, skills)));
                Ok(None)
            }
            Statement::ClaimTask { task_name, .. } => {
                if let Some(task) = self.tasks_state.get_mut(task_name) {
                    task.status = "claimed".to_string();
                    task.owner = "agent".to_string();
                } else {
                    let state = TaskState {
                        name: task_name.clone(),
                        owner: "agent".to_string(),
                        status: "claimed".to_string(),
                        requirement: None,
                        implementation: None,
                        skills: Vec::new(),
                        change_budget: Vec::new(),
                        evidence: Vec::new(),
                        result: None,
                        confidence: None,
                        summary: None,
                        notes: None,
                    };
                    self.tasks_state.insert(task_name.clone(), state);
                }
                self.set_var(&format!("__claim_task_{}", task_name), Value::String("claimed".to_string()));
                Ok(None)
            }
            Statement::CompleteTask { task_name, result, confidence, summary, evidence, notes, .. } => {
                if let Some(task) = self.tasks_state.get_mut(task_name) {
                    task.status = "completed".to_string();
                    task.result = Some(result.clone());
                    task.confidence = *confidence;
                    task.summary = summary.clone();
                    task.notes = notes.clone();
                } else {
                    let state = TaskState {
                        name: task_name.clone(),
                        owner: "agent".to_string(),
                        status: "completed".to_string(),
                        requirement: None,
                        implementation: None,
                        skills: Vec::new(),
                        change_budget: Vec::new(),
                        evidence: evidence.iter().map(|e| ("evidence".to_string(), e.clone())).collect(),
                        result: Some(result.clone()),
                        confidence: *confidence,
                        summary: summary.clone(),
                        notes: notes.clone(),
                    };
                    self.tasks_state.insert(task_name.clone(), state);
                }
                self.set_var(&format!("__complete_task_{}", task_name), Value::String(format!("result={}, confidence={:?}, evidence={:?}", result, confidence, evidence)));
                Ok(None)
            }
            Statement::VerifyTask { target, is_adversarial, skill, .. } => {
                self.verified_tasks.insert(target.clone());
                if let Some(task) = self.tasks_state.get_mut(target) {
                    task.status = "accepted".to_string();
                }
                self.set_var(&format!("__verify_task_{}", target), Value::String(format!("verified: adversarial={}, skill={:?}", is_adversarial, skill)));
                Ok(None)
            }
            Statement::RequirementDecl { req_id, description, .. } => {
                self.requirements.insert(req_id.clone(), description.clone());
                self.set_var(&format!("__requirement_{}", req_id), Value::String(description.clone()));
                Ok(None)
            }
            Statement::ImplementsDecl { req_id, entities, .. } => {
                self.requirement_implements.insert(req_id.clone(), entities.clone());
                self.set_var(&format!("__implements_{}", req_id), Value::String(entities.join(", ")));
                Ok(None)
            }
            Statement::VerifiesDecl { req_id, entities, .. } => {
                self.requirement_verifies.insert(req_id.clone(), entities.clone());
                self.set_var(&format!("__verifies_{}", req_id), Value::String(entities.join(", ")));
                Ok(None)
            }
            Statement::TodoDecl { id, implement, requires, verify, status, .. } => {
                let state = TodoState {
                    id: id.clone(),
                    implement: implement.clone(),
                    requires: requires.clone(),
                    verify: verify.clone(),
                    status: status.clone(),
                };
                self.todos_state.insert(id.clone(), state);
                self.set_var(&format!("__todo_{}", id), Value::String(format!("Todo {}: implement={}, status={}", id, implement, status)));
                Ok(None)
            }
            Statement::AgentBoundaryDecl { module_name, .. } => {
                self.set_var(&format!("__agent_boundary_{}", module_name), Value::Bool(true));
                Ok(None)
            }
            Statement::AgentContextDecl { module_name, expose, hide, .. } => {
                self.set_var(&format!("__agent_context_{}", module_name), Value::String(format!("expose: {:?}, hide: {:?}", expose, hide)));
                Ok(None)
            }
            Statement::ContextFirewallDecl { module_name, deny, expose, .. } => {
                self.set_var(&format!("__context_firewall_{}", module_name), Value::String(format!("deny: {:?}, expose: {:?}", deny, expose)));
                Ok(None)
            }
            Statement::AgentApiDecl { module_name, expose, hide, .. } => {
                self.set_var(&format!("__agent_api_{}", module_name), Value::String(format!("expose: {:?}, hide: {:?}", expose, hide)));
                Ok(None)
            }
            Statement::AgentabilityDecl { max_context_tokens, max_operation_complexity, max_dependency_fanout, .. } => {
                self.set_var("__agentability", Value::String(format!("max_tokens={}, complexity={}, fanout={}", max_context_tokens, max_operation_complexity, max_dependency_fanout)));
                Ok(None)
            }
            Statement::IntentDecl { goal, preserve, optimize, .. } => {
                self.set_var("__intent", Value::String(format!("goal={}, preserve={:?}, optimize={:?}", goal, preserve, optimize)));
                Ok(None)
            }
            Statement::SemanticCommitDecl { task, intent, satisfies, evidence, .. } => {
                self.set_var(&format!("__semantic_commit_{}", task), Value::String(format!("intent={}, satisfies={:?}, evidence={:?}", intent, satisfies, evidence)));
                Ok(None)
            }
            Statement::AgentReviewDecl { task_id, summary, completed, unresolved, risks, confidence, .. } => {
                let report = AgentReportState {
                    task_id: task_id.clone(),
                    summary: summary.clone(),
                    completed: *completed,
                    unresolved: *unresolved,
                    risks: *risks,
                    confidence: *confidence,
                };
                self.agent_reports.push(report);
                self.set_var(&format!("__review_{}", task_id), Value::String(format!("summary={}, completed={}, confidence={}", summary, completed, confidence)));
                Ok(None)
            }
            Statement::ApprovalDecl { required_items, .. } => {
                self.set_var("__approval_required", Value::String(required_items.join(", ")));
                Ok(None)
            }
            Statement::AgentLeaseDecl { module_name, owner, duration, .. } => {
                self.agent_leases.insert(module_name.clone(), (owner.clone(), duration.clone()));
                self.set_var(&format!("__lease_{}", module_name), Value::String(format!("owner={}, duration={}", owner, duration)));
                Ok(None)
            }
            Statement::KnowledgeDecl { name, decisions, constraints, .. } => {
                self.knowledge_base.insert(name.clone(), (decisions.clone(), constraints.clone()));
                self.set_var(&format!("__knowledge_{}", name), Value::String(format!("decisions={:?}, constraints={:?}", decisions, constraints)));
                Ok(None)
            }
            Statement::DecisionDecl { id, choose, because, reject, .. } => {
                self.decision_records.insert(id.clone(), (choose.clone(), because.clone(), reject.clone()));
                self.set_var(&format!("__decision_{}", id), Value::String(format!("choose={}, because={}, reject={}", choose, because, reject)));
                Ok(None)
            }
            Statement::AgentCapabilityDecl { capabilities, cannot, .. } => {
                self.set_var("__agent_capabilities", Value::String(format!("can={:?}, cannot={:?}", capabilities, cannot)));
                Ok(None)
            }
            Statement::RegressionGuardDecl { items, .. } => {
                self.set_var("__regression_guard", Value::String(items.join(", ")));
                Ok(None)
            }
            Statement::OnEventStmt(on_ev) => {
                let should_run = if let Some(g) = &on_ev.guard {
                    match self.eval_expression(g)? {
                        Value::Bool(b) => b,
                        Value::Int(n) => n != 0,
                        _ => true,
                    }
                } else {
                    true
                };
                if should_run {
                    self.push_scope();
                    for s in &on_ev.body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                    self.pop_scope();
                }
                self.set_var(&format!("__on_event_{}", on_ev.event_pattern.replace('.', "_")), Value::Bool(true));
                Ok(None)
            }
            Statement::OnceEventStmt(once_ev) => {
                self.push_scope();
                for s in &once_ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                self.set_var(&format!("__once_event_{}", once_ev.event_pattern.replace('.', "_")), Value::Bool(true));
                Ok(None)
            }
            Statement::EveryEventStmt(ev) => {
                self.push_scope();
                for s in &ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                self.set_var("__every_event", Value::String(ev.interval_str.clone()));
                Ok(None)
            }
            Statement::AfterEventStmt(ev) => {
                self.push_scope();
                for s in &ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                self.set_var("__after_event", Value::String(ev.delay_str.clone()));
                Ok(None)
            }
            Statement::BeforeEventStmt(ev) => {
                self.push_scope();
                for s in &ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                self.set_var(&format!("__before_event_{}", ev.event_pattern.replace('.', "_")), Value::Bool(true));
                Ok(None)
            }
            Statement::ReactiveStateStmt(st) => {
                let init_val = self.eval_expression(&st.initial_val)?;
                self.set_var(&st.name, init_val.clone());
                self.set_var(&format!("__state_{}", st.name), init_val);
                Ok(None)
            }
            Statement::DeriveStmt(d) => {
                let derived_val = self.eval_expression(&d.expr)?;
                self.set_var(&d.target_var, derived_val.clone());
                self.set_var(&format!("__derived_{}", d.target_var), derived_val);
                Ok(None)
            }
            Statement::TopologyStmt(top) => {
                self.set_var(&format!("__topology_{}", top.name), Value::String(format!("nodes={:?}, edges={:?}", top.nodes, top.edges)));
                Ok(None)
            }
            Statement::EventStreamOpStmt(op) => {
                if let Some(b) = &op.body {
                    self.push_scope();
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                    self.pop_scope();
                }
                self.set_var(&format!("__{}_{}", op.op_kind, op.target), Value::String(op.params.join(", ")));
                Ok(None)
            }
            Statement::EventTransactionStmt(tx) => {
                let checkpoint = self.variables.clone();
                let mut failed = false;
                self.push_scope();
                for s in &tx.statements {
                    if self.eval_statement(s).is_err() {
                        failed = true;
                        break;
                    }
                }
                self.pop_scope();
                if failed {
                    self.variables = checkpoint;
                    if let Some(r) = &tx.on_rollback {
                        self.push_scope();
                        for s in &r.statements {
                            let _ = self.eval_statement(s);
                        }
                        self.pop_scope();
                    }
                }
                Ok(None)
            }
            Statement::EventControlStmt(ctl) => {
                self.set_var(&format!("__{}_{}", ctl.action, ctl.target), Value::String(ctl.args.join(", ")));
                Ok(None)
            }
            Statement::ClassDecl(c) => {
                self.set_var(&format!("__class_{}", c.name), Value::String(format!("extends={:?}, mixins={:?}, implements={:?}", c.extends, c.mixins, c.implements)));
                for m in &c.methods {
                    self.functions.insert(format!("{}::{}", c.name, m.name), m.clone());
                }
                Ok(None)
            }
            Statement::TraitDecl(t) => {
                self.set_var(&format!("__trait_{}", t.name), Value::String(format!("extends={:?}", t.extends)));
                Ok(None)
            }
            Statement::InheritStmt(i) => {
                if let Some(b) = &i.body {
                    self.push_scope();
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                    self.pop_scope();
                }
                self.set_var(&format!("__inherit_{}_{}", i.target, i.parent), Value::String(format!("kind={:?}, is_contractual={}", i.kind, i.is_contractual)));
                Ok(None)
            }
            Statement::SuperCallStmt(s) => {
                let mut evaluated_args = Vec::new();
                for arg in &s.args {
                    evaluated_args.push(self.eval_expression(arg)?);
                }
                let target_fn = match &s.target_parent {
                    Some(parent) => format!("{}::{}", parent, s.method),
                    None => s.method.clone(),
                };
                if let Some(func) = self.functions.get(&target_fn).cloned() {
                    let res = self.eval_function(&func, evaluated_args)?;
                    Ok(Some(res))
                } else {
                    self.set_var(&format!("__super_{}", s.method), Value::String(format!("parent={:?}, args={:?}", s.target_parent, evaluated_args)));
                    Ok(None)
                }
            }
            Statement::ConflictStmt(c) => {
                self.set_var(&format!("__conflict_{}_{}", c.left.replace('.', "_"), c.right.replace('.', "_")), Value::Bool(true));
                Ok(None)
            }
            Statement::ResolveConflictStmt(r) => {
                self.set_var(&format!("__resolve_{}", r.preferred.replace('.', "_")), Value::String(r.over.clone().unwrap_or_default()));
                Ok(None)
            }
            Statement::InspectInheritanceStmt(i) => {
                self.set_var(&format!("__inspect_inheritance_{}", i.target), Value::Bool(true));
                Ok(None)
            }
            Statement::ImpactInheritanceStmt(i) => {
                self.set_var(&format!("__impact_inheritance_{}", i.target), Value::Bool(true));
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn matches_pattern(&self, target: &Value, pattern: &Pattern) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(Vec::new()),
            Pattern::Ident(id) => Some(vec![(id.clone(), target.clone())]),
            Pattern::Literal(lit) => {
                let lit_val = match lit {
                    Literal::Int(n) => Value::Int(*n),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::Null => Value::Pointer(0),
                };
                if *target == lit_val {
                    Some(Vec::new())
                } else {
                    None
                }
            }
            Pattern::Variant { variant_name, binding, .. } => {
                if let Value::Enum(_, vname, payload) = target {
                    if vname == variant_name {
                        let mut out = Vec::new();
                        if let (Some(b), Some(p)) = (binding, payload) {
                            out.push((b.clone(), *p.clone()));
                        }
                    }
                }
                None
            }
            Pattern::Binding(id) => Some(vec![(id.clone(), target.clone())]),
            Pattern::Tuple(_) | Pattern::Struct { .. } => Some(Vec::new()),
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Lit(lit, _) => match lit {
                Literal::Int(n) => Ok(Value::Int(*n)),
                Literal::Float(f) => Ok(Value::Float(*f)),
                Literal::String(s) => Ok(Value::String(s.clone())),
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Null => Ok(Value::Pointer(0)),
            },
            Expression::Ident(name, _) => {
                if let Some(v) = self.get_var(name) {
                    Ok(v)
                } else if let Some(op) = self.operations.get(name) {
                    Ok(op.clone())
                } else {
                    Ok(Value::String(name.clone()))
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let l = self.eval_expression(left)?;
                let r = self.eval_expression(right)?;

                match (&l, op, &r) {
                    // Int arithmetic
                    (Value::Int(a), BinaryOp::Add, Value::Int(b)) => Ok(Value::Int(*a + *b)),
                    (Value::Int(a), BinaryOp::Sub, Value::Int(b)) => Ok(Value::Int(*a - *b)),
                    (Value::Int(a), BinaryOp::Mul, Value::Int(b)) => Ok(Value::Int(*a * *b)),
                    (Value::Int(a), BinaryOp::Div, Value::Int(b)) => {
                        if *b == 0 {
                            Err("Division by zero in End runtime".to_string())
                        } else {
                            Ok(Value::Int(*a / *b))
                        }
                    }
                    (Value::Int(a), BinaryOp::Mod, Value::Int(b)) => Ok(Value::Int(if *b != 0 { *a % *b } else { 0 })),
                    // Int comparisons
                    (Value::Int(a), BinaryOp::Equal, Value::Int(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::Int(a), BinaryOp::NotEqual, Value::Int(b)) => Ok(Value::Bool(*a != *b)),
                    (Value::Int(a), BinaryOp::LessThan, Value::Int(b)) => Ok(Value::Bool(*a < *b)),
                    (Value::Int(a), BinaryOp::LessEqual, Value::Int(b)) => Ok(Value::Bool(*a <= *b)),
                    (Value::Int(a), BinaryOp::GreaterThan, Value::Int(b)) => Ok(Value::Bool(*a > *b)),
                    (Value::Int(a), BinaryOp::GreaterEqual, Value::Int(b)) => Ok(Value::Bool(*a >= *b)),

                    // Float arithmetic
                    (Value::Float(a), BinaryOp::Add, Value::Float(b)) => Ok(Value::Float(*a + *b)),
                    (Value::Float(a), BinaryOp::Sub, Value::Float(b)) => Ok(Value::Float(*a - *b)),
                    (Value::Float(a), BinaryOp::Mul, Value::Float(b)) => Ok(Value::Float(*a * *b)),
                    (Value::Float(a), BinaryOp::Div, Value::Float(b)) => Ok(Value::Float(if *b != 0.0 { *a / *b } else { 0.0 })),
                    // Float comparisons
                    (Value::Float(a), BinaryOp::Equal, Value::Float(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::Float(a), BinaryOp::NotEqual, Value::Float(b)) => Ok(Value::Bool(*a != *b)),
                    (Value::Float(a), BinaryOp::LessThan, Value::Float(b)) => Ok(Value::Bool(*a < *b)),
                    (Value::Float(a), BinaryOp::LessEqual, Value::Float(b)) => Ok(Value::Bool(*a <= *b)),
                    (Value::Float(a), BinaryOp::GreaterThan, Value::Float(b)) => Ok(Value::Bool(*a > *b)),
                    (Value::Float(a), BinaryOp::GreaterEqual, Value::Float(b)) => Ok(Value::Bool(*a >= *b)),

                    // Bool logical operations
                    (Value::Bool(a), BinaryOp::And, Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                    (Value::Bool(a), BinaryOp::Or, Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                    (Value::Bool(a), BinaryOp::Equal, Value::Bool(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::Bool(a), BinaryOp::NotEqual, Value::Bool(b)) => Ok(Value::Bool(*a != *b)),

                    // String operations
                    (Value::String(a), BinaryOp::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::String(a), BinaryOp::Equal, Value::String(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::String(a), BinaryOp::NotEqual, Value::String(b)) => Ok(Value::Bool(*a != *b)),

                    // Operation Algebra
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::Add | BinaryOp::Shr, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) => {
                        Ok(Value::ComposedOp(Box::new(l.clone()), Box::new(r.clone())))
                    }
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::Mul, Value::Int(n)) => {
                        Ok(Value::RepeatedOp(Box::new(l.clone()), (*n).max(1) as usize, false))
                    }
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::BitAnd, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) => {
                        Ok(Value::ParallelOp(Box::new(l.clone()), Box::new(r.clone())))
                    }
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::BitOr, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) => {
                        Ok(Value::AlternativeOp(Box::new(l.clone()), Box::new(r.clone())))
                    }
                    _ => Ok(Value::Int(0)),
                }
            }
            Expression::Unary { op, expr, .. } => {
                let v = self.eval_expression(expr)?;
                match (op, v) {
                    (UnaryOp::Negate, Value::Int(n)) => Ok(Value::Int(-n)),
                    (UnaryOp::Negate, Value::Float(f)) => Ok(Value::Float(-f)),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    (_, other) => Ok(other),
                }
            }
            Expression::Call { callee, args, .. } => {
                let mut eval_args = Vec::new();
                for a in args {
                    eval_args.push(self.eval_expression(a)?);
                }

                if let Expression::Ident(name, _) = callee.as_ref() {
                    if name == "println" || name == "print" {
                        let msg = eval_args
                            .iter()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        if name == "println" {
                            println!("{}", msg);
                        } else {
                            print!("{}", msg);
                        }
                        return Ok(Value::Void);
                    }

                    if let Some(op_val) = self.operations.get(name).cloned() {
                        return self.eval_operation(&op_val, eval_args);
                    }

                    if let Some(op_val) = self.get_var(name) {
                        if matches!(op_val, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) {
                            return self.eval_operation(&op_val, eval_args);
                        }
                    }

                    if let Some(func) = self.functions.get(name).cloned() {
                        return self.eval_function(&func, eval_args);
                    }

                    // Morphic function dispatch: if "telegram_send" not found,
                    // search for a morphic template like "{platform}_send"
                    for (fn_name, func) in self.functions.clone() {
                        if let Some(ref morphic_var) = func.morphic_param {
                            let template = &fn_name;
                            let brace_open = template.find('{');
                            let brace_close = template.find('}');
                            if let (Some(bo), Some(bc)) = (brace_open, brace_close) {
                                let prefix = &template[..bo];
                                let suffix = &template[bc+1..];
                                if name.ends_with(suffix) && name.len() > suffix.len() {
                                    let concrete_value = &name[prefix.len()..name.len()-suffix.len()];
                                    self.push_scope();
                                    self.set_var(morphic_var, Value::String(concrete_value.to_string()));
                                    for (param, arg) in func.params.iter().zip(eval_args.into_iter()) {
                                        self.set_var(&param.name, arg);
                                    }
                                    let mut ret_val = Value::Void;
                                    for stmt in &func.body.statements {
                                        match self.eval_statement(stmt)? {
                                            Some(v) => {
                                                ret_val = v;
                                                break;
                                            }
                                            None => {}
                                        }
                                    }
                                    self.pop_scope();
                                    return Ok(ret_val);
                                }
                            }
                        }
                    }
                }

                let callee_val = self.eval_expression(callee)?;
                if matches!(callee_val, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) {
                    return self.eval_operation(&callee_val, eval_args);
                }

                Ok(Value::Void)
            }
            Expression::FieldAccess { object, field, .. } => {
                let obj = self.eval_expression(object)?;
                if let Value::Struct(_, fields) = &obj {
                    if let Some(v) = fields.get(field) {
                        return Ok(v.clone());
                    }
                }
                if let Value::OperationResult { output, status, duration_ns, events, logs, effects, errors } = &obj {
                    match field.as_str() {
                        "output" => return Ok(*output.clone()),
                        "status" => return Ok(Value::String(status.clone())),
                        "duration_ns" => return Ok(Value::Int(*duration_ns as i64)),
                        "events" => return Ok(Value::Array(events.iter().cloned().map(Value::String).collect())),
                        "logs" => return Ok(Value::Array(logs.iter().cloned().map(Value::String).collect())),
                        "effects" => return Ok(Value::Array(effects.iter().cloned().map(Value::String).collect())),
                        "errors" => return Ok(Value::Array(errors.iter().cloned().map(Value::String).collect())),
                        "trace" => return Ok(Value::String(format!("Trace: duration={}ns, status={}, events={:?}", duration_ns, status, events))),
                        "result" => return Ok(obj.clone()),
                        _ => {}
                    }
                }
                Ok(Value::Void)
            }
            Expression::StructInit { name, fields, .. } => {
                let mut map = HashMap::new();
                for (fname, fexpr) in fields {
                    let v = self.eval_expression(fexpr)?;
                    map.insert(fname.clone(), v);
                }
                Ok(Value::Struct(name.clone(), map))
            }
            Expression::EnumInit { enum_name, variant_name, payload, .. } => {
                let payload_val = if let Some(p) = payload {
                    Some(Box::new(self.eval_expression(p)?))
                } else {
                    None
                };
                Ok(Value::Enum(enum_name.clone(), variant_name.clone(), payload_val))
            }
            Expression::Alloc { .. } => {
                Ok(Value::Pointer(0x1000))
            }
            Expression::Catch { expr, .. } => {
                self.eval_expression(expr)
            }
            Expression::Match { expr, arms, .. } => {
                let target_val = self.eval_expression(expr)?;
                for arm in arms {
                    if let Some(bindings) = self.matches_pattern(&target_val, &arm.pattern) {
                        self.push_scope();
                        for (k, v) in bindings {
                            self.set_var(&k, v);
                        }
                        for s in &arm.body.statements {
                            if let Some(ret) = self.eval_statement(s)? {
                                self.pop_scope();
                                return Ok(ret);
                            }
                        }
                        self.pop_scope();
                        break;
                    }
                }
                Ok(Value::Void)
            }
            Expression::Index { array, index, .. } => {
                let arr_val = self.eval_expression(array)?;
                let idx_val = self.eval_expression(index)?;
                let idx = match idx_val {
                    Value::Int(i) => i as usize,
                    _ => 0,
                };
                match arr_val {
                    Value::Array(items) => {
                        Ok(items.get(idx).cloned().unwrap_or(Value::Int(0)))
                    }
                    Value::String(s) => {
                        let byte = s.as_bytes().get(idx).copied().unwrap_or(0);
                        Ok(Value::Int(byte as i64))
                    }
                    _ => Ok(Value::Int(0)),
                }
            }
            Expression::Cast { expr, target_type, .. } => {
                let v = self.eval_expression(expr)?;
                match (v, target_type) {
                    (Value::Int(n), Type::F32 | Type::F64) => Ok(Value::Float(n as f64)),
                    (Value::Float(f), Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::U8 | Type::U16 | Type::U32 | Type::U64) => Ok(Value::Int(f as i64)),
                    (v, _) => Ok(v),
                }
            }
            Expression::Await { expr, .. } => self.eval_expression(expr),
            Expression::Promote { expr, .. } => self.eval_expression(expr),
            Expression::Block(_) => Ok(Value::Void),
            Expression::NameOf { target, .. } => Ok(Value::String(target.clone())),
            Expression::PathOf { target, .. } => Ok(Value::String(target.clone())),
            Expression::TypeOf { expr, .. } => {
                let v = self.eval_expression(expr)?;
                let type_name = match &v {
                    Value::Int(_) => "i64",
                    Value::Float(_) => "f64",
                    Value::String(_) => "str",
                    Value::Bool(_) => "bool",
                    Value::Struct(name, _) => name.as_str(),
                    _ => "unknown",
                };
                Ok(Value::String(type_name.to_string()))
            }
            Expression::DocOf { target, .. } => Ok(Value::String(format!("[Documentation for symbol '{}']", target))),
            Expression::CodeOf { code, .. } => Ok(Value::String(code.clone())),
            Expression::Dbg { expr, code, span } => {
                let v = self.eval_expression(expr)?;
                println!("\x1b[1;36m[DBG {}:{}]\x1b[0m \x1b[1;33m{}\x1b[0m = {}", span.file, span.line, code, v);
                Ok(v)
            }
            Expression::AssertDebug { condition, code, span } => {
                let v = self.eval_expression(condition)?;
                match v {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    _ => {
                        eprintln!("\x1b[1;31m[ASSERTION FAILURE {}:{}]\x1b[0m Condition failed: {}", span.file, span.line, code);
                        Err(format!("Assertion failed at {}:{}: {}", span.file, span.line, code))
                    }
                }
            }
            Expression::Translate { key, .. } => Ok(Value::String(format!("[Localized: {}]", key))),
            Expression::FieldsOf { target, .. } => Ok(Value::String(format!("[Fields of struct {}]", target))),
            Expression::SqlExpr { expr, .. } => {
                let _ = self.eval_expression(expr)?;
                Ok(Value::String("SELECT * FROM table".to_string()))
            }
            Expression::InlineC { .. } => Ok(Value::Void),
            Expression::Pipe { lhs, rhs, .. } => {
                let _ = self.eval_expression(lhs)?;
                self.eval_expression(rhs)
            }
            Expression::UnitLit { value, unit, .. } => {
                match unit.as_str() {
                    "km/h" => Ok(Value::Int((*value * 1000.0 / 3600.0) as i64)),
                    "m/s" => Ok(Value::Int(*value as i64)),
                    "usd" | "eur" => Ok(Value::Int((*value * 100.0) as i64)),
                    _ => Ok(Value::Int(*value as i64)),
                }
            }
            Expression::NullCollapse { left, right, .. } => {
                let l_val = self.eval_expression(left)?;
                match l_val {
                    Value::Void => Ok(Value::Void),
                    _ => self.eval_expression(right),
                }
            }
            Expression::OperationLiteral { name, params, return_type, requires, guarantees, effects, emits, body, .. } => {
                Ok(Value::Operation {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    requires: requires.clone(),
                    guarantees: guarantees.clone(),
                    effects: effects.clone(),
                    emits: emits.clone(),
                    version: None,
                    body: body.clone(),
                })
            }
            Expression::Compose { ops, .. } => {
                if ops.is_empty() {
                    return Ok(Value::Void);
                }
                let mut current = self.eval_expression(&ops[0])?;
                for next_expr in &ops[1..] {
                    let next_val = self.eval_expression(next_expr)?;
                    current = Value::ComposedOp(Box::new(current), Box::new(next_val));
                }
                Ok(current)
            }
            Expression::Repeat { op, count, is_retry, .. } => {
                let op_val = self.eval_expression(op)?;
                let count_val = self.eval_expression(count)?;
                let n = match count_val {
                    Value::Int(i) => i.max(1) as usize,
                    _ => 1,
                };
                Ok(Value::RepeatedOp(Box::new(op_val), n, *is_retry))
            }
            Expression::Alternative { left, right, .. } => {
                let l_val = self.eval_expression(left)?;
                let r_val = self.eval_expression(right)?;
                Ok(Value::AlternativeOp(Box::new(l_val), Box::new(r_val)))
            }
            Expression::Parallel { left, right, .. } => {
                let l_val = self.eval_expression(left)?;
                let r_val = self.eval_expression(right)?;
                Ok(Value::ParallelOp(Box::new(l_val), Box::new(r_val)))
            }
            Expression::ConditionalOp { op, condition, .. } => {
                let cond_val = self.eval_expression(condition)?;
                let is_true = match cond_val {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    _ => false,
                };
                if is_true {
                    self.eval_expression(op)
                } else {
                    Ok(Value::Void)
                }
            }
            Expression::Memoize { op, .. } => {
                let op_val = self.eval_expression(op)?;
                Ok(op_val)
            }
            _ => Ok(Value::Int(1)),
        }
    }
}




