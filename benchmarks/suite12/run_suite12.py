import subprocess
import time
import json
import statistics
import os
import sys

sys.stdout.reconfigure(encoding='utf-8')

SUITE_DIR = r"C:\Users\ASUS\Desktop\flutter_project\endApp\1\benchmarks\suite12"

BENCHMARK_NAMES = [
    "1. 3D SDF Raymarcher (250K Rays)",
    "2. Binary Trees (Depth 16 Dynamic)",
    "3. HFT Limit Order Engine (1M Orders)",
    "4. SHA-256 Crypto Hashing (500K Blocks)",
    "5. N-Body Gravity Orbit (1M Pairwise)",
    "6. SPSC Ring Buffer Queue (10M Items)",
    "7. DNA Levenshtein Matrix (1M Cells)",
    "8. JSON Microservice Serializer (100K)",
    "9. FSM Lexer Stream (10M Chars)",
    "10. GEMM Matrix Multiplication (512x512)",
    "11. Monte Carlo Black-Scholes (2M Paths)",
    "12. Super-Scalar ALU Reduction (10M)"
]

LANGUAGES = [
    {
        "name": "End (C11 Backend)",
        "icon": "👑",
        "file": "suite12_end.end",
        "exe": "suite12_end.exe",
        "compile": ["end.exe", "build", os.path.join(SUITE_DIR, "suite12_end.end"), "--strip", "-o", os.path.join(SUITE_DIR, "suite12_end.exe")]
    },
    {
        "name": "Zig (0.16.0)",
        "icon": "⚡",
        "file": "suite12_zig.zig",
        "exe": "suite12_zig.exe",
        "compile": ["zig", "build-exe", "-O", "ReleaseFast", "-lc", os.path.join(SUITE_DIR, "suite12_zig.zig"), f"-femit-bin={os.path.join(SUITE_DIR, 'suite12_zig.exe')}"]
    },
    {
        "name": "Rust (1.89.0)",
        "icon": "⚡",
        "file": "suite12_rust.rs",
        "exe": "suite12_rust.exe",
        "compile": ["rustc", "-O", "-C", "target-cpu=native", "-C", "lto=fat", "-C", "codegen-units=1", os.path.join(SUITE_DIR, "suite12_rust.rs"), "-o", os.path.join(SUITE_DIR, "suite12_rust.exe")]
    },
    {
        "name": "C (GCC 15.2)",
        "icon": "⚡",
        "file": "suite12_c.c",
        "exe": "suite12_c.exe",
        "compile": ["gcc", "-O3", "-march=native", "-flto", "-funroll-loops", "-fomit-frame-pointer", "-finline-functions", os.path.join(SUITE_DIR, "suite12_c.c"), "-o", os.path.join(SUITE_DIR, "suite12_c.exe")]
    },
    {
        "name": "Go (1.25.1)",
        "icon": "⚡",
        "file": "suite12_go.go",
        "exe": "suite12_go.exe",
        "compile": ["go", "build", "-ldflags=-s -w", "-o", os.path.join(SUITE_DIR, "suite12_go.exe"), os.path.join(SUITE_DIR, "suite12_go.go")]
    }
]

print("=" * 70)
print("🔨 COMPILING 12-CHALLENGE SUITE ACROSS ALL 5 LANGUAGES")
print("=" * 70)

for lang in LANGUAGES:
    print(f"Compiling {lang['name']}...")
    res = subprocess.run(lang["compile"], capture_output=True, text=True)
    exe_path = os.path.join(SUITE_DIR, lang["exe"])
    if os.path.exists(exe_path):
        size_kb = os.path.getsize(exe_path) / 1024.0
        lang["size_kb"] = size_kb
        print(f"  ✔ {lang['name']} compiled successfully ({size_kb:.1f} KB)")
    else:
        print(f"  ❌ Error compiling {lang['name']}:\n{res.stderr}")
        sys.exit(1)

print("\n" + "=" * 70)
print("🚀 EXECUTING 12 BENCHMARKS (5 Runs per test, 2 Warmups)")
print("=" * 70)

results = {
    "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
    "benchmarks": {}
}

for b_idx in range(1, 13):
    b_name = BENCHMARK_NAMES[b_idx - 1]
    print(f"\n📊 Benchmark {b_idx}/12: {b_name}")
    results["benchmarks"][b_name] = {}

    for lang in LANGUAGES:
        exe_path = os.path.join(SUITE_DIR, lang["exe"])
        
        # Warmup
        for _ in range(2):
            subprocess.run([exe_path, str(b_idx)], capture_output=True, text=True)
        
        times = []
        checksum = ""
        for _ in range(5):
            res = subprocess.run([exe_path, str(b_idx)], capture_output=True, text=True)
            output = res.stderr.strip() if res.stderr else res.stdout.strip()
            for line in output.split('\n'):
                if "RESULT:" in line:
                    parts = line.split(',')
                    for p in parts:
                        if "time_ms=" in p:
                            times.append(float(p.split('=')[1]))
                        if "checksum=" in p:
                            checksum = p.split('=')[1]

        times.sort()
        mean_t = statistics.mean(times)
        p50_t = times[len(times)//2]
        std_t = statistics.stdev(times) if len(times) > 1 else 0.0

        results["benchmarks"][b_name][lang["name"]] = {
            "mean_ms": round(mean_t, 2),
            "p50_ms": round(p50_t, 2),
            "std_dev": round(std_t, 2),
            "min_ms": round(min(times), 2),
            "max_ms": round(max(times), 2),
            "checksum": checksum,
            "binary_kb": round(lang.get("size_kb", 0), 1)
        }
        print(f"  {lang['icon']} {lang['name']:<20}: Mean = {mean_t:7.2f} ms | P50 = {p50_t:7.2f} ms | Checksum = {checksum}")

# Save JSON
results_json_path = os.path.join(SUITE_DIR, "suite12_results.json")
with open(results_json_path, "w", encoding="utf-8") as f:
    json.dump(results, f, indent=2)

print(f"\n✔ Full dataset saved to: {results_json_path}")
