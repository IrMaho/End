use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationIntent {
    Fastest,
    LowestMemory,
    LowestEnergy,
    Balanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudget {
    pub max_latency_us: u64,
    pub max_memory_bytes: usize,
    pub max_cpu_percent: f64,
    pub max_energy_mj: Option<f64>,
    pub max_thermal_watts: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoOptimizationResult {
    pub chosen_intent: OptimizationIntent,
    pub representation: String, // "Stack", "Arena", "BitPacked", "DeviceMemory"
    pub vectorized_simd_width: usize,
    pub estimated_latency_us: u64,
    pub budget_satisfied: bool,
}

pub struct IntentOptimizationEngine;

impl IntentOptimizationEngine {
    pub fn optimize_workload(
        intent: OptimizationIntent,
        budget: &PerformanceBudget,
        data_elements: usize,
    ) -> ParetoOptimizationResult {
        let (rep, simd_width, est_latency) = match intent {
            OptimizationIntent::Fastest => {
                let simd = if data_elements >= 8 { 8 } else { 1 };
                ("StackSIMDVectorized", simd, (data_elements as u64) / 4)
            }
            OptimizationIntent::LowestMemory => {
                ("CompactBitPackedArena", 1, (data_elements as u64) * 2)
            }
            OptimizationIntent::LowestEnergy => {
                ("LowPowerDirectCompute", 4, (data_elements as u64))
            }
            OptimizationIntent::Balanced => {
                ("AdaptiveHybridStorage", 4, (data_elements as u64) / 2)
            }
        };

        let satisfied = est_latency <= budget.max_latency_us;

        ParetoOptimizationResult {
            chosen_intent: intent,
            representation: rep.to_string(),
            vectorized_simd_width: simd_width,
            estimated_latency_us: est_latency.max(10),
            budget_satisfied: satisfied,
        }
    }
}
