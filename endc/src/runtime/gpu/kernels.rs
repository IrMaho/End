use super::error::GpuError;
use wgpu::*;

pub const VECTOR_ADD_WGSL: &str = r#"
@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;
@group(0) @binding(3) var<uniform> num_elements: u32;

@compute @workgroup_size(64, 1, 1)
fn main(
    @builtin(workgroup_id) wg_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>
) {
    let wg_index = wg_id.y * 65535u + wg_id.x;
    let idx = wg_index * 64u + local_id.x;
    if (idx < num_elements) {
        output[idx] = a[idx] + b[idx];
    }
}
"#;

pub const MATRIX_MULTIPLY_WGSL: &str = r#"
struct MatrixDims {
    m: u32,
    k: u32,
    n: u32,
    _pad: u32,
};

@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> c: array<f32>;
@group(0) @binding(3) var<uniform> dims: MatrixDims;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let row = global_id.y;
    let col = global_id.x;
    if (row < dims.m && col < dims.n) {
        var sum: f32 = 0.0;
        for (var i: u32 = 0u; i < dims.k; i = i + 1u) {
            sum = sum + a[row * dims.k + i] * b[i * dims.n + col];
        }
        c[row * dims.n + col] = sum;
    }
}
"#;

/// Compile a real WGSL compute shader module with error capture
pub fn compile_shader_module(
    device: &Device,
    wgsl_source: &str,
    label: Option<&str>,
) -> Result<ShaderModule, GpuError> {
    // 1. Validate WGSL syntax using naga directly to produce actionable error messages
    if let Err(e) = naga::front::wgsl::parse_str(wgsl_source) {
        return Err(GpuError::ShaderCompilationFailed(format!("{}", e.emit_to_string(wgsl_source))));
    }

    // 2. Create wgpu shader module
    let shader_module = device.create_shader_module(ShaderModuleDescriptor {
        label,
        source: ShaderSource::Wgsl(wgsl_source.into()),
    });

    Ok(shader_module)
}
