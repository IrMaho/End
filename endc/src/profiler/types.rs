use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    pub function_name: String,
    pub call_count: usize,
    pub total_duration_us: u64,
    pub self_duration_us: u64,
    pub memory_allocated_bytes: usize,
    pub percent: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphEdge {
    pub caller: String,
    pub callee: String,
    pub call_count: usize,
    pub total_duration_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    pub target: String,
    pub total_runtime_ms: f64,
    pub total_memory_kb: usize,
    pub total_samples: usize,
    pub flamegraph_svg: String,
    pub samples: Vec<ProfileSample>,
    pub call_graph: Vec<CallGraphEdge>,
    pub folded_stacks: String,
}
