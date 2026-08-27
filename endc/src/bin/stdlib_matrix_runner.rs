use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleStatus {
    REAL,
    PARTIAL,
    FAKE,
    EXPERIMENTAL,
}

impl ModuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleStatus::REAL => "REAL",
            ModuleStatus::PARTIAL => "PARTIAL",
            ModuleStatus::FAKE => "FAKE",
            ModuleStatus::EXPERIMENTAL => "EXPERIMENTAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixEntry {
    pub name: String,
    pub status: ModuleStatus,
    pub depends_on: Option<Vec<String>>,
    pub integration_test: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdlibMatrix {
    pub version: Option<String>,
    pub modules: Vec<MatrixEntry>,
}

fn main() {
    let handle = std::thread::Builder::new()
        .name("stdlib_matrix_runner".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(run)
        .expect("Failed to spawn runner thread with 16MB stack");
    handle.join().expect("Runner thread panicked");
}

fn run() {
    let args: Vec<String> = std::env::args().collect();
    let _coverage_mode = args.iter().any(|a| a == "--coverage");

    println!("{}", "================================================================================".cyan());
    println!("{}", "           END PROGRAMMING LANGUAGE — STDLIB CAPABILITY MATRIX RUNNER          ".cyan().bold());
    println!("{}", "================================================================================".cyan());

    // 1. Locate stdlib directory and matrix file
    let workspace_root = find_workspace_root();
    let std_dir = workspace_root.join("std");
    let matrix_file = workspace_root.join("endc").join("tests").join("stdlib_matrix.yaml");

    println!("Workspace Root: {}", workspace_root.display());
    println!("Stdlib Dir:     {}", std_dir.display());
    println!("Matrix File:    {}", matrix_file.display());
    println!();

    if !std_dir.exists() {
        eprintln!("{} stdlib directory not found at '{}'", "Error:".red().bold(), std_dir.display());
        exit(1);
    }

    if !matrix_file.exists() {
        eprintln!("{} stdlib matrix not found at '{}'", "Error:".red().bold(), matrix_file.display());
        exit(1);
    }

    // 2. Discover all std/**/*.end files on disk
    let discovered_files = discover_std_files(&std_dir, &workspace_root);
    println!("Discovered {} .end files on disk.", discovered_files.len().to_string().green().bold());

    // 3. Load stdlib_matrix.yaml
    let matrix_yaml = fs::read_to_string(&matrix_file).expect("Failed to read matrix file");
    let matrix: StdlibMatrix = serde_yaml::from_str(&matrix_yaml).expect("Failed to parse stdlib_matrix.yaml");

    println!("Loaded matrix with {} catalogued modules.", matrix.modules.len().to_string().green().bold());
    println!();

    // 4. Validate 1-to-1 Correspondence (No missing, no unknown, no duplicates)
    let mut matrix_names = HashSet::new();
    let mut duplicates = Vec::new();
    let mut violations = Vec::new();

    for entry in &matrix.modules {
        let norm_name = normalize_path(&entry.name);
        if !matrix_names.insert(norm_name.clone()) {
            duplicates.push(norm_name);
        }
    }

    if !duplicates.is_empty() {
        eprintln!("{} Duplicate module entries found in matrix:", "Validation Error:".red().bold());
        for dup in &duplicates {
            eprintln!("  - {}", dup.red());
        }
        exit(1);
    }

    let mut missing_from_matrix = Vec::new();
    for disk_file in &discovered_files {
        if !matrix_names.contains(disk_file) {
            missing_from_matrix.push(disk_file.clone());
        }
    }

    let mut unknown_in_matrix = Vec::new();
    for entry in &matrix.modules {
        let norm_name = normalize_path(&entry.name);
        if !discovered_files.contains(&norm_name) {
            unknown_in_matrix.push(norm_name);
        }
    }

    if !missing_from_matrix.is_empty() {
        eprintln!("{} Files on disk missing from stdlib_matrix.yaml:", "GATE 1 VIOLATION:".red().bold());
        for m in &missing_from_matrix {
            eprintln!("  ✖ {}", m.red());
        }
        violations.push("Missing files in matrix".to_string());
    }

    if !unknown_in_matrix.is_empty() {
        eprintln!("{} Matrix references nonexistent .end files:", "GATE 1 VIOLATION:".red().bold());
        for u in &unknown_in_matrix {
            eprintln!("  ✖ {}", u.red());
        }
        violations.push("Unknown files in matrix".to_string());
    }

    // 5. Execute Integration Tests for REAL Modules & Validate Evidence
    let mut real_count = 0;
    let mut partial_count = 0;
    let mut fake_count = 0;
    let mut experimental_count = 0;
    let mut tested_count = 0;
    let mut failed_tests = Vec::new();

    println!("{:<35} {:<15} {:<25} {:<10}", "MODULE", "STATUS", "INTEGRATION TEST", "RESULT");
    println!("{:-<90}", "");

    for entry in &matrix.modules {
        let norm_name = normalize_path(&entry.name);
        match entry.status {
            ModuleStatus::REAL => {
                real_count += 1;
                let test_files = entry.integration_test.as_ref();
                if test_files.is_none() || test_files.unwrap().is_empty() {
                    eprintln!(
                        "{:<35} {:<15} {:<25} {:<10}",
                        norm_name.red(),
                        "REAL".red().bold(),
                        "MISSING".red(),
                        "FAILED".red().bold()
                    );
                    violations.push(format!("REAL module '{}' has no integration test", norm_name));
                    continue;
                }

                let tests = test_files.unwrap();
                let mut all_tests_passed = true;
                let mut test_desc = String::new();

                for test_rel in tests {
                    let test_path = workspace_root.join(test_rel);
                    if !test_path.exists() {
                        all_tests_passed = false;
                        failed_tests.push((norm_name.clone(), format!("Test file not found: {}", test_rel)));
                        break;
                    }

                    test_desc = test_rel.clone();
                    // Run test using End interpreter
                    let test_ok = run_end_test(&test_path, &workspace_root);
                    if !test_ok {
                        all_tests_passed = false;
                        failed_tests.push((norm_name.clone(), format!("Execution failed: {}", test_rel)));
                        break;
                    }
                }

                if all_tests_passed {
                    tested_count += 1;
                    println!(
                        "{:<35} {:<15} {:<25} {:<10}",
                        norm_name,
                        "REAL".green().bold(),
                        test_desc,
                        "PASS".green().bold()
                    );
                } else {
                    println!(
                        "{:<35} {:<15} {:<25} {:<10}",
                        norm_name.red(),
                        "REAL".red().bold(),
                        test_desc.red(),
                        "FAIL".red().bold()
                    );
                }
            }
            ModuleStatus::PARTIAL => {
                partial_count += 1;
                println!(
                    "{:<35} {:<15} {:<25} {:<10}",
                    norm_name.yellow(),
                    "PARTIAL".yellow(),
                    "-",
                    "-"
                );
            }
            ModuleStatus::FAKE => {
                fake_count += 1;
                println!(
                    "{:<35} {:<15} {:<25} {:<10}",
                    norm_name.red(),
                    "FAKE".red().bold(),
                    "-",
                    "-"
                );
            }
            ModuleStatus::EXPERIMENTAL => {
                experimental_count += 1;
                println!(
                    "{:<35} {:<15} {:<25} {:<10}",
                    norm_name.blue(),
                    "EXPERIMENTAL".blue(),
                    "-",
                    "-"
                );
            }
        }
    }

    println!("{:-<90}", "");
    println!();

    // 6. Coverage and Statistics Summary
    let total_modules = matrix.modules.len();
    let real_pct = if total_modules > 0 {
        (real_count as f64 / total_modules as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", "================================================================================".cyan());
    println!("{}", "                         STDLIB CAPABILITY SUMMARY                              ".cyan().bold());
    println!("{}", "================================================================================".cyan());
    println!("Total Catalogued Modules: {}", total_modules);
    println!("  - REAL:                 {} ({:.1}%)", real_count.to_string().green().bold(), real_pct);
    println!("  - PARTIAL:              {}", partial_count.to_string().yellow());
    println!("  - EXPERIMENTAL:         {}", experimental_count.to_string().blue());
    println!("  - FAKE:                 {}", fake_count.to_string().red().bold());
    println!();
    println!("REAL Integration Tests Executed: {} / {}", tested_count.to_string().green().bold(), real_count);
    println!();

    if !failed_tests.is_empty() {
        eprintln!("{}", "✖ FAILED REAL INTEGRATION TESTS:".red().bold());
        for (m, reason) in &failed_tests {
            eprintln!("  - {}: {}", m.red().bold(), reason);
        }
        violations.push(format!("{} integration tests failed", failed_tests.len()));
    }

    if !violations.is_empty() {
        println!("{}", "================================================================================".red());
        println!("{}", "                 ✖ STDLIB MATRIX QUALITY GATE ENFORCEMENT FAILED                ".red().bold());
        println!("{}", "================================================================================".red());
        for v in &violations {
            eprintln!("  • {}", v.red().bold());
        }
        exit(1);
    }

    println!("{}", "================================================================================".green());
    println!("{}", "                 ✔ STDLIB MATRIX QUALITY GATES VERIFIED (PASS)                  ".green().bold());
    println!("{}", "================================================================================".green());
}

fn normalize_path(p: &str) -> String {
    p.replace('\\', "/").trim_start_matches("./").to_string()
}

fn find_workspace_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("Failed to get current directory");
    loop {
        if dir.join("std").exists() && dir.join("endc").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    // Fallback
    PathBuf::from("c:/Users/ASUS/Desktop/flutter_project/end")
}

fn discover_std_files(std_dir: &Path, workspace_root: &Path) -> HashSet<String> {
    let mut files = HashSet::new();
    collect_end_files_recursive(std_dir, workspace_root, &mut files);
    files
}

fn collect_end_files_recursive(dir: &Path, workspace_root: &Path, files: &mut HashSet<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_end_files_recursive(&path, workspace_root, files);
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "end") {
                if let Ok(rel) = path.strip_prefix(workspace_root) {
                    files.insert(normalize_path(&rel.to_string_lossy()));
                }
            }
        }
    }
}

fn run_end_test(test_file: &Path, _workspace_root: &Path) -> bool {
    let test_buf = test_file.to_path_buf();
    let (module, _) = match endc::loader::load_and_analyze(&test_buf) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Load & Analyze failed for '{}': {}", test_file.display(), e);
            return false;
        }
    };

    // Run in interpreter
    let mut interp = endc::codegen::interpreter::Interpreter::new();
    match interp.run(&module) {
        Ok(endc::codegen::interpreter::Value::Int(code)) => {
            if code != 0 {
                eprintln!("Test '{}' returned non-zero exit code: {}", test_file.display(), code);
            }
            code == 0
        }
        Ok(endc::codegen::interpreter::Value::Bool(b)) => {
            if !b {
                eprintln!("Test '{}' returned false", test_file.display());
            }
            b
        }
        Ok(endc::codegen::interpreter::Value::Void) => true,
        Ok(_) => true,
        Err(e) => {
            eprintln!("Interpreter error running '{}': {}", test_file.display(), e);
            false
        }
    }
}
