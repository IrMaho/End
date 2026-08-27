pub mod buffer;
pub mod device;
pub mod error;
pub mod gpu_tests;
pub mod kernels;
pub mod ops;

pub use buffer::GpuBuffer;
pub use device::GpuContext;
pub use error::GpuError;
pub use kernels::{compile_shader_module, MATRIX_MULTIPLY_WGSL, VECTOR_ADD_WGSL};
pub use ops::{execute_matrix_multiply, execute_vector_add, GpuExecutionReport};

/// Initialize the real GPU context
pub fn init_gpu() -> Result<GpuContext, GpuError> {
    GpuContext::init()
}

/// Convenience single-call vector addition on GPU
pub fn run_gpu_vector_add(a: &[f32], b: &[f32]) -> Result<GpuExecutionReport<Vec<f32>>, GpuError> {
    let ctx = GpuContext::init()?;
    execute_vector_add(&ctx, a, b)
}

/// Convenience single-call matrix multiplication on GPU
pub fn run_gpu_matmul(
    a: &[f32],
    b: &[f32],
    m: u32,
    k: u32,
    n: u32,
) -> Result<GpuExecutionReport<Vec<f32>>, GpuError> {
    let ctx = GpuContext::init()?;
    execute_matrix_multiply(&ctx, a, b, m, k, n)
}
