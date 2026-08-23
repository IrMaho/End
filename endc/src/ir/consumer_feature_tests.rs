#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use std::fs;
    use std::path::Path;

    fn parse_end_file(file_path: &str) -> Result<Module, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        let mut lexer = Lexer::new(file_path, &content);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new(file_path, tokens);
        parser.parse_module("test_consumer_module")
    }

    #[test]
    fn test_all_consumer_end_files_in_features_directory() {
        let feature_dir = Path::new("../tests/features");
        let fallback_dir = Path::new("tests/features");
        let dir_to_use = if feature_dir.exists() {
            feature_dir
        } else if fallback_dir.exists() {
            fallback_dir
        } else {
            panic!("Could not locate tests/features directory");
        };

        let mut verified_count = 0;
        for entry in fs::read_dir(dir_to_use).expect("Failed to read features directory") {
            let entry = entry.expect("Valid directory entry");
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "end") {
                let path_str = path.to_str().expect("Valid path string");
                let parse_result = parse_end_file(path_str);
                assert!(
                    parse_result.is_ok(),
                    "Failed to parse consumer test file {}: {:?}",
                    path_str,
                    parse_result.err()
                );
                verified_count += 1;
            }
        }

        assert!(
            verified_count >= 16,
            "Expected at least 16 consumer .end test files, found {}",
            verified_count
        );
    }
}
