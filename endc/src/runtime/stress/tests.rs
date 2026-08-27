#[cfg(test)]
mod stress_unit_and_integration_tests {
    use super::super::{LatencyPercentiles, StressReport, StressRunner, TestHttpServer};
    use std::time::Duration;

    #[test]
    fn test_01_report_serialization() {
        let mut status_codes = std::collections::HashMap::new();
        status_codes.insert(200, 4998);
        status_codes.insert(500, 2);

        let report = StressReport {
            target_url: "http://127.0.0.1:8080/test".to_string(),
            duration_s: 5.0,
            concurrency: 50,
            total_requests: 5000,
            successful_requests: 4998,
            failed_requests: 2,
            error_rate_percent: 0.04,
            throughput_rps: 1000.0,
            latency: LatencyPercentiles {
                p50_ms: 1.2,
                p90_ms: 2.1,
                p99_ms: 5.4,
                p99_9_ms: 12.3,
                min_ms: 0.5,
                max_ms: 20.0,
                mean_ms: 1.8,
            },
            status_codes,
            connection_errors: 0,
            timeout_errors: 0,
        };

        let json = serde_json::to_string(&report).expect("Serialization failed");
        let deserialized: StressReport = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.total_requests, 5000);
        assert_eq!(deserialized.successful_requests, 4998);
        assert_eq!(deserialized.failed_requests, 2);
        assert_eq!(deserialized.latency.p99_ms, 5.4);
    }

    #[tokio::test]
    async fn test_02_real_http_fast_endpoint() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let fast_url = server.url("/fast");

        let runner = StressRunner::new(&fast_url)
            .concurrency(10)
            .duration(Duration::from_millis(500));

        let report = runner.run_async().await.expect("Fast stress run failed");

        assert!(report.total_requests > 0, "Total requests must be > 0");
        assert_eq!(report.failed_requests, 0, "Fast endpoint should have 0 failures");
        assert_eq!(report.successful_requests, report.total_requests);
        assert_eq!(report.error_rate_percent, 0.0);
        assert!(report.throughput_rps > 0.0);
        assert!(report.latency.p99_ms < 20.0, "Fast p99 latency should be low (got {:.2} ms)", report.latency.p99_ms);
        assert!(report.status_codes.contains_key(&200));

        server.shutdown();
    }

    #[tokio::test]
    async fn test_03_real_http_slow_endpoint() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let slow_url = server.url("/slow");

        let runner = StressRunner::new(&slow_url)
            .concurrency(5)
            .max_requests(10);

        let report = runner.run_async().await.expect("Slow stress run failed");

        assert!(report.total_requests >= 5);
        assert_eq!(report.failed_requests, 0);
        // Each request has 50ms delay, so p50 and p99 should be >= 45ms
        assert!(
            report.latency.p50_ms >= 45.0,
            "Slow p50 should be >= 45ms, got {:.2} ms",
            report.latency.p50_ms
        );
        assert!(
            report.latency.p99_ms >= 45.0,
            "Slow p99 should be >= 45ms, got {:.2} ms",
            report.latency.p99_ms
        );

        server.shutdown();
    }

    #[tokio::test]
    async fn test_04_differential_fast_vs_slow() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let fast_url = server.url("/fast");
        let slow_url = server.url("/slow");

        let fast_report = StressRunner::new(&fast_url)
            .concurrency(5)
            .max_requests(20)
            .run_async()
            .await
            .expect("Fast run failed");

        let slow_report = StressRunner::new(&slow_url)
            .concurrency(5)
            .max_requests(10)
            .run_async()
            .await
            .expect("Slow run failed");

        println!("Fast p99: {:.2} ms, Slow p99: {:.2} ms", fast_report.latency.p99_ms, slow_report.latency.p99_ms);

        assert!(
            slow_report.latency.p99_ms > fast_report.latency.p99_ms,
            "Slow p99 ({:.2} ms) must be strictly greater than fast p99 ({:.2} ms)",
            slow_report.latency.p99_ms,
            fast_report.latency.p99_ms
        );
        assert!(slow_report.latency.p99_ms - fast_report.latency.p99_ms >= 30.0);

        server.shutdown();
    }

    #[tokio::test]
    async fn test_05_http_500_error_rate() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let error_url = server.url("/error");

        let report = StressRunner::new(&error_url)
            .concurrency(5)
            .max_requests(25)
            .run_async()
            .await
            .expect("Error run failed");

        assert!(report.total_requests >= 20);
        assert_eq!(report.successful_requests, 0);
        assert_eq!(report.failed_requests, report.total_requests);
        assert_eq!(report.error_rate_percent, 100.0);
        assert!(report.status_codes.contains_key(&500));
        assert_eq!(*report.status_codes.get(&500).unwrap(), report.total_requests);

        server.shutdown();
    }

    #[tokio::test]
    async fn test_06_reproducibility() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let delay_url = server.url("/custom_delay");

        let run1 = StressRunner::new(&delay_url)
            .concurrency(4)
            .max_requests(20)
            .run_async()
            .await
            .expect("Run 1 failed");

        let run2 = StressRunner::new(&delay_url)
            .concurrency(4)
            .max_requests(20)
            .run_async()
            .await
            .expect("Run 2 failed");

        let rps1 = run1.throughput_rps;
        let rps2 = run2.throughput_rps;
        let variance_pct = ((rps1 - rps2).abs() / ((rps1 + rps2) / 2.0)) * 100.0;

        println!("Run 1 RPS: {:.1}, Run 2 RPS: {:.1}, Variance: {:.2}%", rps1, rps2, variance_pct);
        assert!(
            variance_pct <= 20.0,
            "Throughput variance must be <= 20%, got {:.2}%",
            variance_pct
        );

        server.shutdown();
    }

    #[tokio::test]
    async fn test_07_concurrency_scaling() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let delay_url = server.url("/custom_delay");

        let low_c = StressRunner::new(&delay_url)
            .concurrency(1)
            .duration(Duration::from_millis(300))
            .run_async()
            .await
            .expect("Low concurrency failed");

        let high_c = StressRunner::new(&delay_url)
            .concurrency(10)
            .duration(Duration::from_millis(300))
            .run_async()
            .await
            .expect("High concurrency failed");

        println!("Concurrency 1 requests: {}, Concurrency 10 requests: {}", low_c.total_requests, high_c.total_requests);
        assert!(
            high_c.total_requests >= low_c.total_requests * 2,
            "High concurrency (10) should complete significantly more requests ({}) than low concurrency (1) ({})",
            high_c.total_requests,
            low_c.total_requests
        );

        server.shutdown();
    }

    #[tokio::test]
    async fn test_08_unreachable_target_handling() {
        // Target an unassigned local port
        let report = StressRunner::new("http://127.0.0.1:19")
            .concurrency(2)
            .max_requests(4)
            .timeout(Duration::from_millis(200))
            .run_async()
            .await
            .expect("Runner should return report with connection failures");

        assert!(report.total_requests > 0);
        assert_eq!(report.successful_requests, 0);
        assert_eq!(report.failed_requests, report.total_requests);
        assert!(report.connection_errors > 0 || report.timeout_errors > 0);
    }

    #[tokio::test]
    async fn test_09_cli_handle_stress_fast() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let fast_url = server.url("/fast");

        let args = crate::cli::dev_args::StressArgs {
            file: std::path::PathBuf::new(),
            url: Some(fast_url),
            concurrency: 5,
            duration: Some("500ms".to_string()),
            iterations: 100,
            json: true,
        };

        crate::driver::test_sim::handle_stress(args);
        server.shutdown();
    }

    #[tokio::test]
    async fn test_10_end_interpreter_builtins() {
        let server = TestHttpServer::start().expect("Failed to start server");
        let fast_url = server.url("/fast");

        let src = format!(r#"
fn main() {{
    val url = "{}"
    val report_json = end_stress_run(url, 5, 0.5)
    val p99 = end_stress_report_p99(report_json)
    val rps = end_stress_report_rps(report_json)
    val total = end_stress_report_total(report_json)
    val errs = end_stress_report_errors(report_json)
    ret total
}}
        "#, fast_url);

        let mut lexer = crate::lexer::Lexer::new("test_stress.end", &src);
        let tokens = lexer.tokenize_all().unwrap();
        let mut parser = crate::parser::Parser::new("test_stress.end", tokens);
        let module = parser.parse_module("test_stress").unwrap();
        let mut interp = crate::codegen::interpreter::Interpreter::new();
        let res = interp.run(&module).unwrap();

        if let crate::codegen::interpreter::Value::Int(t) = res {
            assert!(t > 0, "Total requests evaluated in End script should be > 0, got {}", t);
        } else {
            panic!("Expected total to be int, got {:?}", res);
        }

        server.shutdown();
    }
}
