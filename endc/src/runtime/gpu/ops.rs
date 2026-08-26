use super::buffer::GpuBuffer;
use super::device::GpuContext;
use super::error::GpuError;
use super::kernels::{compile_shader_module, MATRIX_MULTIPLY_WGSL, VECTOR_ADD_WGSL};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;
use wgpu::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuExecutionReport<T> {
    pub backend: String,
    pub adapter: String,
    pub device_type: String,
    pub driver: String,
    pub operation: String,
    pub workgroup_size: [u32; 3],
    pub dispatch_size: [u32; 3],
    pub input_elements: usize,
    pub duration_ms: u128,
    pub output_sha256: String,
    pub result: T,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MatrixMeta {
    m: u32,
    k: u32,
    n: u32,
    _pad: u32,
}

/// Execute vector addition on the real GPU: output[i] = a[i] + b[i]
pub fn execute_vector_add(
    ctx: &GpuContext,
    a: &[f32],
    b: &[f32],
) -> Result<GpuExecutionReport<Vec<f32>>, GpuError> {
    if a.len() != b.len() {
        return Err(GpuError::ExecutionFailed(format!(
            "Vector size mismatch: a has {} elements, b has {} elements",
            a.len(),
            b.len()
        )));
    }
    if a.is_empty() {
        return Err(GpuError::InvalidBufferSize {
            size: 0,
            reason: "Input vectors cannot be empty".to_string(),
        });
    }

    let n = a.len();
    let num_elements_u32 = n as u32;
    let byte_size = (n * std::mem::size_of::<f32>()) as u64;

    let device = &ctx.device;
    let queue = &ctx.queue;

    // 1. Allocate GPU Buffers
    let buf_a = GpuBuffer::allocate(device, byte_size, BufferUsages::STORAGE | BufferUsages::COPY_DST, Some("GPU Vec A"))?;
    let buf_b = GpuBuffer::allocate(device, byte_size, BufferUsages::STORAGE | BufferUsages::COPY_DST, Some("GPU Vec B"))?;
    let buf_out = GpuBuffer::allocate(device, byte_size, BufferUsages::STORAGE | BufferUsages::COPY_SRC, Some("GPU Vec Out"))?;
    let buf_uniform = GpuBuffer::allocate(device, 4, BufferUsages::UNIFORM | BufferUsages::COPY_DST, Some("GPU Vec Uniform"))?;

    // 2. Upload Data to GPU
    GpuBuffer::upload(queue, &buf_a.buffer, 0, a)?;
    GpuBuffer::upload(queue, &buf_b.buffer, 0, b)?;
    GpuBuffer::upload(queue, &buf_uniform.buffer, 0, &[num_elements_u32])?;

    // 3. Compile WGSL Shader Module
    let shader = compile_shader_module(device, VECTOR_ADD_WGSL, Some("Vector Add Shader"))?;

    // 4. Create Compute Pipeline
    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Vector Add Compute Pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Vector Add Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buf_a.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buf_b.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: buf_out.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: buf_uniform.buffer.as_entire_binding(),
            },
        ],
    });

    // 5. Dispatch Compute Pass with 2D grid support for large N
    let total_workgroups = (num_elements_u32 + 63) / 64;
    let dispatch_x = std::cmp::min(total_workgroups, 65535);
    let dispatch_y = (total_workgroups + 65534) / 65535;

    let start_time = Instant::now();

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Vector Add Command Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Vector Add Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
    }

    queue.submit(Some(encoder.finish()));

    // 6. Read back results from GPU
    let result_vec: Vec<f32> = GpuBuffer::readback(device, queue, &buf_out.buffer, byte_size)?;
    let duration_ms = start_time.elapsed().as_millis();

    // 7. Calculate SHA-256 of result bytes
    let result_bytes: &[u8] = bytemuck::cast_slice(&result_vec);
    let mut hasher = Sha256::new();
    hasher.update(result_bytes);
    let output_sha256 = format!("{:x}", hasher.finalize());

    Ok(GpuExecutionReport {
        backend: ctx.backend_name(),
        adapter: ctx.adapter_name().to_string(),
        device_type: ctx.device_type_str(),
        driver: ctx.driver_info().to_string(),
        operation: "vector_add".to_string(),
        workgroup_size: [64, 1, 1],
        dispatch_size: [dispatch_x, dispatch_y, 1],
        input_elements: n,
        duration_ms,
        output_sha256,
        result: result_vec,
    })
}

/// Execute matrix multiplication on the real GPU: C[M, N] = A[M, K] * B[K, N]
pub fn execute_matrix_multiply(
    ctx: &GpuContext,
    a: &[f32],
    b: &[f32],
    m: u32,
    k: u32,
    n: u32,
) -> Result<GpuExecutionReport<Vec<f32>>, GpuError> {
    let expected_a_len = (m * k) as usize;
    let expected_b_len = (k * n) as usize;
    let expected_c_len = (m * n) as usize;

    if a.len() != expected_a_len {
        return Err(GpuError::ExecutionFailed(format!(
            "Matrix A dimension mismatch: expected {}x{} = {} elements, found {}",
            m, k, expected_a_len, a.len()
        )));
    }
    if b.len() != expected_b_len {
        return Err(GpuError::ExecutionFailed(format!(
            "Matrix B dimension mismatch: expected {}x{} = {} elements, found {}",
            k, n, expected_b_len, b.len()
        )));
    }

    let byte_size_a = (expected_a_len * std::mem::size_of::<f32>()) as u64;
    let byte_size_b = (expected_b_len * std::mem::size_of::<f32>()) as u64;
    let byte_size_c = (expected_c_len * std::mem::size_of::<f32>()) as u64;

    let device = &ctx.device;
    let queue = &ctx.queue;

    // 1. Allocate GPU Buffers
    let buf_a = GpuBuffer::allocate(device, byte_size_a, BufferUsages::STORAGE | BufferUsages::COPY_DST, Some("GPU Mat A"))?;
    let buf_b = GpuBuffer::allocate(device, byte_size_b, BufferUsages::STORAGE | BufferUsages::COPY_DST, Some("GPU Mat B"))?;
    let buf_c = GpuBuffer::allocate(device, byte_size_c, BufferUsages::STORAGE | BufferUsages::COPY_SRC, Some("GPU Mat C"))?;
    let meta = MatrixMeta { m, k, n, _pad: 0 };
    let buf_uniform = GpuBuffer::allocate(device, std::mem::size_of::<MatrixMeta>() as u64, BufferUsages::UNIFORM | BufferUsages::COPY_DST, Some("GPU Mat Uniform"))?;

    // 2. Upload Data to GPU
    GpuBuffer::upload(queue, &buf_a.buffer, 0, a)?;
    GpuBuffer::upload(queue, &buf_b.buffer, 0, b)?;
    GpuBuffer::upload(queue, &buf_uniform.buffer, 0, &[meta])?;

    // 3. Compile WGSL Shader Module
    let shader = compile_shader_module(device, MATRIX_MULTIPLY_WGSL, Some("Matrix Multiply Shader"))?;

    // 4. Create Compute Pipeline
    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("Matrix Multiply Pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Matrix Multiply Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buf_a.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buf_b.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: buf_c.buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: buf_uniform.buffer.as_entire_binding(),
            },
        ],
    });

    // 5. Dispatch Compute Pass
    let dispatch_x = (n + 15) / 16;
    let dispatch_y = (m + 15) / 16;

    let start_time = Instant::now();

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Matrix Multiply Command Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Matrix Multiply Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
    }

    queue.submit(Some(encoder.finish()));

    // 6. Read back results from GPU
    let result_vec: Vec<f32> = GpuBuffer::readback(device, queue, &buf_c.buffer, byte_size_c)?;
    let duration_ms = start_time.elapsed().as_millis();

    // 7. Calculate SHA-256
    let result_bytes: &[u8] = bytemuck::cast_slice(&result_vec);
    let mut hasher = Sha256::new();
    hasher.update(result_bytes);
    let output_sha256 = format!("{:x}", hasher.finalize());

    Ok(GpuExecutionReport {
        backend: ctx.backend_name(),
        adapter: ctx.adapter_name().to_string(),
        device_type: ctx.device_type_str(),
        driver: ctx.driver_info().to_string(),
        operation: format!("matrix_multiply_{}x{}x{}", m, k, n),
        workgroup_size: [16, 16, 1],
        dispatch_size: [dispatch_x, dispatch_y, 1],
        input_elements: expected_a_len + expected_b_len,
        duration_ms,
        output_sha256,
        result: result_vec,
    })
}
