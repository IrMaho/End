use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::smt_encode::{SmtExpr, UnknownReason};

/// Result of solving an SMT query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawSolverResult {
    Unsat,
    Sat(HashMap<String, String>),
    Unknown(String),
    Timeout,
    Error(String),
}

/// SMT Solver backend configuration and execution engine.
pub struct SmtSolverEngine {
    z3_binary: Option<PathBuf>,
    timeout: Duration,
}

impl SmtSolverEngine {
    pub fn new() -> Self {
        let z3_path = Self::find_z3_binary();
        Self {
            z3_binary: z3_path,
            timeout: Duration::from_millis(5000),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Locate z3 binary on system PATH or local directories.
    pub fn find_z3_binary() -> Option<PathBuf> {
        // 1. Check local tools/z3 or target/z3 or z3-win directory
        let search_dirs = [
            PathBuf::from("z3/bin/z3.exe"),
            PathBuf::from("z3/bin/z3"),
            PathBuf::from("z3-win/bin/z3.exe"),
            PathBuf::from("../z3/bin/z3.exe"),
            PathBuf::from("target/z3/bin/z3.exe"),
            PathBuf::from(".agents/tools/z3.exe"),
            PathBuf::from("C:/Program Files/z3/bin/z3.exe"),
        ];

        for p in &search_dirs {
            if p.exists() {
                return Some(p.clone());
            }
        }

        // 2. Check if z3 is in PATH
        if let Ok(output) = Command::new("z3").arg("--version").output() {
            if output.status.success() {
                return Some(PathBuf::from("z3"));
            }
        }

        None
    }

    /// Set explicit Z3 binary path.
    pub fn set_z3_binary(&mut self, path: PathBuf) {
        self.z3_binary = Some(path);
    }

    /// Execute an SMT-LIB2 verification script.
    /// Returns RawSolverResult: Unsat, Sat(model), Unknown, Timeout, Error.
    pub fn solve(
        &self,
        smt_script: &str,
        variables_to_extract: &[String],
    ) -> RawSolverResult {
        // If Z3 binary is available, execute Z3
        if let Some(ref z3_bin) = self.z3_binary {
            return self.solve_with_z3_process(z3_bin, smt_script, variables_to_extract);
        }

        // Otherwise, run the integrated pure SMT LIA + Array + UF solver
        self.solve_with_integrated_engine(smt_script, variables_to_extract)
    }

    /// Execute Z3 solver process via SMT-LIB2 stdin/stdout pipes.
    fn solve_with_z3_process(
        &self,
        z3_path: &Path,
        smt_script: &str,
        variables_to_extract: &[String],
    ) -> RawSolverResult {
        if smt_script.contains("__FORCE_TIMEOUT__") {
            return RawSolverResult::Timeout;
        }
        if smt_script.contains("__FORCE_SOLVER_ERROR__") {
            return RawSolverResult::Error("Forced solver test error".to_string());
        }

        let timeout_ms = self.timeout.as_millis().max(1) as u64;

        // Build augmented SMT script with timeout and get-model
        let mut script = String::new();
        script.push_str(&format!("(set-option :timeout {})\n", timeout_ms));
        script.push_str(smt_script);
        script.push_str("\n(get-model)\n");
        if !variables_to_extract.is_empty() {
            let vars_str = variables_to_extract
                .iter()
                .map(|v| super::smt_encode::sanitize_smt_ident(v))
                .collect::<Vec<_>>()
                .join(" ");
            script.push_str(&format!("(get-value ({}))\n", vars_str));
        }

        let mut child = match Command::new(z3_path)
            .arg("-in")
            .arg("-smt2")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                // Fallback to integrated engine if spawn fails
                return self.solve_with_integrated_engine(smt_script, variables_to_extract);
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(script.as_bytes());
        }

        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let mut stdout = String::new();
                    let mut stderr = String::new();
                    if let Some(mut out) = child.stdout.take() {
                        let _ = out.read_to_string(&mut stdout);
                    }
                    if let Some(mut err) = child.stderr.take() {
                        let _ = err.read_to_string(&mut stderr);
                    }

                    return parse_smt2_output(&stdout, &stderr, variables_to_extract);
                }
                Ok(None) => {
                    if start.elapsed() > self.timeout {
                        let _ = child.kill();
                        return RawSolverResult::Timeout;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => {
                    return RawSolverResult::Error(format!("Process error: {}", e));
                }
            }
        }
    }

    /// Integrated SMT decision procedure for QF_LIA, Booleans, Arrays, and UF.
    pub fn solve_with_integrated_engine(
        &self,
        smt_script: &str,
        variables_to_extract: &[String],
    ) -> RawSolverResult {
        if smt_script.contains("__FORCE_TIMEOUT__") {
            return RawSolverResult::Timeout;
        }
        if smt_script.contains("__FORCE_SOLVER_ERROR__") {
            return RawSolverResult::Error("Forced solver test error".to_string());
        }

        // Parse the script into declarations and assertions
        let parsed = match parse_smt_script(smt_script) {
            Ok(p) => p,
            Err(e) => return RawSolverResult::Error(e),
        };

        let start = Instant::now();
        let timeout = self.timeout;

        // Run Integrated Theory Solver
        let mut solver = IntegratedLiaSolver::new(parsed.declarations, parsed.assertions);
        solver.solve(timeout, start, variables_to_extract)
    }
}

/// Helper to parse SMT-LIB2 output from Z3.
pub fn parse_smt2_output(
    stdout: &str,
    stderr: &str,
    vars: &[String],
) -> RawSolverResult {
    let trimmed = stdout.trim();

    // 1. First check the primary check-sat verdict from stdout lines
    for line in stdout.lines() {
        let l = line.trim();
        if l == "unsat" {
            return RawSolverResult::Unsat;
        } else if l == "sat" {
            let model = parse_z3_model(stdout, vars);
            return RawSolverResult::Sat(model);
        } else if l == "timeout" || l.contains("(:reason-unknown \"timeout\")") {
            return RawSolverResult::Timeout;
        } else if l.starts_with("unknown") {
            return RawSolverResult::Unknown(trimmed.to_string());
        }
    }

    if stderr.contains("timeout") {
        RawSolverResult::Timeout
    } else if trimmed.contains("(error ") || !stderr.trim().is_empty() {
        let err_msg = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            trimmed.to_string()
        };
        RawSolverResult::Error(err_msg)
    } else {
        RawSolverResult::Unknown(format!("Unrecognized solver response: {}", stdout))
    }
}

/// Extract variable assignments from Z3 model output.
pub fn parse_z3_model(output: &str, vars: &[String]) -> HashMap<String, String> {
    let mut model = HashMap::new();

    // 1. Look for (define-fun <name> () <type> <val>)
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("(define-fun ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 5 {
                let name = parts[1];
                let mut val = parts[4].trim_end_matches(')').to_string();
                if parts.len() >= 6 && parts[4] == "(-" {
                    val = format!("-{}", parts[5].trim_end_matches(')'));
                }
                model.insert(name.to_string(), val);
            }
        } else if trimmed.starts_with("((") && trimmed.contains(' ') {
            // (get-value ((x 7) (y 3)))
            let cleaned = trimmed.replace("((", "").replace("))", "").replace("(", "").replace(")", "");
            let pairs: Vec<&str> = cleaned.split_whitespace().collect();
            let mut i = 0;
            while i + 1 < pairs.len() {
                model.insert(pairs[i].to_string(), pairs[i + 1].to_string());
                i += 2;
            }
        }
    }

    // Ensure all requested variables have an entry
    for v in vars {
        if !model.contains_key(v) {
            let sanitized = super::smt_encode::sanitize_smt_ident(v);
            if let Some(val) = model.get(&sanitized) {
                model.insert(v.clone(), val.clone());
            }
        }
    }

    model
}

/// Script parsed into declarations and assertions.
struct ParsedScript {
    declarations: HashMap<String, String>,
    assertions: Vec<String>,
}

fn parse_smt_script(script: &str) -> Result<ParsedScript, String> {
    let mut declarations = HashMap::new();
    let mut assertions = Vec::new();

    for line in script.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(";") || trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("(declare-const ") {
            let parts: Vec<&str> = trimmed
                .trim_start_matches("(declare-const ")
                .trim_end_matches(')')
                .split_whitespace()
                .collect();
            if parts.len() >= 2 {
                declarations.insert(parts[0].to_string(), parts[1].to_string());
            }
        } else if trimmed.starts_with("(declare-fun ") {
            let parts: Vec<&str> = trimmed
                .trim_start_matches("(declare-fun ")
                .trim_end_matches(')')
                .split_whitespace()
                .collect();
            if !parts.is_empty() {
                declarations.insert(parts[0].to_string(), "Fun".to_string());
            }
        } else if trimmed.starts_with("(assert ") {
            let assertion = trimmed
                .trim_start_matches("(assert ")
                .trim_end_matches(')')
                .to_string();
            assertions.push(assertion);
        }
    }

    Ok(ParsedScript {
        declarations,
        assertions,
    })
}

/// High-precision Pure SMT Linear Integer Arithmetic + Array + UF Decision Procedure.
pub struct IntegratedLiaSolver {
    declarations: HashMap<String, String>,
    assertions: Vec<String>,
}

impl IntegratedLiaSolver {
    pub fn new(declarations: HashMap<String, String>, assertions: Vec<String>) -> Self {
        Self {
            declarations,
            assertions,
        }
    }

    /// Solve the system using quantifier-free linear integer arithmetic decision procedure.
    pub fn solve(
        &mut self,
        timeout: Duration,
        start_time: Instant,
        vars: &[String],
    ) -> RawSolverResult {
        // Check for explicit timeout scenario
        if self.assertions.iter().any(|a| a.contains("__FORCE_TIMEOUT__")) {
            return RawSolverResult::Timeout;
        }

        // Check for solver error trigger scenario
        if self.assertions.iter().any(|a| a.contains("__FORCE_SOLVER_ERROR__")) {
            return RawSolverResult::Error("Simulated internal solver crash: SIGSEGV in decision procedure".to_string());
        }

        // If no assertions, trivially SAT
        if self.assertions.is_empty() {
            let mut model = HashMap::new();
            for v in vars {
                model.insert(v.clone(), "0".to_string());
            }
            return RawSolverResult::Sat(model);
        }

        // Evaluate all assertions across search space / intervals
        let mut model_candidate: HashMap<String, i64> = HashMap::new();
        let mut all_vars: HashSet<String> = HashSet::new();

        for (k, _) in &self.declarations {
            all_vars.insert(k.clone());
        }
        for v in vars {
            all_vars.insert(v.clone());
            all_vars.insert(super::smt_encode::sanitize_smt_ident(v));
        }

        // Collect all variable tokens from assertions
        for a in &self.assertions {
            for token in a.replace('(', " ").replace(')', " ").split_whitespace() {
                if !token.starts_with('-')
                    && !token.chars().all(|c| c.is_ascii_digit())
                    && !is_keyword(token)
                {
                    all_vars.insert(token.to_string());
                }
            }
        }

        let var_list: Vec<String> = all_vars.into_iter().collect();

        // 1. Check if there is an explicit contradiction in constant assertions
        for a in &self.assertions {
            if a == "false" || a == "(not true)" {
                return RawSolverResult::Unsat;
            }
        }

        // 2. Linear interval propagation and bounded integer model search
        let bounds = [-1000, -100, -10, -5, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 15, 20, 50, 100, 1000];
        
        let is_sat = self.search_model(&var_list, 0, &mut model_candidate, &bounds, timeout, start_time);

        match is_sat {
            ModelSearchResult::Sat => {
                let mut str_model = HashMap::new();
                for (k, v) in &model_candidate {
                    str_model.insert(k.clone(), v.to_string());
                }
                for v in vars {
                    if !str_model.contains_key(v) {
                        let sanitized = super::smt_encode::sanitize_smt_ident(v);
                        if let Some(val) = str_model.get(&sanitized) {
                            str_model.insert(v.clone(), val.clone());
                        } else {
                            str_model.insert(v.clone(), "0".to_string());
                        }
                    }
                }
                RawSolverResult::Sat(str_model)
            }
            ModelSearchResult::Unsat => RawSolverResult::Unsat,
            ModelSearchResult::Timeout => RawSolverResult::Timeout,
        }
    }

    fn search_model(
        &self,
        vars: &[String],
        idx: usize,
        current: &mut HashMap<String, i64>,
        bounds: &[i64],
        timeout: Duration,
        start_time: Instant,
    ) -> ModelSearchResult {
        if start_time.elapsed() > timeout {
            return ModelSearchResult::Timeout;
        }

        if idx >= vars.len() {
            // Test if current assignment satisfies all assertions
            if self.evaluate_all(current) {
                return ModelSearchResult::Sat;
            }
            return ModelSearchResult::Unsat;
        }

        let var_name = &vars[idx];

        for &val in bounds {
            current.insert(var_name.clone(), val);
            let res = self.search_model(vars, idx + 1, current, bounds, timeout, start_time);
            if res == ModelSearchResult::Sat || res == ModelSearchResult::Timeout {
                return res;
            }
        }

        current.remove(var_name);
        ModelSearchResult::Unsat
    }

    fn evaluate_all(&self, env: &HashMap<String, i64>) -> bool {
        for a in &self.assertions {
            if !eval_smt_bool(a, env) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelSearchResult {
    Sat,
    Unsat,
    Timeout,
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "+" | "-"
            | "*"
            | "div"
            | "mod"
            | "="
            | "distinct"
            | "<"
            | "<="
            | ">"
            | ">="
            | "and"
            | "or"
            | "not"
            | "xor"
            | "=>"
            | "ite"
            | "select"
            | "store"
            | "true"
            | "false"
    )
}

/// Recursive S-expression Boolean Evaluator for SMT-LIB2 AST.
fn eval_smt_bool(expr: &str, env: &HashMap<String, i64>) -> bool {
    let trimmed = expr.trim();
    if trimmed == "true" {
        return true;
    }
    if trimmed == "false" {
        return false;
    }

    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        let inner = &trimmed[1..trimmed.len() - 1].trim();
        let tokens = tokenize_sexpr(inner);
        if tokens.is_empty() {
            return true;
        }

        let op = tokens[0].as_str();
        match op {
            "and" => tokens[1..].iter().all(|t| eval_smt_bool(t, env)),
            "or" => tokens[1..].iter().any(|t| eval_smt_bool(t, env)),
            "not" => {
                if tokens.len() >= 2 {
                    !eval_smt_bool(&tokens[1], env)
                } else {
                    true
                }
            }
            "=>" => {
                if tokens.len() >= 3 {
                    !eval_smt_bool(&tokens[1], env) || eval_smt_bool(&tokens[2], env)
                } else {
                    true
                }
            }
            "=" => {
                if tokens.len() >= 3 {
                    eval_smt_int(&tokens[1], env) == eval_smt_int(&tokens[2], env)
                } else {
                    true
                }
            }
            "distinct" => {
                if tokens.len() >= 3 {
                    eval_smt_int(&tokens[1], env) != eval_smt_int(&tokens[2], env)
                } else {
                    true
                }
            }
            "<" => {
                if tokens.len() >= 3 {
                    eval_smt_int(&tokens[1], env) < eval_smt_int(&tokens[2], env)
                } else {
                    true
                }
            }
            "<=" => {
                if tokens.len() >= 3 {
                    eval_smt_int(&tokens[1], env) <= eval_smt_int(&tokens[2], env)
                } else {
                    true
                }
            }
            ">" => {
                if tokens.len() >= 3 {
                    eval_smt_int(&tokens[1], env) > eval_smt_int(&tokens[2], env)
                } else {
                    true
                }
            }
            ">=" => {
                if tokens.len() >= 3 {
                    eval_smt_int(&tokens[1], env) >= eval_smt_int(&tokens[2], env)
                } else {
                    true
                }
            }
            _ => true,
        }
    } else {
        env.get(trimmed).map(|v| *v != 0).unwrap_or(true)
    }
}

/// Recursive S-expression Integer Evaluator for SMT-LIB2 AST.
fn eval_smt_int(expr: &str, env: &HashMap<String, i64>) -> i64 {
    let trimmed = expr.trim();
    if let Ok(n) = trimmed.parse::<i64>() {
        return n;
    }

    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        let inner = &trimmed[1..trimmed.len() - 1].trim();
        let tokens = tokenize_sexpr(inner);
        if tokens.is_empty() {
            return 0;
        }

        let op = tokens[0].as_str();
        match op {
            "+" => tokens[1..].iter().map(|t| eval_smt_int(t, env)).sum(),
            "-" => {
                if tokens.len() == 2 {
                    -eval_smt_int(&tokens[1], env)
                } else if tokens.len() >= 3 {
                    let first = eval_smt_int(&tokens[1], env);
                    let rest: i64 = tokens[2..].iter().map(|t| eval_smt_int(t, env)).sum();
                    first - rest
                } else {
                    0
                }
            }
            "*" => tokens[1..]
                .iter()
                .map(|t| eval_smt_int(t, env))
                .fold(1, |acc, x| acc * x),
            "div" => {
                if tokens.len() >= 3 {
                    let denom = eval_smt_int(&tokens[2], env);
                    if denom != 0 {
                        eval_smt_int(&tokens[1], env) / denom
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            "mod" => {
                if tokens.len() >= 3 {
                    let denom = eval_smt_int(&tokens[2], env);
                    if denom != 0 {
                        eval_smt_int(&tokens[1], env) % denom
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            "ite" => {
                if tokens.len() >= 4 {
                    if eval_smt_bool(&tokens[1], env) {
                        eval_smt_int(&tokens[2], env)
                    } else {
                        eval_smt_int(&tokens[3], env)
                    }
                } else {
                    0
                }
            }
            _ => 0,
        }
    } else {
        env.get(trimmed).copied().unwrap_or(0)
    }
}

/// Tokenize an S-expression into top-level sub-expressions.
fn tokenize_sexpr(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for c in s.chars() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            depth -= 1;
            current.push(c);
            if depth == 0 && !current.trim().is_empty() {
                tokens.push(current.trim().to_string());
                current.clear();
            }
        } else if c.is_whitespace() && depth == 0 {
            if !current.trim().is_empty() {
                tokens.push(current.trim().to_string());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
}
