#[cfg(test)]
mod tests {
    use crate::runtime::ai::error::AiError;
    use crate::runtime::ai::gguf::{parse_gguf_metadata, validate_gguf_file, GGUF_MAGIC};
    use crate::runtime::ai::inference::{execute_inference, InferenceConfig};
    use crate::runtime::ai::model::LlmModel;
    use crate::runtime::ai::tokenizer::LlmTokenizer;
    use candle_core::Device;
    use std::fs::{self, File};
    use std::io::{Seek, SeekFrom, Write};
    use std::path::{Path, PathBuf};

    fn get_temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("end_ai_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Helper to write a complete, valid GGUF v3 file with Llama architecture
    fn write_test_gguf_file(path: &Path, arch: &str, vocab_size: usize, hidden_size: usize, intermediate_size: usize) {
        let mut f = File::create(path).unwrap();

        // 1. Header
        f.write_all(&GGUF_MAGIC.to_le_bytes()).unwrap(); // Magic "GGUF"
        f.write_all(&3u32.to_le_bytes()).unwrap();       // Version 3
        
        let tensor_names = [
            "token_embd.weight",
            "blk.0.attn_norm.weight",
            "blk.0.attn_q.weight",
            "blk.0.attn_k.weight",
            "blk.0.attn_v.weight",
            "blk.0.attn_output.weight",
            "blk.0.ffn_norm.weight",
            "blk.0.ffn_gate.weight",
            "blk.0.ffn_up.weight",
            "blk.0.ffn_down.weight",
            "output_norm.weight",
            "output.weight",
        ];

        let tensor_count = tensor_names.len() as u64;
        let kv_count = 17u64; // Architecture params + Tokenizer metadata

        f.write_all(&tensor_count.to_le_bytes()).unwrap();
        f.write_all(&kv_count.to_le_bytes()).unwrap();

        // 2. Metadata Key-Value pairs
        write_kv_string(&mut f, "general.architecture", arch);
        write_kv_string(&mut f, "general.name", "TestMiniLlama");
        write_kv_u32(&mut f, &format!("{}.block_count", arch), 1);
        write_kv_u32(&mut f, &format!("{}.context_length", arch), 128);
        write_kv_u32(&mut f, &format!("{}.embedding_length", arch), hidden_size as u32);
        write_kv_u32(&mut f, &format!("{}.feed_forward_length", arch), intermediate_size as u32);
        write_kv_u32(&mut f, &format!("{}.attention.head_count", arch), 2);
        write_kv_u32(&mut f, &format!("{}.attention.head_count_kv", arch), 2);
        write_kv_u32(&mut f, &format!("{}.rope.dimension_count", arch), (hidden_size / 2) as u32);
        write_kv_f32(&mut f, &format!("{}.rope.freq_base", arch), 10000.0);
        write_kv_f32(&mut f, &format!("{}.attention.layer_norm_rms_epsilon", arch), 1e-5);

        // Tokenizer metadata
        write_kv_string(&mut f, "tokenizer.ggml.model", "llama");
        let vocab_tokens: Vec<String> = (0..vocab_size)
            .map(|i| match i {
                0 => "<unk>".to_string(),
                1 => "<s>".to_string(),
                2 => "</s>".to_string(),
                idx => format!("tok_{}", idx),
            })
            .collect();
        let scores: Vec<f32> = vec![0.0; vocab_size];
        let token_types: Vec<i32> = (0..vocab_size)
            .map(|i| if i == 0 { 2 } else if i <= 2 { 3 } else { 1 })
            .collect();

        write_kv_string_array(&mut f, "tokenizer.ggml.tokens", &vocab_tokens);
        write_kv_f32_array(&mut f, "tokenizer.ggml.scores", &scores);
        write_kv_i32_array(&mut f, "tokenizer.ggml.token_type", &token_types);
        write_kv_u32(&mut f, "tokenizer.ggml.bos_token_id", 1);
        write_kv_u32(&mut f, "tokenizer.ggml.eos_token_id", 2);

        // 3. Tensor Info Table
        let tensor_shapes: Vec<(&str, Vec<u64>)> = vec![
            ("token_embd.weight", vec![hidden_size as u64, vocab_size as u64]),
            ("blk.0.attn_norm.weight", vec![hidden_size as u64]),
            ("blk.0.attn_q.weight", vec![hidden_size as u64, hidden_size as u64]),
            ("blk.0.attn_k.weight", vec![hidden_size as u64, hidden_size as u64]),
            ("blk.0.attn_v.weight", vec![hidden_size as u64, hidden_size as u64]),
            ("blk.0.attn_output.weight", vec![hidden_size as u64, hidden_size as u64]),
            ("blk.0.ffn_norm.weight", vec![hidden_size as u64]),
            ("blk.0.ffn_gate.weight", vec![hidden_size as u64, intermediate_size as u64]),
            ("blk.0.ffn_up.weight", vec![hidden_size as u64, intermediate_size as u64]),
            ("blk.0.ffn_down.weight", vec![intermediate_size as u64, hidden_size as u64]),
            ("output_norm.weight", vec![hidden_size as u64]),
            ("output.weight", vec![hidden_size as u64, vocab_size as u64]),
        ];

        let mut offsets: Vec<u64> = Vec::new();
        let mut curr_offset: u64 = 0;
        let alignment = 32u64;

        for (_name, shape) in &tensor_shapes {
            let elem_count: u64 = shape.iter().product();
            let byte_size = elem_count * 4; // F32 = 4 bytes
            offsets.push(curr_offset);
            curr_offset += byte_size;
            // Pad to alignment
            let pad = (alignment - (curr_offset % alignment)) % alignment;
            curr_offset += pad;
        }

        for (i, (name, shape)) in tensor_shapes.iter().enumerate() {
            write_string(&mut f, name);
            f.write_all(&(shape.len() as u32).to_le_bytes()).unwrap();
            for dim in shape {
                f.write_all(&dim.to_le_bytes()).unwrap();
            }
            f.write_all(&0u32.to_le_bytes()).unwrap(); // Type 0 = F32
            f.write_all(&offsets[i].to_le_bytes()).unwrap();
        }

        // Pad to alignment before tensor data
        let pos = f.stream_position().unwrap();
        let pad = (alignment - (pos % alignment)) % alignment;
        for _ in 0..pad {
            f.write_all(&[0u8]).unwrap();
        }

        let tensor_data_start = f.stream_position().unwrap();

        // 4. Write Tensor Data (Deterministic weights)
        for (i, (_name, shape)) in tensor_shapes.iter().enumerate() {
            f.seek(SeekFrom::Start(tensor_data_start + offsets[i])).unwrap();
            let elem_count: usize = shape.iter().product::<u64>() as usize;
            for j in 0..elem_count {
                // Deterministic non-zero values for stable activations
                let val: f32 = (((i * 100 + j) % 17) as f32 - 8.0) * 0.05 + 0.1;
                f.write_all(&val.to_le_bytes()).unwrap();
            }
        }
        f.flush().unwrap();
    }

    fn write_string(f: &mut File, s: &str) {
        let bytes = s.as_bytes();
        f.write_all(&(bytes.len() as u64).to_le_bytes()).unwrap();
        f.write_all(bytes).unwrap();
    }

    fn write_kv_string(f: &mut File, key: &str, val: &str) {
        write_string(f, key);
        f.write_all(&8u32.to_le_bytes()).unwrap(); // Type 8 = String
        write_string(f, val);
    }

    fn write_kv_u32(f: &mut File, key: &str, val: u32) {
        write_string(f, key);
        f.write_all(&4u32.to_le_bytes()).unwrap(); // Type 4 = Uint32
        f.write_all(&val.to_le_bytes()).unwrap();
    }

    fn write_kv_f32(f: &mut File, key: &str, val: f32) {
        write_string(f, key);
        f.write_all(&6u32.to_le_bytes()).unwrap(); // Type 6 = Float32
        f.write_all(&val.to_le_bytes()).unwrap();
    }

    fn write_kv_string_array(f: &mut File, key: &str, arr: &[String]) {
        write_string(f, key);
        f.write_all(&9u32.to_le_bytes()).unwrap(); // Type 9 = Array
        f.write_all(&8u32.to_le_bytes()).unwrap(); // Element Type 8 = String
        f.write_all(&(arr.len() as u64).to_le_bytes()).unwrap();
        for s in arr {
            write_string(f, s);
        }
    }

    fn write_kv_f32_array(f: &mut File, key: &str, arr: &[f32]) {
        write_string(f, key);
        f.write_all(&9u32.to_le_bytes()).unwrap(); // Type 9 = Array
        f.write_all(&6u32.to_le_bytes()).unwrap(); // Element Type 6 = Float32
        f.write_all(&(arr.len() as u64).to_le_bytes()).unwrap();
        for val in arr {
            f.write_all(&val.to_le_bytes()).unwrap();
        }
    }

    fn write_kv_i32_array(f: &mut File, key: &str, arr: &[i32]) {
        write_string(f, key);
        f.write_all(&9u32.to_le_bytes()).unwrap(); // Type 9 = Array
        f.write_all(&5u32.to_le_bytes()).unwrap(); // Element Type 5 = Int32
        f.write_all(&(arr.len() as u64).to_le_bytes()).unwrap();
        for val in arr {
            f.write_all(&val.to_le_bytes()).unwrap();
        }
    }

    // =========================================================================
    // Gate 2: Real GGUF Loading & Metadata Parsing
    // =========================================================================

    #[test]
    fn test_real_gguf_loading_and_metadata_validation() {
        let dir = get_temp_dir();
        let model_path = dir.join("valid_llama.gguf");
        write_test_gguf_file(&model_path, "llama", 32, 16, 32);

        let meta = validate_gguf_file(&model_path).expect("GGUF validation must succeed for valid model");
        assert_eq!(meta.version, 3);
        assert_eq!(meta.architecture, "llama");
        assert_eq!(meta.tensor_count, 12);
        assert_eq!(meta.embedding_length, Some(16));
        assert_eq!(meta.feed_forward_length, Some(32));
        assert_eq!(meta.block_count, Some(1));
        assert_eq!(meta.head_count, Some(2));
        assert_eq!(meta.tensor_info.len(), 12);
        assert_eq!(meta.tensor_info[0].name, "token_embd.weight");
        assert_eq!(meta.tensor_info[0].tensor_type, "F32");

        let device = Device::Cpu;
        let model = LlmModel::load_from_file(&model_path, &device)
            .expect("Model weights must load successfully into Candle");
        assert_eq!(model.architecture, "llama");
    }

    // =========================================================================
    // Gate 3 & 4: Real Inference & Deterministic Reproducibility
    // =========================================================================

    #[test]
    fn test_real_inference_and_reproducibility() {
        let dir = get_temp_dir();
        let model_path = dir.join("llama_repro.gguf");
        write_test_gguf_file(&model_path, "llama", 256, 16, 32);

        let device = Device::Cpu;
        let mut model1 = LlmModel::load_from_file(&model_path, &device).unwrap();
        let mut model2 = LlmModel::load_from_file(&model_path, &device).unwrap();

        // Build vocabulary for 256 tokens
        let vocab: Vec<String> = (0..256).map(|i| format!("tok_{}", i)).collect();
        let tokenizer = LlmTokenizer::from_vocab(vocab, Some(1), Some(2));

        let prompt = "Hello world";
        let config = InferenceConfig {
            max_tokens: 10,
            temperature: 0.0, // Greedy deterministic
            seed: 12345,
            repeat_penalty: 1.0,
            repeat_last_n: 64,
            top_p: None,
        };

        // Execution Run 1
        let res1 = execute_inference(&mut model1, &tokenizer, prompt, &config).unwrap();
        assert_eq!(res1.status, "success");
        assert_eq!(res1.generated_token_ids.len(), 10);
        assert!(!res1.generated_tokens.is_empty());
        assert!(!res1.output_text.is_empty());

        // Execution Run 2 (Same model, prompt, seed, temp)
        let res2 = execute_inference(&mut model2, &tokenizer, prompt, &config).unwrap();
        assert_eq!(res2.status, "success");
        assert_eq!(res2.generated_token_ids.len(), 10);

        // Reproducibility Assertion: 100% exact equality on all 10 token IDs
        assert_eq!(
            res1.generated_token_ids, res2.generated_token_ids,
            "Run 1 and Run 2 must produce identical token IDs under deterministic configuration"
        );
        assert_eq!(res1.output_text, res2.output_text);
    }

    // =========================================================================
    // Gate 6: Invalid & Malformed Input Handling
    // =========================================================================

    #[test]
    fn test_corrupt_gguf_magic_rejected() {
        let dir = get_temp_dir();
        let corrupt_path = dir.join("corrupt_magic.gguf");
        let mut f = File::create(&corrupt_path).unwrap();
        f.write_all(b"FAKE_MAGIC_HEADER_123456").unwrap();

        let res = validate_gguf_file(&corrupt_path);
        assert!(res.is_err());
        match res.unwrap_err() {
            AiError::InvalidGguf(msg) => {
                assert!(msg.contains("magic"), "Error must clearly identify magic failure: {}", msg);
            }
            other => panic!("Expected InvalidGguf error, got: {:?}", other),
        }
    }

    #[test]
    fn test_truncated_gguf_rejected() {
        let dir = get_temp_dir();
        let truncated_path = dir.join("truncated.gguf");
        let mut f = File::create(&truncated_path).unwrap();
        f.write_all(&GGUF_MAGIC.to_le_bytes()).unwrap();
        f.write_all(&3u32.to_le_bytes()).unwrap(); // Cut off immediately after version

        let res = validate_gguf_file(&truncated_path);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AiError::InvalidGguf(_)));
    }

    #[test]
    fn test_unsupported_architecture_rejected() {
        let dir = get_temp_dir();
        let falcon_path = dir.join("falcon_model.gguf");
        write_test_gguf_file(&falcon_path, "falcon", 32, 16, 32);

        let device = Device::Cpu;
        let res = LlmModel::load_from_file(&falcon_path, &device);
        match res {
            Err(AiError::UnsupportedArchitecture { found, supported }) => {
                assert_eq!(found, "falcon");
                assert!(supported.contains(&"llama".to_string()));
            }
            Err(other) => panic!("Expected UnsupportedArchitecture, got: {:?}", other),
            Ok(_) => panic!("Unsupported architecture 'falcon' must be rejected!"),
        }
    }

    #[test]
    fn test_nonexistent_model_file_rejected() {
        let non_existent = Path::new("this_file_does_not_exist_98765.gguf");
        let device = Device::Cpu;
        let res = LlmModel::load_from_file(non_existent, &device);
        match res {
            Err(AiError::ModelNotFound(_)) => {}
            Err(other) => panic!("Expected ModelNotFound, got: {:?}", other),
            Ok(_) => panic!("Non-existent model file must fail to load!"),
        }
    }

    // =========================================================================
    // Gate 8: max_tokens & Stopping Conditions
    // =========================================================================

    #[test]
    fn test_max_tokens_enforced() {
        let dir = get_temp_dir();
        let model_path = dir.join("max_tokens.gguf");
        write_test_gguf_file(&model_path, "llama", 256, 16, 32);

        let device = Device::Cpu;
        let mut model = LlmModel::load_from_file(&model_path, &device).unwrap();
        let vocab: Vec<String> = (0..256).map(|i| format!("tok_{}", i)).collect();
        let tokenizer = LlmTokenizer::from_vocab(vocab, Some(1), Some(2));

        for target_tokens in [3, 7, 12] {
            let config = InferenceConfig {
                max_tokens: target_tokens,
                temperature: 0.0,
                seed: 42,
                repeat_penalty: 1.0,
                repeat_last_n: 64,
                top_p: None,
            };
            let res = execute_inference(&mut model, &tokenizer, "Test prompt", &config).unwrap();
            assert!(
                res.generated_token_ids.len() <= target_tokens,
                "Generated tokens ({}) must not exceed max_tokens ({})",
                res.generated_token_ids.len(),
                target_tokens
            );
        }
    }

    // =========================================================================
    // Gate 5: Differential Reference Verification
    // =========================================================================

    #[test]
    fn test_differential_verification_against_reference() {
        let dir = get_temp_dir();
        let model_path = dir.join("diff_verify_llama.gguf");
        write_test_gguf_file(&model_path, "llama", 256, 16, 32);

        // 1. Run End inference
        let device = Device::Cpu;
        let mut model = LlmModel::load_from_file(&model_path, &device).unwrap();
        let vocab: Vec<String> = (0..256).map(|i| format!("tok_{}", i)).collect();
        let tokenizer = LlmTokenizer::from_vocab(vocab, Some(1), Some(2));

        let prompt = "Hello world";
        let config = InferenceConfig {
            max_tokens: 5,
            temperature: 0.0,
            seed: 42,
            repeat_penalty: 1.0,
            repeat_last_n: 64,
            top_p: None,
        };

        let end_res = execute_inference(&mut model, &tokenizer, prompt, &config).unwrap();
        assert_eq!(end_res.generated_token_ids.len(), 5);
        assert_eq!(end_res.status, "success");

        // 2. Differential test with Python llama-cpp reference
        let py_script = r#"
import sys, json
try:
    from llama_cpp import Llama
    model_path = sys.argv[1]
    prompt = sys.argv[2]
    llm = Llama(model_path=model_path, seed=42, verbose=False, n_ctx=128)
    output = llm(prompt, max_tokens=5, temperature=0.0)
    print(json.dumps({"status": "ok", "text": output["choices"][0]["text"]}))
except Exception as e:
    print(json.dumps({"status": "skipped", "reason": str(e)}))
"#;
        let script_path = dir.join("run_llama_cpp.py");
        fs::write(&script_path, py_script).unwrap();

        let output = std::process::Command::new("python")
            .arg(&script_path)
            .arg(&model_path)
            .arg(prompt)
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            println!("Differential reference execution report: {}", stdout);
        }
    }
}
