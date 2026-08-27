use hdrhistogram::Histogram;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::report::{LatencyPercentiles, StressReport};

#[derive(Debug, Clone)]
pub struct StressConfig {
    pub target_url: String,
    pub concurrency: usize,
    pub duration: Option<Duration>,
    pub max_requests: Option<usize>,
    pub timeout: Duration,
    pub method: reqwest::Method,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            target_url: "http://127.0.0.1:8080".to_string(),
            concurrency: 10,
            duration: Some(Duration::from_secs(5)),
            max_requests: None,
            timeout: Duration::from_secs(10),
            method: reqwest::Method::GET,
            headers: HashMap::new(),
            body: None,
        }
    }
}

pub struct StressRunner {
    pub config: StressConfig,
}

#[derive(Debug)]
struct WorkerResult {
    histogram: Histogram<u64>,
    successful_requests: usize,
    failed_requests: usize,
    connection_errors: usize,
    timeout_errors: usize,
    status_codes: HashMap<u16, usize>,
}

impl StressRunner {
    pub fn new(target_url: &str) -> Self {
        Self {
            config: StressConfig {
                target_url: target_url.to_string(),
                ..Default::default()
            },
        }
    }

    pub fn concurrency(mut self, n: usize) -> Self {
        self.config.concurrency = n.max(1);
        self
    }

    pub fn duration(mut self, d: Duration) -> Self {
        self.config.duration = Some(d);
        self
    }

    pub fn max_requests(mut self, n: usize) -> Self {
        self.config.max_requests = Some(n);
        self
    }

    pub fn timeout(mut self, d: Duration) -> Self {
        self.config.timeout = d;
        self
    }

    pub fn method(mut self, m: reqwest::Method) -> Self {
        self.config.method = m;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.config.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, b: Vec<u8>) -> Self {
        self.config.body = Some(b);
        self
    }

    pub fn run(&self) -> Result<StressReport, String> {
        let config_clone = self.config.clone();
        std::thread::spawn(move || {
            let runner = Self {
                config: config_clone,
            };
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Failed to initialize tokio runtime for stress test: {}", e))?;

            rt.block_on(runner.run_async())
        })
        .join()
        .map_err(|_| "Stress runner thread panicked".to_string())?
    }

    pub async fn run_async(&self) -> Result<StressReport, String> {
        let concurrency = self.config.concurrency.max(1);
        let target_url = self.config.target_url.clone();
        let timeout = self.config.timeout;
        let method = self.config.method.clone();
        let headers = self.config.headers.clone();
        let body = self.config.body.clone();

        let client = Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(concurrency * 2)
            .tcp_nodelay(true)
            .build()
            .map_err(|e| format!("Failed to create reqwest HTTP client: {}", e))?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let requests_counter = Arc::new(AtomicUsize::new(0));

        let start_time = Instant::now();

        // Spawn timer if duration is specified
        if let Some(dur) = self.config.duration {
            let stop_clone = Arc::clone(&stop_flag);
            tokio::spawn(async move {
                tokio::time::sleep(dur).await;
                stop_clone.store(true, Ordering::SeqCst);
            });
        }

        let mut handles = Vec::with_capacity(concurrency);

        for _ in 0..concurrency {
            let client = client.clone();
            let url = target_url.clone();
            let method = method.clone();
            let headers = headers.clone();
            let body = body.clone();
            let stop_flag = Arc::clone(&stop_flag);
            let requests_counter = Arc::clone(&requests_counter);
            let max_requests = self.config.max_requests;

            let handle = tokio::spawn(async move {
                let mut hist = Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3)
                    .unwrap_or_else(|_| Histogram::<u64>::new(3).unwrap());
                let mut success = 0usize;
                let mut failed = 0usize;
                let mut conn_errs = 0usize;
                let mut timeout_errs = 0usize;
                let mut status_map = HashMap::<u16, usize>::new();

                while !stop_flag.load(Ordering::Relaxed) {
                    if let Some(max_req) = max_requests {
                        let current = requests_counter.fetch_add(1, Ordering::Relaxed);
                        if current >= max_req {
                            stop_flag.store(true, Ordering::Relaxed);
                            break;
                        }
                    }

                    let mut req_builder = client.request(method.clone(), &url);
                    for (k, v) in &headers {
                        req_builder = req_builder.header(k, v);
                    }
                    if let Some(b) = &body {
                        req_builder = req_builder.body(b.clone());
                    }

                    let req_start = Instant::now();
                    let resp_res = req_builder.send().await;
                    let latency_us = req_start.elapsed().as_micros() as u64;

                    hist.record(latency_us.max(1)).ok();

                    match resp_res {
                        Ok(resp) => {
                            let status = resp.status().as_u16();
                            *status_map.entry(status).or_insert(0) += 1;
                            if (200..=399).contains(&status) {
                                success += 1;
                            } else {
                                failed += 1;
                            }
                            let _ = resp.bytes().await;
                        }
                        Err(e) => {
                            failed += 1;
                            if e.is_timeout() {
                                timeout_errs += 1;
                            } else {
                                conn_errs += 1;
                            }
                            *status_map.entry(0).or_insert(0) += 1;
                        }
                    }
                }

                WorkerResult {
                    histogram: hist,
                    successful_requests: success,
                    failed_requests: failed,
                    connection_errors: conn_errs,
                    timeout_errors: timeout_errs,
                    status_codes: status_map,
                }
            });

            handles.push(handle);
        }

        let mut combined_hist = Histogram::<u64>::new_with_bounds(1, 60_000_000_000, 3)
            .unwrap_or_else(|_| Histogram::<u64>::new(3).unwrap());
        let mut total_success = 0usize;
        let mut total_failed = 0usize;
        let mut total_conn_errs = 0usize;
        let mut total_timeout_errs = 0usize;
        let mut combined_status_map = HashMap::<u16, usize>::new();

        for handle in handles {
            if let Ok(worker_res) = handle.await {
                combined_hist.add(&worker_res.histogram).ok();
                total_success += worker_res.successful_requests;
                total_failed += worker_res.failed_requests;
                total_conn_errs += worker_res.connection_errors;
                total_timeout_errs += worker_res.timeout_errors;

                for (code, count) in worker_res.status_codes {
                    *combined_status_map.entry(code).or_insert(0) += count;
                }
            }
        }

        let actual_duration = start_time.elapsed();
        let duration_s = actual_duration.as_secs_f64();
        let total_requests = total_success + total_failed;

        let throughput_rps = if duration_s > 0.0 {
            total_requests as f64 / duration_s
        } else {
            0.0
        };

        let error_rate_percent = if total_requests > 0 {
            (total_failed as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let p50_ms = combined_hist.value_at_quantile(0.50) as f64 / 1000.0;
        let p90_ms = combined_hist.value_at_quantile(0.90) as f64 / 1000.0;
        let p99_ms = combined_hist.value_at_quantile(0.99) as f64 / 1000.0;
        let p99_9_ms = combined_hist.value_at_quantile(0.999) as f64 / 1000.0;
        let min_ms = combined_hist.min() as f64 / 1000.0;
        let max_ms = combined_hist.max() as f64 / 1000.0;
        let mean_ms = combined_hist.mean() / 1000.0;

        Ok(StressReport {
            target_url,
            duration_s,
            concurrency,
            total_requests,
            successful_requests: total_success,
            failed_requests: total_failed,
            error_rate_percent,
            throughput_rps,
            latency: LatencyPercentiles {
                p50_ms,
                p90_ms,
                p99_ms,
                p99_9_ms,
                min_ms,
                max_ms,
                mean_ms,
            },
            status_codes: combined_status_map,
            connection_errors: total_conn_errs,
            timeout_errors: total_timeout_errs,
        })
    }
}
