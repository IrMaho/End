#[cfg(test)]
mod tests {
    use crate::runtime::gpu::buffer::GpuBuffer;
    use crate::runtime::gpu::device::GpuContext;
    use crate::runtime::gpu::error::GpuError;
    use crate::runtime::gpu::kernels::compile_shader_module;
    use crate::runtime::gpu::ops::{execute_matrix_multiply, execute_vector_add};
    use wgpu::BufferUsages;

    fn get_gpu_context_or_skip() -> Option<GpuContext> {
        match GpuContext::init() {
            Ok(ctx) => {
                println!(
                    "GPU Adapter Initialized: '{}' (backend={}, type={}, driver='{}')",
                    ctx.adapter_name(),
                    ctx.backend_name(),
                    ctx.device_type_str(),
                    ctx.driver_info()
                );
                Some(ctx)
            }
            Err(e) => {
                eprintln!("SKIPPING GPU test on this environment: {}", e);
                None
            }
        }
    }

    // =========================================================================
    // Gate 2: Real Adapter / Device Initialization & Inspection
    // =========================================================================

    #[test]
    fn test_real_gpu_adapter_initialization() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        assert!(!ctx.adapter_name().is_empty(), "Adapter name must not be empty");
        assert!(!ctx.backend_name().is_empty(), "Backend name must not be empty");
        let limits = ctx.adapter.limits();
        assert!(limits.max_compute_workgroup_size_x >= 64, "Workgroup size X must be at least 64");
    }

    // =========================================================================
    // Gate 5: Vector Add 1,000 Elements (1K)
    // =========================================================================

    #[test]
    fn test_vector_add_1k_elements() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        let n = 1_000;
        let mut a = Vec::with_capacity(n);
        let mut b = Vec::with_capacity(n);
        let mut expected = Vec::with_capacity(n);

        for i in 0..n {
            let val_a = (i as f32) * 0.5;
            let val_b = ((n - i) as f32) * 1.5;
            a.push(val_a);
            b.push(val_b);
            expected.push(val_a + val_b);
        }

        let report = execute_vector_add(&ctx, &a, &b).expect("GPU vector add (1K) must succeed");
        assert_eq!(report.input_elements, n);
        assert_eq!(report.result.len(), n);
        assert!(!report.output_sha256.is_empty());

        // Assert 100% exact equality against CPU reference
        for i in 0..n {
            assert_eq!(
                report.result[i], expected[i],
                "Mismatch at index {}: GPU={}, CPU={}",
                i, report.result[i], expected[i]
            );
        }
    }

    // =========================================================================
    // Gate 5: Vector Add 1,000,000 Elements (1M)
    // =========================================================================

    #[test]
    fn test_vector_add_1m_elements() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        let n = 1_000_000;
        let mut a = Vec::with_capacity(n);
        let mut b = Vec::with_capacity(n);
        let mut expected = Vec::with_capacity(n);

        for i in 0..n {
            let val_a = ((i % 100) as f32) * 0.1;
            let val_b = (((i + 7) % 100) as f32) * 0.2;
            a.push(val_a);
            b.push(val_b);
            expected.push(val_a + val_b);
        }

        let report = execute_vector_add(&ctx, &a, &b).expect("GPU vector add (1M) must succeed");
        assert_eq!(report.input_elements, n);
        assert_eq!(report.result.len(), n);

        for i in 0..n {
            assert_eq!(
                report.result[i], expected[i],
                "Mismatch at index {}: GPU={}, CPU={}",
                i, report.result[i], expected[i]
            );
        }
    }

    // =========================================================================
    // Gate 5: Vector Add 64,000,000 Elements (64M / 256 MB)
    // =========================================================================

    #[test]
    fn test_vector_add_64m_elements() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        let n = 64_000_000;
        let required_bytes = (n * 4) as u64; // 256 MB
        let max_storage_size = ctx.adapter.limits().max_storage_buffer_binding_size as u64;

        if max_storage_size < required_bytes {
            println!(
                "Adapter max storage buffer size ({} MB) is smaller than 256 MB, scaling to max supported chunk...",
                max_storage_size / (1024 * 1024)
            );
            let supported_n = (max_storage_size / 4) as usize;
            let a = vec![1.25f32; supported_n];
            let b = vec![2.75f32; supported_n];
            let report = execute_vector_add(&ctx, &a, &b).expect("GPU vector add must succeed on max supported size");
            assert_eq!(report.result.len(), supported_n);
            assert_eq!(report.result[0], 4.0f32);
            assert_eq!(report.result[supported_n - 1], 4.0f32);
            return;
        }

        let a = vec![1.25f32; n];
        let b = vec![2.75f32; n];

        let report = execute_vector_add(&ctx, &a, &b).expect("GPU vector add (64M) must succeed");
        assert_eq!(report.input_elements, n);
        assert_eq!(report.result.len(), n);
        assert_eq!(report.result[0], 4.0f32);
        assert_eq!(report.result[n / 2], 4.0f32);
        assert_eq!(report.result[n - 1], 4.0f32);
    }

    // =========================================================================
    // Gate 6: Matrix Multiplication 256x256
    // =========================================================================

    #[test]
    fn test_matrix_multiply_256x256() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        let m = 256u32;
        let k = 256u32;
        let n = 256u32;

        let mut a = Vec::with_capacity((m * k) as usize);
        let mut b = Vec::with_capacity((k * n) as usize);

        for row in 0..m {
            for col in 0..k {
                let val = (((row + col) % 13) as f32) * 0.1 - 0.5;
                a.push(val);
            }
        }

        for row in 0..k {
            for col in 0..n {
                let val = (((row * 3 + col) % 17) as f32) * 0.1 - 0.5;
                b.push(val);
            }
        }

        // 1. Calculate CPU Reference
        let mut expected = vec![0.0f32; (m * n) as usize];
        for r in 0..m as usize {
            for c in 0..n as usize {
                let mut sum = 0.0f32;
                for i in 0..k as usize {
                    sum += a[r * (k as usize) + i] * b[i * (n as usize) + c];
                }
                expected[r * (n as usize) + c] = sum;
            }
        }

        // 2. Execute on GPU
        let report = execute_matrix_multiply(&ctx, &a, &b, m, k, n)
            .expect("GPU matrix multiplication (256x256) must succeed");

        assert_eq!(report.result.len(), (m * n) as usize);

        // 3. Floating point tolerance comparison: float32 sum of 256 products tolerance <= 1e-4
        let max_tolerance = 1e-4f32;
        for i in 0..(m * n) as usize {
            let diff = (report.result[i] - expected[i]).abs();
            assert!(
                diff <= max_tolerance,
                "Matrix mismatch at [{}]: GPU={}, CPU={}, diff={} > tolerance {}",
                i, report.result[i], expected[i], diff, max_tolerance
            );
        }
    }

    // =========================================================================
    // Gate 7: Negative / Error-Handling Tests
    // =========================================================================

    #[test]
    fn test_invalid_shader_compilation_failure() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        let invalid_wgsl = "@compute fn main() { this is completely broken syntax !!! }";
        let res = compile_shader_module(&ctx.device, invalid_wgsl, Some("Broken Shader"));
        assert!(res.is_err(), "Invalid WGSL shader must fail compilation");
        match res.unwrap_err() {
            GpuError::ShaderCompilationFailed(msg) => {
                assert!(!msg.is_empty(), "Error message must contain compilation details");
            }
            other => panic!("Expected ShaderCompilationFailed, got: {:?}", other),
        }
    }

    #[test]
    fn test_zero_sized_buffer_rejected() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        let res = GpuBuffer::allocate(&ctx.device, 0, BufferUsages::STORAGE, Some("Zero Buffer"));
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), GpuError::InvalidBufferSize { size: 0, .. }));
    }

    #[test]
    fn test_vector_size_mismatch_rejected() {
        let ctx = match get_gpu_context_or_skip() {
            Some(c) => c,
            None => return,
        };

        let a = vec![1.0f32; 100];
        let b = vec![1.0f32; 50]; // Mismatched length

        let res = execute_vector_add(&ctx, &a, &b);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), GpuError::ExecutionFailed(_)));
    }
}
