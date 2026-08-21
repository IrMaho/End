import time
import json
import sys
import os
import statistics
import socket
import subprocess

sys.stdout.reconfigure(encoding='utf-8')

BASE_DIR = os.path.dirname(os.path.abspath(__file__))

SERVERS = [
    ("C (GCC 15.2)", 9001, os.path.join(BASE_DIR, "server_c.exe")),
    ("Zig (0.16.0)", 9002, os.path.join(BASE_DIR, "server_zig.exe")),
    ("Rust (1.89.0)", 9003, os.path.join(BASE_DIR, "server_rust.exe")),
    ("Go (1.25.1)", 9004, os.path.join(BASE_DIR, "server_go.exe")),
    ("End Language", 9005, os.path.join(BASE_DIR, "server_end.exe")),
]

ENDPOINTS = [
    ("Health Check (Baseline Latency)", "/health"),
    ("CPU Compute (1M xorshift+LCG)", "/compute?n=1000000"),
    ("JSON Serialization", "/json"),
]

WARMUP = 20
BENCH_N = 200

def raw_http_get(port, path):
    """Ultra-fast raw TCP HTTP request with zero Python overhead"""
    t0 = time.perf_counter()
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(2.0)
        s.connect(('127.0.0.1', port))
        req = f"GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n".encode('ascii')
        s.sendall(req)
        chunks = []
        while True:
            chunk = s.recv(4096)
            if not chunk:
                break
            chunks.append(chunk)
        s.close()
        t1 = time.perf_counter()
        data = b"".join(chunks)
        if b"200 OK" in data:
            return (t1 - t0) * 1000.0, len(data), None
        else:
            return (t1 - t0) * 1000.0, len(data), f"HTTP status not 200"
    except Exception as e:
        t1 = time.perf_counter()
        return (t1 - t0) * 1000.0, 0, str(e)

def wait_server(port, timeout=4.0):
    t0 = time.time()
    while time.time() - t0 < timeout:
        ms, sz, err = raw_http_get(port, "/health")
        if err is None and sz > 0:
            return True
        time.sleep(0.05)
    return False

def bench_latency(port, path, n):
    lats = []
    errs = 0
    for _ in range(n):
        ms, sz, err = raw_http_get(port, path)
        if err is not None:
            errs += 1
        else:
            lats.append(ms)
    if not lats:
        return {"error": "all failed", "errors": errs}
    lats.sort()
    nn = len(lats)
    return {
        "total": n, "ok": nn, "errors": errs,
        "mean_ms": round(statistics.mean(lats), 3),
        "median_ms": round(lats[nn//2], 3),
        "p95_ms": round(lats[int(nn*0.95)], 3),
        "p99_ms": round(lats[int(nn*0.99)], 3),
        "min_ms": round(lats[0], 3),
        "max_ms": round(lats[-1], 3),
        "stdev_ms": round(statistics.stdev(lats), 3) if nn > 1 else 0,
        "rps": round(nn / (sum(lats)/1000.0), 1) if sum(lats) > 0 else 0,
    }

def bench_throughput(port, path, dur=3.0):
    cnt = 0
    errs = 0
    t0 = time.perf_counter()
    while time.perf_counter() - t0 < dur:
        ms, sz, err = raw_http_get(port, path)
        if err is None:
            cnt += 1
        else:
            errs += 1
    elapsed = time.perf_counter() - t0
    return {"total": cnt, "errors": errs, "elapsed_sec": round(elapsed, 2), "rps": round(cnt/elapsed, 1)}

def main():
    print("="*65)
    print("🚀 NATIVE HTTP BACKEND BENCHMARK SUITE (5 LANGUAGES)")
    print("="*65)

    results = {}

    for lang, port, binary in SERVERS:
        print(f"\n--- [{lang}] (port {port}) ---")

        # Start server as subprocess
        proc = subprocess.Popen(
            [binary],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )

        if not wait_server(port):
            print(f"  ❌ FAIL: Server {lang} did not respond on port {port}")
            try: proc.kill()
            except: pass
            continue

        print(f"  ✔ Server active (PID {proc.pid})")
        bsz = round(os.path.getsize(binary)/1024.0, 1)

        lr = {"binary_size_kb": bsz, "endpoints": {}, "throughput": {}}

        # Warmup
        for _ in range(WARMUP):
            raw_http_get(port, "/health")

        # Benchmark endpoints
        for epn, epp in ENDPOINTS:
            r = bench_latency(port, epp, BENCH_N)
            lr["endpoints"][epn] = r
            if "mean_ms" in r:
                print(f"  • {epn:<32} Mean: {r['mean_ms']:>6.2f} ms | P95: {r['p95_ms']:>6.2f} ms | RPS: {r['rps']:>7.1f}")
            else:
                print(f"  • {epn:<32} ERROR: {r}")

        # Throughput burst
        tp1 = bench_throughput(port, "/health", 3.0)
        lr["throughput"]["health_rps"] = tp1["rps"]
        print(f"  • Burst Throughput (Health)      : {tp1['rps']:>7.1f} req/s ({tp1['total']} reqs in {tp1['elapsed_sec']}s)")

        tp2 = bench_throughput(port, "/compute?n=1000000", 3.0)
        lr["throughput"]["compute_rps"] = tp2["rps"]
        print(f"  • Burst Throughput (Compute 1M)  : {tp2['rps']:>7.1f} req/s ({tp2['total']} reqs in {tp2['elapsed_sec']}s)")

        results[lang] = lr

        # Graceful terminate
        try:
            proc.terminate()
            proc.wait(timeout=2)
        except:
            proc.kill()
        time.sleep(0.3)

    # Save output
    out = os.path.join(BASE_DIR, "benchmark_results.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print("\n" + "="*65)
    print(f"✔ Benchmark successfully completed and saved to: {out}")
    print("="*65)

if __name__ == "__main__":
    main()
