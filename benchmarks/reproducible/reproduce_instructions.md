# ?? Reproducible Benchmark Suite for End Language

This directory contains self-contained benchmark sources, automation scripts, and raw execution logs allowing any developer or reviewer to reproduce the multi-language performance measurements independently on their own machine.

---

## ?? How to Run Locally

### Windows (PowerShell)
```powershell
powershell -ExecutionPolicy Bypass -File .\run_all.ps1
```

### Linux / macOS (Bash)
```bash
chmod +x run_all.sh
./run_all.sh
```

---

## ?? Workload Description

- **Workload:** 10,000,000 element numeric reduction with anti-optimization checksum verification.
- **Statistical Outputs:** Min, Max, Mean, Median, P50, P90, P95, P99, and Standard Deviation (StdDev) in milliseconds.
- **Artifact:** Generates `results_live.json` capturing system CPU, OS, commit SHA, and exact timings.
