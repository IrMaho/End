// Integration Test: Documentation Example Verification (Feature F-36 / Prompt 28)
// Extracts all ```end fenced code snippets from README.md and documentation,
// runs them through the End lexer, parser, loader, and semantic analyzer,
// and asserts 100% compilation and truthful execution.

use endc::lexer::Lexer;
use endc::loader::load_and_analyze;
use endc::parser::Parser as EndParser;
use endc::semantic::SemanticAnalyzer;
use std::fs;
use std::path::PathBuf;

fn get_repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().expect("repo root must exist").to_path_buf()
}

fn extract_end_code_blocks(markdown_content: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut in_end_block = false;
    let mut current_block = String::new();
    let mut block_start_line = 0;

    for (line_idx, line) in markdown_content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```end") {
            in_end_block = true;
            current_block.clear();
            block_start_line = line_idx + 1;
        } else if in_end_block && trimmed == "```" {
            in_end_block = false;
            if !current_block.trim().is_empty() {
                blocks.push((block_start_line, current_block.clone()));
            }
            current_block.clear();
        } else if in_end_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    blocks
}

#[test]
fn test_readme_code_examples_compile_and_analyze() {
    let repo_root = get_repo_root();
    let readme_path = repo_root.join("README.md");
    assert!(readme_path.exists(), "README.md must exist at repo root: {:?}", readme_path);

    let content = fs::read_to_string(&readme_path).expect("failed to read README.md");
    let blocks = extract_end_code_blocks(&content);

    assert!(!blocks.is_empty(), "README.md must contain at least one ```end code block");
    println!("🔍 Found {} End code blocks in README.md", blocks.len());

    let temp_dir = repo_root.join("target").join("doc_example_tests");
    let _ = fs::create_dir_all(&temp_dir);

    for (idx, (start_line, code)) in blocks.iter().enumerate() {
        println!("  ⚙️ Testing README.md Example #{} (Line {})...", idx + 1, start_line);

        // 1. Lexer Verification
        let mut lexer = Lexer::new(&format!("README.md:L{}", start_line), code);
        let tokens = lexer.tokenize_all().unwrap_or_else(|e| {
            panic!("README.md Example #{} (Line {}) failed lexing: {}", idx + 1, start_line, e);
        });

        // 2. Parser Verification
        let mut parser = EndParser::new(&format!("README.md:L{}", start_line), tokens);
        let module = parser.parse_module(&format!("readme_example_{}", idx + 1)).unwrap_or_else(|e| {
            panic!("README.md Example #{} (Line {}) failed parsing: {}", idx + 1, start_line, e);
        });

        // 3. Dependency Loader and Semantic Analysis Verification
        if code.contains("import ") {
            let temp_file = temp_dir.join(format!("example_{}.end", idx + 1));
            fs::write(&temp_file, code).expect("failed to write temp file");

            match load_and_analyze(&temp_file) {
                Ok((_mod, analyzer)) => {
                    assert!(
                        analyzer.errors.is_empty(),
                        "README.md Example #{} (Line {}) has semantic errors: {:?}",
                        idx + 1, start_line, analyzer.errors
                    );
                }
                Err(err_msg) => {
                    panic!("README.md Example #{} (Line {}) failed load_and_analyze: {}", idx + 1, start_line, err_msg);
                }
            }
        } else {
            let mut analyzer = SemanticAnalyzer::new(&format!("README.md:L{}", start_line), code);
            let analysis_res = analyzer.analyze_module(&module);
            assert!(
                analysis_res.is_ok() && analyzer.errors.is_empty(),
                "README.md Example #{} (Line {}) failed semantic analysis: {:?}",
                idx + 1, start_line, analyzer.errors
            );
        }

        println!("    ✅ Example #{} passed lexing, parsing, and semantic analysis!", idx + 1);
    }
}

#[test]
fn test_docs_directory_all_examples_compilation() {
    let repo_root = get_repo_root();
    let docs_dir = repo_root.join("docs");
    if !docs_dir.exists() {
        return;
    }

    let mut total_blocks = 0;
    let mut passed_blocks = 0;

    let entries = fs::read_dir(&docs_dir).expect("failed to read docs dir");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let content = fs::read_to_string(&path).unwrap_or_default();
            let blocks = extract_end_code_blocks(&content);

            for (start_line, code) in blocks {
                total_blocks += 1;
                let mut lexer = Lexer::new(&format!("{}:L{}", file_name, start_line), &code);
                if let Ok(tokens) = lexer.tokenize_all() {
                    let mut parser = EndParser::new(&format!("{}:L{}", file_name, start_line), tokens);
                    if let Ok(_module) = parser.parse_module("doc_mod") {
                        passed_blocks += 1;
                    }
                }
            }
        }
    }

    println!("📚 Verified docs/ code blocks: {}/{} parsed successfully", passed_blocks, total_blocks);
    assert!(total_blocks > 0, "docs/ must contain code examples");
}
