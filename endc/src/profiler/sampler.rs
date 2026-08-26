use std::collections::HashMap;
use std::time::Instant;
use super::types::{CallGraphEdge, ProfileSample, ProfilingReport};
use super::flamegraph::FlameGraphGenerator;

#[derive(Debug, Clone)]
pub struct FunctionMetric {
    pub function_name: String,
    pub call_count: usize,
    pub total_duration_us: u64,
    pub self_duration_us: u64,
    pub memory_allocated_bytes: usize,
    pub samples: usize,
}

#[derive(Debug, Clone)]
pub struct CallGraphMetric {
    pub caller: String,
    pub callee: String,
    pub call_count: usize,
    pub total_duration_us: u64,
}

#[derive(Debug, Clone)]
pub struct ProfilerSession {
    pub target: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub call_stack: Vec<String>,
    pub call_stack_samples: Vec<String>,
    pub function_metrics: HashMap<String, FunctionMetric>,
    pub call_graph: HashMap<(String, String), CallGraphMetric>,
    pub is_active: bool,
}

impl ProfilerSession {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            start_time: Instant::now(),
            end_time: None,
            call_stack: Vec::new(),
            call_stack_samples: Vec::new(),
            function_metrics: HashMap::new(),
            call_graph: HashMap::new(),
            is_active: true,
        }
    }

    pub fn enter_function(&mut self, func_name: &str) {
        if !self.is_active {
            return;
        }
        let caller = self.call_stack.last().cloned().unwrap_or_else(|| "root".to_string());
        self.call_stack.push(func_name.to_string());

        let stack_str = self.call_stack.join(";");
        self.call_stack_samples.push(stack_str);

        let metric = self.function_metrics.entry(func_name.to_string()).or_insert_with(|| FunctionMetric {
            function_name: func_name.to_string(),
            call_count: 0,
            total_duration_us: 0,
            self_duration_us: 0,
            memory_allocated_bytes: 0,
            samples: 0,
        });
        metric.call_count += 1;
        metric.samples += 1;

        let edge = self.call_graph.entry((caller.clone(), func_name.to_string())).or_insert_with(|| CallGraphMetric {
            caller,
            callee: func_name.to_string(),
            call_count: 0,
            total_duration_us: 0,
        });
        edge.call_count += 1;
    }

    pub fn exit_function(&mut self, func_name: &str, duration_us: u64, self_duration_us: u64, mem_bytes: usize) {
        if !self.is_active {
            return;
        }
        if let Some(pos) = self.call_stack.iter().rposition(|f| f == func_name) {
            self.call_stack.truncate(pos);
        }

        if let Some(metric) = self.function_metrics.get_mut(func_name) {
            metric.total_duration_us += duration_us;
            metric.self_duration_us += self_duration_us;
            metric.memory_allocated_bytes += mem_bytes;
        }
    }

    pub fn finish(&mut self) -> ProfilingReport {
        self.is_active = false;
        let end = Instant::now();
        self.end_time = Some(end);
        let total_runtime_ms = end.duration_since(self.start_time).as_secs_f64() * 1000.0;

        let mut total_duration_all: u64 = self.function_metrics.values().map(|m| m.self_duration_us.max(m.total_duration_us)).sum();
        if total_duration_all == 0 {
            total_duration_all = (total_runtime_ms * 1000.0) as u64;
        }
        if total_duration_all == 0 {
            total_duration_all = 1;
        }

        let total_samples: usize = self.call_stack_samples.len().max(1);

        let mut samples_vec: Vec<ProfileSample> = self.function_metrics.values().map(|m| {
            let metric_dur = if m.self_duration_us > 0 { m.self_duration_us } else { m.total_duration_us };
            let percent = ((metric_dur as f64) / (total_duration_all as f64)) * 100.0;
            ProfileSample {
                function_name: m.function_name.clone(),
                call_count: m.call_count,
                total_duration_us: m.total_duration_us,
                self_duration_us: m.self_duration_us,
                memory_allocated_bytes: m.memory_allocated_bytes,
                percent: percent.min(100.0),
                sample_count: m.samples,
            }
        }).collect();

        // Sort samples by duration descending
        samples_vec.sort_by(|a, b| b.total_duration_us.cmp(&a.total_duration_us));

        // Group folded stacks
        let mut stack_counts: HashMap<String, usize> = HashMap::new();
        for stack in &self.call_stack_samples {
            *stack_counts.entry(stack.clone()).or_insert(0) += 1;
        }

        let mut folded_stacks = String::new();
        for (stack, count) in &stack_counts {
            folded_stacks.push_str(&format!("{} {}\n", stack, count));
        }

        let root_frame = FlameGraphGenerator::parse_folded_stacks(&folded_stacks);
        let flamegraph_svg = FlameGraphGenerator::generate_svg(&root_frame, &self.target, total_runtime_ms);

        let call_graph: Vec<CallGraphEdge> = self.call_graph.values().map(|e| CallGraphEdge {
            caller: e.caller.clone(),
            callee: e.callee.clone(),
            call_count: e.call_count,
            total_duration_us: e.total_duration_us,
        }).collect();

        let total_memory_kb = self.function_metrics.values().map(|m| m.memory_allocated_bytes).sum::<usize>() / 1024;

        ProfilingReport {
            target: self.target.clone(),
            total_runtime_ms,
            total_memory_kb: total_memory_kb.max(32),
            total_samples,
            flamegraph_svg,
            samples: samples_vec,
            call_graph,
            folded_stacks,
        }
    }
}
