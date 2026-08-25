use crate::codegen::backend_trait::BackendError;

pub struct WasmValidator;

impl WasmValidator {
    pub fn validate_wat(wat_source: &str) -> Result<(), BackendError> {
        let trimmed = wat_source.trim();
        if !trimmed.starts_with("(module") {
            return Err(BackendError::CodegenFailed(
                "WAT validation failed: source must begin with '(module'".to_string(),
            ));
        }
        if !trimmed.ends_with(')') {
            return Err(BackendError::CodegenFailed(
                "WAT validation failed: source must end with matching closing parenthesis ')'".to_string(),
            ));
        }

        // Check parentheses balance
        let mut depth = 0i32;
        for (line_idx, line) in trimmed.lines().enumerate() {
            for ch in line.chars() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth < 0 {
                        return Err(BackendError::CodegenFailed(format!(
                            "WAT validation failed: unmatched closing parenthesis at line {}",
                            line_idx + 1
                        )));
                    }
                }
            }
        }

        if depth != 0 {
            return Err(BackendError::CodegenFailed(format!(
                "WAT validation failed: {} unclosed parenthesis block(s)",
                depth
            )));
        }

        Ok(())
    }

    pub fn validate_wasm(bytes: &[u8]) -> Result<(), BackendError> {
        if bytes.len() < 8 {
            return Err(BackendError::CodegenFailed(format!(
                "WASM validation failed: binary too short ({} bytes)",
                bytes.len()
            )));
        }

        // 1. Magic Header
        if &bytes[0..4] != b"\0asm" {
            return Err(BackendError::CodegenFailed(format!(
                "WASM validation failed: invalid magic header {:?}",
                &bytes[0..4]
            )));
        }

        // 2. Version
        if &bytes[4..8] != &[0x01, 0x00, 0x00, 0x00] {
            return Err(BackendError::CodegenFailed(format!(
                "WASM validation failed: unsupported version {:?}",
                &bytes[4..8]
            )));
        }

        // 3. Section traversal
        let mut offset = 8;
        let mut prev_section_id = 0u8;

        while offset < bytes.len() {
            let section_id = bytes[offset];
            offset += 1;

            if section_id != 0 && section_id <= prev_section_id {
                return Err(BackendError::CodegenFailed(format!(
                    "WASM validation failed: section id {} out of order after section {}",
                    section_id, prev_section_id
                )));
            }
            if section_id != 0 {
                prev_section_id = section_id;
            }

            let (section_len, leb_len) = Self::read_u32_leb128(&bytes[offset..])
                .map_err(|e| BackendError::CodegenFailed(format!("WASM validation failed: {}", e)))?;
            offset += leb_len;

            if offset + (section_len as usize) > bytes.len() {
                return Err(BackendError::CodegenFailed(format!(
                    "WASM validation failed: section {} payload extends beyond file boundary",
                    section_id
                )));
            }

            offset += section_len as usize;
        }

        Ok(())
    }

    fn read_u32_leb128(bytes: &[u8]) -> Result<(u32, usize), &'static str> {
        let mut result = 0u32;
        let mut shift = 0;
        let mut count = 0;

        for &byte in bytes {
            count += 1;
            result |= ((byte & 0x7F) as u32) << shift;
            if (byte & 0x80) == 0 {
                return Ok((result, count));
            }
            shift += 7;
            if shift >= 35 {
                return Err("LEB128 integer overflow");
            }
        }

        Err("Unexpected EOF in LEB128")
    }
}
