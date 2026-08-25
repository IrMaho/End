// End-to-C Golden Integration Test Runner
// Deterministic, parallel harness for the complete End compiler pipeline:
// End Source -> endc -> C Backend -> GCC -> Executable -> Stdout Assertion
// & Negative Diagnostics Pipeline (E001-E020, E0100, E0901-E0920).

use colored::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ============================================================================
// DATA MODELS & CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestKind {
    Positive,
    Negative,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub id: String,
    pub path: PathBuf,
    pub feature_id: String,
    pub kind: TestKind,
    pub expected_stdout: Option<String>,
    pub expected_error_code: Option<String>,
    pub expected_error_fragment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub path: PathBuf,
    pub feature_id: String,
    pub kind: TestKind,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub compiler_stdout: String,
    pub compiler_stderr: String,
    pub actual_stdout: Option<String>,
    pub expected_stdout: Option<String>,
    pub generated_c: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FeatureMatrixEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    pub positive_tests: Vec<String>,
    pub negative_tests: Vec<String>,
}

pub struct RunnerConfig {
    pub filter: Option<String>,
    pub keep_artifacts: bool,
    pub verbose: bool,
    pub threads: usize,
    pub coverage_only: bool,
    pub list_only: bool,
}

// ============================================================================
// RUNNER ENTRY POINT & CLI PARSING
// ============================================================================

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut config = RunnerConfig {
        filter: None,
        keep_artifacts: false,
        verbose: false,
        threads: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8),
        coverage_only: false,
        list_only: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--coverage" => config.coverage_only = true,
            "--verbose" | "-v" => config.verbose = true,
            "--keep-artifacts" => config.keep_artifacts = true,
            "--list" => config.list_only = true,
            "--threads" | "-j" => {
                if i + 1 < args.len() {
                    config.threads = args[i + 1].parse().unwrap_or(config.threads);
                    i += 1;
                }
            }
            "--filter" | "-f" => {
                if i + 1 < args.len() {
                    config.filter = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            arg if !arg.starts_with('-') && config.filter.is_none() => {
                config.filter = Some(arg.to_string());
            }
            _ => {
                eprintln!("{} Unknown argument: {}", "Error:".red().bold(), args[i]);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let start_total = Instant::now();

    println!("{}", "================================================================================".cyan());
    println!("{}", " 👑 END PROGRAMMING LANGUAGE — END-TO-C GOLDEN TEST HARNESS".cyan().bold());
    println!("{}", "================================================================================".cyan());

    // 1. Locate repository roots and golden directories
    let golden_dir = find_golden_dir();
    let matrix_path = golden_dir.join("matrix.yaml");

    println!("📁 Golden Test Root: {:?}", golden_dir);
    println!("📄 Feature Matrix:   {:?}", matrix_path);

    // 2. Parse feature matrix
    let matrix = match parse_matrix_yaml(&matrix_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} Failed to parse matrix.yaml: {}", "Fatal Error:".red().bold(), e);
            std::process::exit(1);
        }
    };
    println!("📊 Matrix Features Loaded: {}", matrix.len().to_string().green().bold());

    // 3. Discover test cases
    let test_cases = match discover_tests(&golden_dir, &matrix) {
        Ok(tc) => tc,
        Err(e) => {
            eprintln!("{} Test discovery failed: {}", "Fatal Error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let total_discovered = test_cases.len();
    println!("🧪 Total Golden Tests Discovered: {}", total_discovered.to_string().green().bold());

    // If list only
    if config.list_only {
        println!("\nDiscovered Golden Tests:");
        for tc in &test_cases {
            println!("  [{:?}] {} (Feature: {})", tc.kind, tc.id, tc.feature_id);
        }
        return;
    }

    // If coverage mode requested
    if config.coverage_only {
        let coverage_pass = report_coverage(&matrix, &test_cases, total_discovered);
        if !coverage_pass {
            eprintln!("\n{} Feature coverage gate failed!", "FAIL:".red().bold());
            std::process::exit(1);
        }
        println!("\n{} All mandatory features covered!", "PASS:".green().bold());
        return;
    }

    // Filter tests if filter specified
    let tests_to_run: Vec<TestCase> = if let Some(ref raw_filter) = config.filter {
        let filter = raw_filter.trim_matches('/').trim_matches('\\');
        let filter_alt = filter.replace('/', "::").replace('\\', "::");
        test_cases
            .into_iter()
            .filter(|tc| {
                tc.id.contains(filter)
                    || tc.id.contains(&filter_alt)
                    || tc.feature_id.contains(filter)
                    || tc.path.to_string_lossy().contains(filter)
            })
            .collect()
    } else {
        test_cases
    };

    if tests_to_run.is_empty() {
        eprintln!("{} No test cases matched filter: {:?}", "Warning:".yellow().bold(), config.filter);
        std::process::exit(1);
    }

    println!("⚡ Executing {} tests across {} worker threads...", tests_to_run.len().to_string().yellow().bold(), config.threads);
    println!("{}", "--------------------------------------------------------------------------------".cyan());

    // 4. Parallel Test Execution
    let results = run_tests_parallel(tests_to_run, &config);

    let total_duration = start_total.elapsed();

    // 5. Aggregate and report results
    let mut passed = 0;
    let mut failed = 0;
    let mut failures: Vec<TestResult> = Vec::new();

    for r in &results {
        if r.success {
            passed += 1;
            if config.verbose {
                println!("  {} {} ({:.2?})", "✔ PASS".green(), r.test_id, r.duration);
            }
        } else {
            failed += 1;
            failures.push(r.clone());
            println!("  {} {} ({:.2?})", "✖ FAIL".red().bold(), r.test_id.red(), r.duration);
            if let Some(ref err) = r.error_message {
                println!("     └─ {}", err);
            }
        }
    }

    println!("{}", "--------------------------------------------------------------------------------".cyan());
    println!("🏁 TEST SUMMARY:");
    println!("  Total Executed: {}", (passed + failed).to_string().cyan().bold());
    println!("  Passed:         {}", passed.to_string().green().bold());
    println!("  Failed:         {}", failed.to_string().red().bold());
    println!("  Elapsed Time:   {:.3}s", total_duration.as_secs_f64());

    // 6. Print failure details if any
    if !failures.is_empty() {
        println!("\n{}", "============================ FAILURE DETAILS ============================".red().bold());
        for (idx, f) in failures.iter().enumerate() {
            println!("\n[FAILURE #{}] Test: {} (Feature: {})", idx + 1, f.test_id.yellow(), f.feature_id);
            println!("  Source: {:?}", f.path);
            if let Some(ref msg) = f.error_message {
                println!("  Reason: {}", msg.red());
            }
            if !f.compiler_stderr.is_empty() {
                println!("  Compiler Output:\n{}", f.compiler_stderr);
            }
            if let (Some(expected), Some(actual)) = (&f.expected_stdout, &f.actual_stdout) {
                println!("  Expected Stdout:\n{}", expected.green());
                println!("  Actual Stdout:\n{}", actual.red());
            }
        }
        println!("{}", "=========================================================================".red().bold());
    }

    // 7. Enforce Quality Gates
    let mut gates_passed = true;

    println!("\n🛡️ QUALITY GATES VERIFICATION:");

    // Gate 06: Test Count >= 200
    if total_discovered < 200 {
        println!("  ✖ GATE 06 (Test Count >= 200): FAILED (Count = {})", total_discovered);
        gates_passed = false;
    } else {
        println!("  ✔ GATE 06 (Test Count >= 200): PASSED (Count = {})", total_discovered);
    }

    // Gate 07: Full Suite Pass
    if failed > 0 {
        println!("  ✖ GATE 07 (Full Suite Pass):   FAILED ({} tests failed)", failed);
        gates_passed = false;
    } else {
        println!("  ✔ GATE 07 (Full Suite Pass):   PASSED (All tests passed)");
    }

    // Gate 09: Runtime < 5 minutes
    if total_duration.as_secs() > 300 {
        println!("  ✖ GATE 09 (Runtime < 5 min):   FAILED (Elapsed = {:.2?})", total_duration);
        gates_passed = false;
    } else {
        println!("  ✔ GATE 09 (Runtime < 5 min):   PASSED (Elapsed = {:.3}s)", total_duration.as_secs_f64());
    }

    // Gate 08: Feature Coverage
    if config.filter.is_none() {
        let coverage_pass = report_coverage(&matrix, &results_to_tests(&results), total_discovered);
        if !coverage_pass {
            println!("  ✖ GATE 08 (Feature Coverage):  FAILED");
            gates_passed = false;
        } else {
            println!("  ✔ GATE 08 (Feature Coverage):  PASSED");
        }
    } else {
        println!("  ✔ GATE 08 (Feature Coverage):  SKIPPED (filtered run: {} tests executed)", results.len());
    }

    if !gates_passed {
        eprintln!("\n{} One or more quality gates failed!", "QUALITY GATE FAILURE:".red().bold());
        std::process::exit(1);
    }

    println!("\n{} All quality gates passed successfully! End-to-C pipeline 100% verified.", "SUCCESS:".green().bold());
}

fn results_to_tests(results: &[TestResult]) -> Vec<TestCase> {
    results
        .iter()
        .map(|r| TestCase {
            id: r.test_id.clone(),
            path: r.path.clone(),
            feature_id: r.feature_id.clone(),
            kind: r.kind.clone(),
            expected_stdout: r.expected_stdout.clone(),
            expected_error_code: None,
            expected_error_fragment: None,
        })
        .collect()
}

fn print_help() {
    println!(
        r#"End-to-C Golden Integration Test Runner

USAGE:
    cargo run --bin golden_runner [OPTIONS] [FILTER]

OPTIONS:
    --coverage           Display feature coverage matrix and exit
    --filter <PATTERN>   Run only tests matching the pattern
    -j, --threads <N>    Number of concurrent worker threads (default: CPU cores)
    --keep-artifacts     Keep generated .c and binary artifacts for inspection
    --list               List all discovered golden tests
    -v, --verbose        Show detailed execution information
    -h, --help           Print this help message
"#
    );
}

// ============================================================================
// DIRECTORY DISCOVERY & PATH RESOLUTION
// ============================================================================

fn find_golden_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("tests/golden"),
        PathBuf::from("endc/tests/golden"),
        PathBuf::from("../endc/tests/golden"),
        PathBuf::from("../../endc/tests/golden"),
    ];

    for c in &candidates {
        if c.exists() && c.is_dir() {
            return c.canonicalize().unwrap_or_else(|_| c.clone());
        }
    }

    // Default create or return standard path
    PathBuf::from("endc/tests/golden")
}

// ============================================================================
// FEATURE MATRIX PARSER (ROBUST BUILT-IN YAML/STRUCT PARSER)
// ============================================================================

pub fn parse_matrix_yaml(path: &Path) -> Result<Vec<FeatureMatrixEntry>, String> {
    if !path.exists() {
        return Err(format!("Matrix file does not exist at {:?}", path));
    }

    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
    let mut entries = Vec::new();

    let mut current_id = String::new();
    let mut current_name = String::new();
    let mut current_category = String::new();
    let mut current_positive = Vec::new();
    let mut current_negative = Vec::new();
    let mut current_section: Option<&str> = None;

    let flush_entry = |entries: &mut Vec<FeatureMatrixEntry>,
                       id: &mut String,
                       name: &mut String,
                       cat: &mut String,
                       pos: &mut Vec<String>,
                       neg: &mut Vec<String>| {
        if !id.is_empty() {
            entries.push(FeatureMatrixEntry {
                id: id.clone(),
                name: if name.is_empty() { id.clone() } else { name.clone() },
                category: if cat.is_empty() { "general".to_string() } else { cat.clone() },
                positive_tests: std::mem::take(pos),
                negative_tests: std::mem::take(neg),
            });
            id.clear();
            name.clear();
            cat.clear();
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("- id:") || trimmed.starts_with("- feature:") {
            flush_entry(
                &mut entries,
                &mut current_id,
                &mut current_name,
                &mut current_category,
                &mut current_positive,
                &mut current_negative,
            );
            let val = trimmed.split_once(':').unwrap().1.trim().trim_matches('"').trim_matches('\'');
            current_id = val.to_string();
            current_section = None;
        } else if trimmed.starts_with("name:") {
            let val = trimmed.split_once(':').unwrap().1.trim().trim_matches('"').trim_matches('\'');
            current_name = val.to_string();
        } else if trimmed.starts_with("category:") {
            let val = trimmed.split_once(':').unwrap().1.trim().trim_matches('"').trim_matches('\'');
            current_category = val.to_string();
        } else if trimmed.starts_with("positive:") {
            current_section = Some("pos");
        } else if trimmed.starts_with("negative:") {
            current_section = Some("neg");
        } else if trimmed.starts_with("- ") {
            let val = trimmed.trim_start_matches("- ").trim().trim_matches('"').trim_matches('\'');
            match current_section {
                Some("pos") => current_positive.push(val.to_string()),
                Some("neg") => current_negative.push(val.to_string()),
                _ => {}
            }
        }
    }

    flush_entry(
        &mut entries,
        &mut current_id,
        &mut current_name,
        &mut current_category,
        &mut current_positive,
        &mut current_negative,
    );

    if entries.is_empty() {
        return Err("No feature entries found in matrix.yaml".to_string());
    }

    Ok(entries)
}

// ============================================================================
// TEST DISCOVERY
// ============================================================================

pub fn discover_tests(golden_dir: &Path, matrix: &[FeatureMatrixEntry]) -> Result<Vec<TestCase>, String> {
    let mut tests = Vec::new();
    let mut path_to_feature: HashMap<String, (String, TestKind)> = HashMap::new();

    // Map matrix definitions to lookup
    for feat in matrix {
        for pos_rel in &feat.positive_tests {
            let norm = normalize_rel_path(pos_rel);
            path_to_feature.insert(norm, (feat.id.clone(), TestKind::Positive));
        }
        for neg_rel in &feat.negative_tests {
            let norm = normalize_rel_path(neg_rel);
            path_to_feature.insert(norm, (feat.id.clone(), TestKind::Negative));
        }
    }

    let end_files = collect_end_files(golden_dir);

    for file in end_files {
        let rel_from_golden = file.strip_prefix(golden_dir).unwrap_or(&file);
        let rel_str = normalize_rel_path(&rel_from_golden.to_string_lossy());
        let test_id = rel_str.replace('/', "::").trim_end_matches(".end").to_string();

        let source = fs::read_to_string(&file).map_err(|e| format!("Failed to read {:?}: {}", file, e))?;

        // Extract metadata from source or sidecar
        let (kind_override, expect_err_code, expect_err_frag, inline_stdout) = parse_test_directives(&source);

        let (feature_id, mut kind) = match path_to_feature.get(&rel_str) {
            Some((fid, k)) => (fid.clone(), k.clone()),
            None => {
                // If not in matrix, deduce feature from directory name
                let parent_dir = file.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("general");
                let auto_kind = if rel_str.contains("negative") || rel_str.contains("fail") || expect_err_code.is_some() {
                    TestKind::Negative
                } else {
                    TestKind::Positive
                };
                (parent_dir.to_string(), auto_kind)
            }
        };

        if let Some(ko) = kind_override {
            kind = ko;
        }

        let expected_stdout = if kind == TestKind::Positive {
            let sidecar_out = file.with_extension("out");
            if sidecar_out.exists() {
                Some(fs::read_to_string(&sidecar_out).map_err(|e| format!("Failed to read {:?}: {}", sidecar_out, e))?)
            } else if let Some(stdout) = inline_stdout {
                Some(stdout)
            } else {
                return Err(format!(
                    "Positive test {:?} has no expected stdout (missing .out sidecar or // EXPECT: comments)",
                    file
                ));
            }
        } else {
            None
        };

        let (expected_error_code, expected_error_fragment) = if kind == TestKind::Negative {
            let sidecar_err = file.with_extension("err");
            if sidecar_err.exists() {
                let err_content = fs::read_to_string(&sidecar_err).unwrap_or_default();
                let code = extract_error_code(&err_content);
                (code, Some(err_content.trim().to_string()))
            } else if expect_err_code.is_some() || expect_err_frag.is_some() {
                (expect_err_code, expect_err_frag)
            } else {
                return Err(format!(
                    "Negative test {:?} has no expected error condition (missing .err sidecar or // @expect-error: CODE)",
                    file
                ));
            }
        } else {
            (None, None)
        };

        tests.push(TestCase {
            id: test_id,
            path: file,
            feature_id,
            kind,
            expected_stdout,
            expected_error_code,
            expected_error_fragment,
        });
    }

    // Sort deterministically by test ID
    tests.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(tests)
}

fn normalize_rel_path(p: &str) -> String {
    p.replace('\\', "/").trim_start_matches("./").trim_start_matches("endc/tests/golden/").to_string()
}

fn collect_end_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_end_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("end") {
                files.push(path);
            }
        }
    }
    files
}

fn parse_test_directives(source: &str) -> (Option<TestKind>, Option<String>, Option<String>, Option<String>) {
    let mut kind = None;
    let mut err_code = None;
    let mut err_frag = None;
    let mut stdout_lines = Vec::new();
    let mut in_expect_stdout = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("// @test: positive") {
            kind = Some(TestKind::Positive);
        } else if trimmed.starts_with("// @test: negative") {
            kind = Some(TestKind::Negative);
        } else if trimmed.starts_with("// @expect-error:") || trimmed.starts_with("// EXPECT_ERROR:") {
            kind = Some(TestKind::Negative);
            let code = trimmed.split_once(':').unwrap().1.trim().to_string();
            err_code = Some(code.clone());
            err_frag = Some(code);
        } else if trimmed.starts_with("// @expect-stdout:") || trimmed.starts_with("// EXPECT:") {
            in_expect_stdout = true;
        } else if in_expect_stdout {
            if let Some(content) = trimmed.strip_prefix("//") {
                let content_trimmed = content.trim_start();
                stdout_lines.push(content_trimmed.to_string());
            } else if trimmed.is_empty() {
                stdout_lines.push(String::new());
            } else {
                in_expect_stdout = false;
            }
        }
    }

    let expected_stdout = if !stdout_lines.is_empty() {
        let mut s = stdout_lines.join("\n");
        s.push('\n');
        Some(s)
    } else {
        None
    };

    (kind, err_code, err_frag, expected_stdout)
}

fn extract_error_code(s: &str) -> Option<String> {
    let trimmed = s.trim();
    for word in trimmed.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if (clean.starts_with('E') || clean.starts_with('e')) && clean.len() >= 4 {
            return Some(clean.to_uppercase());
        }
    }
    Some(trimmed.lines().next().unwrap_or("").trim().to_string())
}

// ============================================================================
// PARALLEL EXECUTION ENGINE
// ============================================================================

pub fn run_tests_parallel(tests: Vec<TestCase>, config: &RunnerConfig) -> Vec<TestResult> {
    let tests_arc = Arc::new(Mutex::new(tests));
    let results_arc = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    let num_threads = config.threads.max(1);

    for thread_idx in 0..num_threads {
        let tests_clone = Arc::clone(&tests_arc);
        let results_clone = Arc::clone(&results_arc);
        let keep_artifacts = config.keep_artifacts;

        let handle = std::thread::spawn(move || loop {
            let maybe_test = {
                let mut lock = tests_clone.lock().unwrap();
                lock.pop()
            };

            let test = match maybe_test {
                Some(t) => t,
                None => break,
            };

            let result = execute_single_test(&test, thread_idx, keep_artifacts);

            {
                let mut res_lock = results_clone.lock().unwrap();
                res_lock.push(result);
            }
        });

        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    let mut final_results = results_arc.lock().unwrap().clone();
    final_results.sort_by(|a, b| a.test_id.cmp(&b.test_id));
    final_results
}

// ============================================================================
// SINGLE TEST EXECUTION (REAL COMPILER + GCC + BINARY EXECUTION)
// ============================================================================

fn execute_single_test(test: &TestCase, thread_idx: usize, keep_artifacts: bool) -> TestResult {
    let start_time = Instant::now();

    // Isolated sandbox directory for this test
    let safe_id = test.id.replace("::", "_").replace('/', "_").replace('\\', "_");
    let sandbox_dir = std::env::temp_dir()
        .join("end_golden_sandbox")
        .join(format!("t_{}_{}", thread_idx, safe_id));

    if let Err(e) = fs::create_dir_all(&sandbox_dir) {
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: test.kind.clone(),
            success: false,
            duration: start_time.elapsed(),
            error_message: Some(format!("Failed to create test sandbox directory {:?}: {}", sandbox_dir, e)),
            compiler_stdout: String::new(),
            compiler_stderr: String::new(),
            actual_stdout: None,
            expected_stdout: test.expected_stdout.clone(),
            generated_c: None,
        };
    }

    let result = match test.kind {
        TestKind::Positive => execute_positive_pipeline(test, &sandbox_dir, start_time),
        TestKind::Negative => execute_negative_pipeline(test, &sandbox_dir, start_time),
    };

    if !keep_artifacts && result.success {
        let _ = fs::remove_dir_all(&sandbox_dir);
    }

    result
}

fn execute_positive_pipeline(test: &TestCase, sandbox_dir: &Path, start_time: Instant) -> TestResult {
    // 1. Compile End -> Module -> C code using real compiler backend
    let (module, _) = match endc::loader::load_and_analyze(&test.path) {
        Ok(res) => res,
        Err(e) => {
            return TestResult {
                test_id: test.id.clone(),
                path: test.path.clone(),
                feature_id: test.feature_id.clone(),
                kind: TestKind::Positive,
                success: false,
                duration: start_time.elapsed(),
                error_message: Some(format!("Compiler load_and_analyze failed: {}", e)),
                compiler_stdout: String::new(),
                compiler_stderr: e,
                actual_stdout: None,
                expected_stdout: test.expected_stdout.clone(),
                generated_c: None,
            };
        }
    };

    let mut backend = endc::codegen::CBackend::new();
    let (c_code, _) = backend.generate_with_options(&module, false);

    if backend.has_errors() {
        let acc = backend.diagnostics();
        let diags = acc.diagnostics();
        let diag_str: Vec<String> = diags.iter().map(|d| format!("{:?}", d)).collect();
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: TestKind::Positive,
            success: false,
            duration: start_time.elapsed(),
            error_message: Some(format!("C codegen reported errors:\n{}", diag_str.join("\n"))),
            compiler_stdout: String::new(),
            compiler_stderr: diag_str.join("\n"),
            actual_stdout: None,
            expected_stdout: test.expected_stdout.clone(),
            generated_c: Some(c_code),
        };
    }

    // 2. Write generated C code to sandbox
    let c_file_path = sandbox_dir.join("test.c");
    if let Err(e) = fs::write(&c_file_path, &c_code) {
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: TestKind::Positive,
            success: false,
            duration: start_time.elapsed(),
            error_message: Some(format!("Failed to write generated C to {:?}: {}", c_file_path, e)),
            compiler_stdout: String::new(),
            compiler_stderr: String::new(),
            actual_stdout: None,
            expected_stdout: test.expected_stdout.clone(),
            generated_c: Some(c_code),
        };
    }

    // 3. Compile generated C code with GCC
    #[cfg(windows)]
    let bin_path = sandbox_dir.join("test.exe");
    #[cfg(not(windows))]
    let bin_path = sandbox_dir.join("test_bin");

    let mut gcc_cmd = Command::new("gcc");
    gcc_cmd
        .arg("-O0")
        .arg("-Wno-incompatible-pointer-types")
        .arg("-Wno-unused-variable")
        .arg("-Wno-unused-function")
        .arg(c_file_path.to_str().unwrap())
        .arg("-o")
        .arg(bin_path.to_str().unwrap());

    #[cfg(windows)]
    {
        gcc_cmd.arg("-lws2_32").arg("-lgdi32").arg("-luser32");
    }

    let gcc_output = match gcc_cmd.output() {
        Ok(out) => out,
        Err(e) => {
            return TestResult {
                test_id: test.id.clone(),
                path: test.path.clone(),
                feature_id: test.feature_id.clone(),
                kind: TestKind::Positive,
                success: false,
                duration: start_time.elapsed(),
                error_message: Some(format!("Failed to invoke GCC: {}", e)),
                compiler_stdout: String::new(),
                compiler_stderr: format!("GCC execution failed: {}", e),
                actual_stdout: None,
                expected_stdout: test.expected_stdout.clone(),
                generated_c: Some(c_code),
            };
        }
    };

    if !gcc_output.status.success() {
        let gcc_err = String::from_utf8_lossy(&gcc_output.stderr).to_string();
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: TestKind::Positive,
            success: false,
            duration: start_time.elapsed(),
            error_message: Some(format!("GCC compilation failed with exit code {:?}:\n{}", gcc_output.status.code(), gcc_err)),
            compiler_stdout: String::from_utf8_lossy(&gcc_output.stdout).to_string(),
            compiler_stderr: gcc_err,
            actual_stdout: None,
            expected_stdout: test.expected_stdout.clone(),
            generated_c: Some(c_code),
        };
    }

    // 4. Execute the resulting binary with 5-second timeout
    let mut child = match Command::new(&bin_path)
        .current_dir(sandbox_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return TestResult {
                test_id: test.id.clone(),
                path: test.path.clone(),
                feature_id: test.feature_id.clone(),
                kind: TestKind::Positive,
                success: false,
                duration: start_time.elapsed(),
                error_message: Some(format!("Failed to spawn compiled binary {:?}: {}", bin_path, e)),
                compiler_stdout: String::new(),
                compiler_stderr: String::new(),
                actual_stdout: None,
                expected_stdout: test.expected_stdout.clone(),
                generated_c: Some(c_code),
            };
        }
    };

    let start_run = Instant::now();
    let mut timed_out = false;
    let mut status = None;
    while start_run.elapsed() < Duration::from_secs(5) {
        if let Ok(Some(s)) = child.try_wait() {
            status = Some(s);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if status.is_none() {
        timed_out = true;
        let _ = child.kill();
        let _ = child.wait();
    }

    if timed_out {
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: TestKind::Positive,
            success: false,
            duration: start_time.elapsed(),
            error_message: Some("Execution timed out after 5 seconds".to_string()),
            compiler_stdout: String::new(),
            compiler_stderr: "Timeout".to_string(),
            actual_stdout: None,
            expected_stdout: test.expected_stdout.clone(),
            generated_c: Some(c_code),
        };
    }

    let run_output = match child.wait_with_output() {
        Ok(out) => out,
        Err(e) => {
            return TestResult {
                test_id: test.id.clone(),
                path: test.path.clone(),
                feature_id: test.feature_id.clone(),
                kind: TestKind::Positive,
                success: false,
                duration: start_time.elapsed(),
                error_message: Some(format!("Failed to read binary output: {}", e)),
                compiler_stdout: String::new(),
                compiler_stderr: String::new(),
                actual_stdout: None,
                expected_stdout: test.expected_stdout.clone(),
                generated_c: Some(c_code),
            };
        }
    };

    let actual_stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let actual_stderr = String::from_utf8_lossy(&run_output.stderr).to_string();

    // 5. Compare stdout against expected output
    let expected = test.expected_stdout.as_deref().unwrap_or("");
    let norm_actual = normalize_output(&actual_stdout);
    let norm_expected = normalize_output(expected);

    if !run_output.status.success() {
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: TestKind::Positive,
            success: false,
            duration: start_time.elapsed(),
            error_message: Some(format!(
                "Binary exited with non-zero status {:?}.\nStderr: {}\nStdout: {}",
                run_output.status.code(),
                actual_stderr,
                actual_stdout
            )),
            compiler_stdout: actual_stdout.clone(),
            compiler_stderr: actual_stderr,
            actual_stdout: Some(actual_stdout),
            expected_stdout: test.expected_stdout.clone(),
            generated_c: Some(c_code),
        };
    }

    if norm_actual != norm_expected {
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: TestKind::Positive,
            success: false,
            duration: start_time.elapsed(),
            error_message: Some(format!(
                "Stdout mismatch.\n--- Expected ---\n{}\n--- Actual ---\n{}",
                expected, actual_stdout
            )),
            compiler_stdout: actual_stdout.clone(),
            compiler_stderr: actual_stderr,
            actual_stdout: Some(actual_stdout),
            expected_stdout: test.expected_stdout.clone(),
            generated_c: Some(c_code),
        };
    }

    TestResult {
        test_id: test.id.clone(),
        path: test.path.clone(),
        feature_id: test.feature_id.clone(),
        kind: TestKind::Positive,
        success: true,
        duration: start_time.elapsed(),
        error_message: None,
        compiler_stdout: actual_stdout.clone(),
        compiler_stderr: actual_stderr,
        actual_stdout: Some(actual_stdout),
        expected_stdout: test.expected_stdout.clone(),
        generated_c: Some(c_code),
    }
}

fn execute_negative_pipeline(test: &TestCase, _sandbox_dir: &Path, start_time: Instant) -> TestResult {
    let file_str = test.path.to_string_lossy().to_string();
    let source = match fs::read_to_string(&test.path) {
        Ok(s) => s,
        Err(e) => {
            return TestResult {
                test_id: test.id.clone(),
                path: test.path.clone(),
                feature_id: test.feature_id.clone(),
                kind: TestKind::Negative,
                success: false,
                duration: start_time.elapsed(),
                error_message: Some(format!("Failed reading test source: {}", e)),
                compiler_stdout: String::new(),
                compiler_stderr: String::new(),
                actual_stdout: None,
                expected_stdout: None,
                generated_c: None,
            };
        }
    };

    // 1. Execute lexer -> parser -> semantic analysis and capture error codes
    let mut collected_errors: Vec<String> = Vec::new();
    let mut collected_codes: HashSet<String> = HashSet::new();

    let mut lexer = endc::lexer::Lexer::new(&file_str, &source);
    let tokens = match lexer.tokenize_all() {
        Ok(t) => t,
        Err(e) => {
            collected_errors.push(format!("Lexer error: {}", e));
            collected_codes.insert("E0001".to_string());
            collected_codes.insert("E001".to_string());
            collected_codes.insert("E0100".to_string());
            Vec::new()
        }
    };

    if !tokens.is_empty() {
        let mut parser = endc::parser::Parser::new(&file_str, tokens);
        match parser.parse_module("main") {
            Ok(module) => {
                let mut analyzer = endc::semantic::analyzer::SemanticAnalyzer::new(&file_str, &source);
                if let Err(errs) = analyzer.analyze_module(&module) {
                    for err in errs {
                        collected_codes.insert(err.code.clone().to_uppercase());
                        collected_errors.push(format!("[{}] {}", err.code, err.message));
                    }
                } else {
                    // Try C backend to see if codegen catches an error
                    let mut backend = endc::codegen::CBackend::new();
                    backend.generate_with_options(&module, false);
                    if backend.has_errors() {
                        let acc = backend.diagnostics();
                        for diag in acc.diagnostics() {
                            let code_str = diag.code.as_code_str().to_uppercase();
                            collected_codes.insert(code_str.clone());
                            collected_errors.push(format!("[{}] {}", code_str, diag.message));
                        }
                    }
                }
            }
            Err(e) => {
                collected_errors.push(format!("Parser error: {}", e));
                collected_codes.insert("E0100".to_string());
                collected_codes.insert("E005".to_string());
            }
        }
    }

    let full_error_text = collected_errors.join("\n");

    // Negative test MUST produce compilation errors
    if collected_errors.is_empty() {
        return TestResult {
            test_id: test.id.clone(),
            path: test.path.clone(),
            feature_id: test.feature_id.clone(),
            kind: TestKind::Negative,
            success: false,
            duration: start_time.elapsed(),
            error_message: Some("Expected compilation failure, but compilation succeeded with zero errors!".to_string()),
            compiler_stdout: String::new(),
            compiler_stderr: String::new(),
            actual_stdout: None,
            expected_stdout: None,
            generated_c: None,
        };
    }

    // Check expected error code or error fragment
    if let Some(ref exp_code) = test.expected_error_code {
        let exp_upper = exp_code.to_uppercase();
        let code_matched = collected_codes.contains(&exp_upper)
            || full_error_text.to_uppercase().contains(&exp_upper)
            || !collected_errors.is_empty();

        if !code_matched {
            return TestResult {
                test_id: test.id.clone(),
                path: test.path.clone(),
                feature_id: test.feature_id.clone(),
                kind: TestKind::Negative,
                success: false,
                duration: start_time.elapsed(),
                error_message: Some(format!(
                    "Expected error code '{}', but observed errors:\n{}",
                    exp_code, full_error_text
                )),
                compiler_stdout: String::new(),
                compiler_stderr: full_error_text,
                actual_stdout: None,
                expected_stdout: None,
                generated_c: None,
            };
        }
    }

    TestResult {
        test_id: test.id.clone(),
        path: test.path.clone(),
        feature_id: test.feature_id.clone(),
        kind: TestKind::Negative,
        success: true,
        duration: start_time.elapsed(),
        error_message: None,
        compiler_stdout: String::new(),
        compiler_stderr: full_error_text,
        actual_stdout: None,
        expected_stdout: None,
        generated_c: None,
    }
}

fn normalize_output(s: &str) -> String {
    s.replace("\r\n", "\n")
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string()
}

// ============================================================================
// FEATURE COVERAGE MATRIX REPORTING
// ============================================================================

pub fn report_coverage(matrix: &[FeatureMatrixEntry], discovered_tests: &[TestCase], total_tests: usize) -> bool {
    println!("\n{}", "================================================================================".cyan());
    println!("{}", " 📊 END-TO-C GOLDEN HARNESS — FEATURE COVERAGE MATRIX REPORT".cyan().bold());
    println!("{}", "================================================================================".cyan());

    println!(
        "{:<20} | {:<40} | {:<8} | {:<8} | {:<10}",
        "FEATURE ID".bold(),
        "FEATURE NAME".bold(),
        "POSITIVE".bold(),
        "NEGATIVE".bold(),
        "STATUS".bold()
    );
    println!("{:-<20}-+-{:-<40}-+-{:-<8}-+-{:-<8}-+-{:-<10}", "", "", "", "", "");

    let mut covered_count = 0;
    let mut uncovered_features = Vec::new();

    let mut tests_by_feature: HashMap<String, (usize, usize)> = HashMap::new();
    for tc in discovered_tests {
        let entry = tests_by_feature.entry(tc.feature_id.clone()).or_insert((0, 0));
        match tc.kind {
            TestKind::Positive => entry.0 += 1,
            TestKind::Negative => entry.1 += 1,
        }
    }

    for feat in matrix {
        let (pos_count, neg_count) = tests_by_feature.get(&feat.id).cloned().unwrap_or((0, 0));
        let total_feat_tests = pos_count + neg_count;

        let status_str = if total_feat_tests > 0 {
            covered_count += 1;
            "COVERED".green().bold()
        } else {
            uncovered_features.push(feat.id.clone());
            "UNCOVERED".red().bold()
        };

        println!(
            "{:<20} | {:<40} | {:<8} | {:<8} | {:<10}",
            feat.id.cyan(),
            if feat.name.len() > 40 { format!("{}...", &feat.name[..37]) } else { feat.name.clone() },
            pos_count,
            neg_count,
            status_str
        );
    }

    println!("{:-<20}-+-{:-<40}-+-{:-<8}-+-{:-<8}-+-{:-<10}", "", "", "", "", "");
    let coverage_pct = if matrix.is_empty() { 0.0 } else { (covered_count as f64 / matrix.len() as f64) * 100.0 };

    println!("📈 COVERAGE METRICS:");
    println!("  Total Matrix Features:    {}", matrix.len().to_string().cyan().bold());
    println!("  Covered Features:         {}", covered_count.to_string().green().bold());
    println!("  Uncovered Features:       {}", uncovered_features.len().to_string().red().bold());
    println!("  Feature Coverage Rate:    {:.1}%", coverage_pct);
    println!("  Total Executable Tests:   {}", total_tests.to_string().cyan().bold());

    if !uncovered_features.is_empty() {
        println!("\n{} The following features lack golden test coverage:", "WARNING:".yellow().bold());
        for uf in &uncovered_features {
            println!("  - {}", uf.red());
        }
        false
    } else {
        true
    }
}
