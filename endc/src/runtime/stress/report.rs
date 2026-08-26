use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatencyPercentiles {
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p99_ms: f64,
    pub p99_9_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressReport {
    pub target_url: String,
    pub duration_s: f64,
    pub concurrency: usize,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub error_rate_percent: f64,
    pub throughput_rps: f64,
    pub latency: LatencyPercentiles,
    pub status_codes: HashMap<u16, usize>,
    pub connection_errors: usize,
    pub timeout_errors: usize,
}

impl StressReport {
    pub fn is_empty(&self) -> bool {
        self.total_requests == 0
    }
}
