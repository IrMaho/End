use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum GpuError {
    NoAdapterAvailable(String),
    DeviceRequestFailed(String),
    BufferAllocationFailed(String),
    InvalidBufferSize { size: usize, reason: String },
    ShaderCompilationFailed(String),
    PipelineCreationFailed(String),
    InvalidDispatchDimensions { x: u32, y: u32, z: u32, reason: String },
    ExecutionFailed(String),
    ReadbackFailed(String),
    OutOfMemory(String),
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::NoAdapterAvailable(msg) => write!(f, "No suitable GPU adapter available: {}", msg),
            GpuError::DeviceRequestFailed(msg) => write!(f, "Failed to request GPU device: {}", msg),
            GpuError::BufferAllocationFailed(msg) => write!(f, "Failed to allocate GPU buffer: {}", msg),
            GpuError::InvalidBufferSize { size, reason } => {
                write!(f, "Invalid buffer size ({} bytes): {}", size, reason)
            }
            GpuError::ShaderCompilationFailed(msg) => write!(f, "WGSL shader compilation failed: {}", msg),
            GpuError::PipelineCreationFailed(msg) => write!(f, "Compute pipeline creation failed: {}", msg),
            GpuError::InvalidDispatchDimensions { x, y, z, reason } => {
                write!(f, "Invalid compute dispatch dimensions [{}, {}, {}]: {}", x, y, z, reason)
            }
            GpuError::ExecutionFailed(msg) => write!(f, "GPU compute execution failed: {}", msg),
            GpuError::ReadbackFailed(msg) => write!(f, "GPU buffer readback failed: {}", msg),
            GpuError::OutOfMemory(msg) => write!(f, "GPU memory exhausted: {}", msg),
        }
    }
}

impl std::error::Error for GpuError {}
