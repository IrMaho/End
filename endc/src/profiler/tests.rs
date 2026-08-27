#[cfg(test)]
mod profiler_unit_and_integration_tests {
    use crate::profiler::{EndProfiler, FlameGraphGenerator, ProfileSample, ProfilingReport};

    #[test]
    fn test_01_report_serialization() {
        let sample = ProfileSample {
            function_name: "compute_hash".to_string(),
            call_count: 100,
            total_duration_us: 5000,
            self_duration_us: 4500,
            memory_allocated_bytes: 1024,
            percent: 90.0,
            sample_count: 100,
        };

        let report = ProfilingReport {
            target: "test_target.end".to_string(),
            total_runtime_ms: 5.5,
            total_memory_kb: 64,
            total_samples: 100,
            flamegraph_svg: "<svg></svg>".to_string(),
            samples: vec![sample],
            call_graph: vec![],
            folded_stacks: "main;compute_hash 100\n".to_string(),
        };

        let json = serde_json::to_string(&report).expect("Must serialize to JSON");
        assert!(json.contains("compute_hash"));
        assert!(json.contains("test_target.end"));

        let deserialized: ProfilingReport = serde_json::from_str(&json).expect("Must deserialize from JSON");
        assert_eq!(deserialized.samples.len(), 1);
        assert_eq!(deserialized.samples[0].function_name, "compute_hash");
    }

    #[test]
    fn test_02_flamegraph_svg_generation() {
        let folded = "main 5\nmain;work_alpha 40\nmain;work_alpha;deep_math 55\n";
        let root = FlameGraphGenerator::parse_folded_stacks(folded);
        assert_eq!(root.samples, 100);

        let svg = FlameGraphGenerator::generate_svg(&root, "test_app", 10.5);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
        assert!(svg.contains("test_app"));
        assert!(svg.contains("work_alpha"));
        assert!(svg.contains("deep_math"));
        assert!(svg.contains("viewBox="));
    }

    #[test]
    fn test_03_cpu_bound_profiling() {
        let source = r#"
fn fib(n: i64) i64 {
    if n <= 1 {
        ret n
    }
    ret fib(n - 1) + fib(n - 2)
}

fn main() void {
    val res = fib(12)
    println(res)
}
"#;

        let report = EndProfiler::profile_source(source, "cpu_fib_test").expect("Must profile source successfully");
        assert!(report.total_runtime_ms > 0.0);
        assert!(!report.samples.is_empty(), "Must have collected function samples");

        // The dominant function MUST be `fib`
        let fib_sample = report.samples.iter().find(|s| s.function_name == "fib");
        assert!(fib_sample.is_some(), "Must have profiled 'fib' function");

        let fib = fib_sample.unwrap();
        assert!(fib.call_count > 100, "fib(12) must have over 100 recursive calls, got {}", fib.call_count);
        assert!(fib.percent > 40.0, "fib must consume >40% of runtime in CPU-bound workload, got {:.2}%", fib.percent);
        assert!(report.flamegraph_svg.contains("fib"), "Flamegraph SVG must contain fib frame");
    }

    #[test]
    fn test_04_io_sleep_bound_profiling() {
        let source = r#"
fn do_io_delay() void {
    cpu_sleep_ms(30)
}

fn main() void {
    do_io_delay()
}
"#;

        let report = EndProfiler::profile_source(source, "io_sleep_test").expect("Must profile source successfully");
        assert!(report.total_runtime_ms >= 20.0, "Total runtime must reflect the 30ms sleep");

        // The dominant function MUST be `cpu_sleep_ms` or `do_io_delay`
        let sleep_sample = report.samples.iter().find(|s| s.function_name == "cpu_sleep_ms" || s.function_name == "do_io_delay");
        assert!(sleep_sample.is_some(), "Must have captured sleep / IO function");

        let sleep = sleep_sample.unwrap();
        assert!(sleep.total_duration_us >= 20_000, "Sleep duration must be >= 20ms (20,000 µs), got {} µs", sleep.total_duration_us);
    }

    #[test]
    fn test_05_differential_profiling_assertion() {
        let cpu_source = r#"
fn fib(n: i64) i64 {
    if n <= 1 { ret n }
    ret fib(n - 1) + fib(n - 2)
}
fn main() void {
    val r = fib(10)
}
"#;

        let io_source = r#"
fn io_wait() void {
    cpu_sleep_ms(25)
}
fn main() void {
    io_wait()
}
"#;

        let cpu_report = EndProfiler::profile_source(cpu_source, "cpu_workload").unwrap();
        let io_report = EndProfiler::profile_source(io_source, "io_workload").unwrap();

        // 1. Top functions must differ
        let cpu_top = &cpu_report.samples[0].function_name;
        let io_top = &io_report.samples[0].function_name;
        assert_ne!(cpu_top, io_top, "Top function for CPU ({}) must differ from IO ({})", cpu_top, io_top);

        // 2. Call counts must differ drastically (recursive compute vs sleep)
        let cpu_calls: usize = cpu_report.samples.iter().map(|s| s.call_count).sum();
        let io_calls: usize = io_report.samples.iter().map(|s| s.call_count).sum();
        assert!(cpu_calls > io_calls * 10, "CPU calls ({}) must be significantly higher than IO calls ({})", cpu_calls, io_calls);
    }

    #[test]
    fn test_06_reproducibility_under_20_percent_variance() {
        let source = r#"
fn compute_heavy(iterations: i64) i64 {
    var sum: i64 = 0
    var i: i64 = 0
    while i < iterations {
        sum = sum + i
        i = i + 1
    }
    ret sum
}

fn main() void {
    val r = compute_heavy(5000)
}
"#;

        let run1 = EndProfiler::profile_source(source, "run1").unwrap();
        let run2 = EndProfiler::profile_source(source, "run2").unwrap();

        let s1 = run1.samples.iter().find(|s| s.function_name == "compute_heavy").expect("compute_heavy in run1");
        let s2 = run2.samples.iter().find(|s| s.function_name == "compute_heavy").expect("compute_heavy in run2");

        let p1 = s1.percent;
        let p2 = s2.percent;
        let diff = (p1 - p2).abs();
        let max_val = p1.max(p2).max(1.0);
        let variance_pct = (diff / max_val) * 100.0;

        assert!(
            variance_pct < 20.0,
            "Variance between identical runs must be < 20.0%, got {:.2}% (Run 1: {:.2}%, Run 2: {:.2}%)",
            variance_pct, p1, p2
        );
    }

    #[test]
    fn test_07_honest_error_handling_on_nonexistent_target() {
        let report = EndProfiler::profile_execution("non_existent_file_xyz123.end");
        assert_eq!(report.samples.len(), 0, "Non-existent target must not return fabricated samples");
        assert!(report.total_runtime_ms < 50.0);
    }
}
