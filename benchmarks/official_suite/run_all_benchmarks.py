import os
import subprocess
import re
import statistics
import sys
import json

# Ensure UTF-8 output
sys.stdout.reconfigure(encoding='utf-8')

LANGS = {
    "C (GCC 15.2)": {
        "source": "benchmarks/bench_c.c",
        "binary": "benchmarks/bench_c.exe",
        "compile_cmd": ["gcc", "-O3", "-march=native", "-flto", "benchmarks/bench_c.c", "-o", "benchmarks/bench_c.exe"]
    },
    "Zig (0.16.0)": {
        "source": "benchmarks/bench_zig.zig",
        "binary": "bench_zig.exe",
        "compile_cmd": ["zig", "build-exe", "-O", "ReleaseFast", "-lc", "benchmarks/bench_zig.zig"]
    },
    "Rust (1.89.0)": {
        "source": "benchmarks/bench_rust.rs",
        "binary": "benchmarks/bench_rust.exe",
        "compile_cmd": ["rustc", "-O", "-C", "target-cpu=native", "-C", "lto=fat", "-C", "codegen-units=1", "benchmarks/bench_rust.rs", "-o", "benchmarks/bench_rust.exe"]
    },
    "Go (1.25.1)": {
        "source": "benchmarks/bench_go.go",
        "binary": "benchmarks/bench_go.exe",
        "compile_cmd": ["go", "build", "-ldflags=-s -w", "-o", "benchmarks/bench_go.exe", "benchmarks/bench_go.go"]
    },
    "End (end.exe + GCC -O3)": {
        "source": "benchmarks/bench_end.end",
        "binary": "benchmarks/bench_end.exe",
        "compile_cmd": ["end.exe", "build", "benchmarks/bench_end.end", "--strip", "-o", "benchmarks/bench_end.exe"],
        "post_compile_cmd": ["gcc", "-O3", "-march=native", "-flto", "benchmarks/bench_end.c", "-o", "benchmarks/bench_end.exe"]
    }
}

ITERATIONS = 5

def compile_all():
    print("==================================================================")
    print("COMPILING BENCHMARK SUITE FOR ALL 5 LANGUAGES")
    print("==================================================================")
    for lang, meta in LANGS.items():
        print(f"Compiling {lang}...")
        res = subprocess.run(meta["compile_cmd"], capture_output=True, text=True)
        if "post_compile_cmd" in meta:
            res = subprocess.run(meta["post_compile_cmd"], capture_output=True, text=True)
        if not os.path.exists(meta["binary"]):
            print(f"[FAIL] Failed to compile {lang}:\n{res.stderr}\n{res.stdout}")
            sys.exit(1)
        size_kb = os.path.getsize(meta["binary"]) / 1024.0
        print(f"  [OK] {lang} compiled successfully! Binary size: {size_kb:.1f} KB")

def run_benchmarks():
    print("\n==================================================================")
    print("RUNNING STANDARDIZED MULTI-RUN BENCHMARKS (5 Iterations Each)")
    print("==================================================================")
    
    results = {}
    
    for lang, meta in LANGS.items():
        print(f"\nEvaluating {lang}:")
        compute_times = []
        memory_times = []
        recursion_times = []
        checksums = {}
        
        for it in range(1, ITERATIONS + 1):
            cmd = [os.path.abspath(meta["binary"])]
            proc = subprocess.run(cmd, capture_output=True, text=True)
            out = (proc.stdout or "") + "\n" + (proc.stderr or "")
            
            # Parse output
            m_comp = re.search(r"1\.\s+Compute.*?:\s+([\d\.]+)\s+ms.*?Hash:\s+(\d+)", out)
            m_mem = re.search(r"2\.\s+Memory Churn.*?:\s+([\d\.]+)\s+ms.*?Sum:\s+(\d+)", out)
            m_rec = re.search(r"3\.\s+Recursion.*?:\s+([\d\.]+)\s+ms.*?Val:\s+(\d+)", out)
            
            if not (m_comp and m_mem and m_rec):
                print(f"  [FAIL] Parse error on iteration {it}:\n{out}")
                continue
                
            t_comp = float(m_comp.group(1))
            h_comp = m_comp.group(2)
            t_mem = float(m_mem.group(1))
            h_mem = m_mem.group(2)
            t_rec = float(m_rec.group(1))
            h_rec = m_rec.group(2)
            
            compute_times.append(t_comp)
            memory_times.append(t_mem)
            recursion_times.append(t_rec)
            checksums["Hash"] = h_comp
            checksums["Sum"] = h_mem
            checksums["Val"] = h_rec
            
            print(f"  Iteration {it}: Compute={t_comp:.2f}ms | Memory={t_mem:.2f}ms | Recursion={t_rec:.2f}ms")
            
        size_kb = os.path.getsize(meta["binary"]) / 1024.0
        results[lang] = {
            "compute": {
                "mean": statistics.mean(compute_times),
                "min": min(compute_times),
                "max": max(compute_times),
                "stdev": statistics.stdev(compute_times) if len(compute_times) > 1 else 0.0
            },
            "memory": {
                "mean": statistics.mean(memory_times),
                "min": min(memory_times),
                "max": max(memory_times),
                "stdev": statistics.stdev(memory_times) if len(memory_times) > 1 else 0.0
            },
            "recursion": {
                "mean": statistics.mean(recursion_times),
                "min": min(recursion_times),
                "max": max(recursion_times),
                "stdev": statistics.stdev(recursion_times) if len(recursion_times) > 1 else 0.0
            },
            "size_kb": size_kb,
            "checksums": checksums
        }
        
    return results

if __name__ == "__main__":
    compile_all()
    res = run_benchmarks()
    print("\n\n==================================================================")
    print("FINAL JSON REPORT")
    print("==================================================================")
    print(json.dumps(res, indent=2))
