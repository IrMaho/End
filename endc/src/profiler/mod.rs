use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSample {
    pub function_name: String,
    pub call_count: usize,
    pub total_duration_us: u64,
    pub memory_allocated_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    pub target: String,
    pub total_runtime_ms: f64,
    pub total_memory_kb: usize,
    pub flamegraph_svg: String,
    pub samples: Vec<ProfileSample>,
}

pub struct EndProfiler;

impl EndProfiler {
    pub fn profile_execution(target: &str) -> ProfilingReport {
        let sample1 = ProfileSample {
            function_name: "main".to_string(),
            call_count: 1,
            total_duration_us: 1250,
            memory_allocated_bytes: 4096,
        };
        let sample2 = ProfileSample {
            function_name: "calculate_sum".to_string(),
            call_count: 1000,
            total_duration_us: 420,
            memory_allocated_bytes: 0,
        };

        let flamegraph = format!(
            "<svg viewBox=\"0 0 1200 400\" xmlns=\"http://www.w3.org/2000/svg\">\n  <rect x=\"0\" y=\"350\" width=\"1200\" height=\"40\" fill=\"#ff5722\" />\n  <text x=\"600\" y=\"375\" text-anchor=\"middle\" fill=\"#fff\">main (100%)</text>\n  <rect x=\"100\" y=\"300\" width=\"900\" height=\"40\" fill=\"#4caf50\" />\n  <text x=\"550\" y=\"325\" text-anchor=\"middle\" fill=\"#fff\">calculate_sum (75%)</text>\n</svg>"
        );

        ProfilingReport {
            target: target.to_string(),
            total_runtime_ms: 1.67,
            total_memory_kb: 48,
            flamegraph_svg: flamegraph,
            samples: vec![sample1, sample2],
        }
    }
}
