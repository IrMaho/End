import os
import subprocess
import re
import statistics
import sys
import json

sys.stdout.reconfigure(encoding='utf-8')

BASE_DIR = os.path.dirname(os.path.abspath(__file__))

LANGS = {
    "C (GCC 15.2)": {
        "binary": os.path.join(BASE_DIR, "brutal_c.exe"),
        "compile_cmd": ["gcc", "-O3", "-march=native", "-flto", os.path.join(BASE_DIR, "brutal_c.c"), "-o", os.path.join(BASE_DIR, "brutal_c.exe")]
    },
    "Zig (0.16.0)": {
        "binary": os.path.join(BASE_DIR, "brutal_zig.exe"),
        "compile_cmd": ["zig", "build-exe", "-O", "ReleaseFast", "-lc", os.path.join(BASE_DIR, "brutal_zig.zig")]
    },
    "Rust (1.89.0)": {
        "binary": os.path.join(BASE_DIR, "brutal_rust.exe"),
        "compile_cmd": ["rustc", "-O", "-C", "target-cpu=native", "-C", "lto=fat", "-C", "codegen-units=1", os.path.join(BASE_DIR, "brutal_rust.rs"), "-o", os.path.join(BASE_DIR, "brutal_rust.exe")]
    },
    "Go (1.25.1)": {
        "binary": os.path.join(BASE_DIR, "brutal_go.exe"),
        "compile_cmd": ["go", "build", "-ldflags=-s -w", "-o", os.path.join(BASE_DIR, "brutal_go.exe"), os.path.join(BASE_DIR, "brutal_go.go")]
    },
    "End Language": {
        "binary": os.path.join(BASE_DIR, "brutal_end.exe"),
        "compile_cmd": ["end.exe", "build", os.path.join(BASE_DIR, "brutal_end.end"), "--strip", "-o", os.path.join(BASE_DIR, "brutal_end.exe")]
    }
}

ITERATIONS = 5

def compile_all():
    print("="*65)
    print("🔨 COMPILING 3 BRUTAL CHALLENGES FOR ALL 5 LANGUAGES")
    print("="*65)
    for lang, meta in LANGS.items():
        print(f"Compiling {lang}...")
        res = subprocess.run(meta["compile_cmd"], capture_output=True, text=True)
        # Move zig if needed
        if lang == "Zig (0.16.0)":
            if os.path.exists("brutal_zig.exe"):
                import shutil
                shutil.move("brutal_zig.exe", meta["binary"])
        if not os.path.exists(meta["binary"]):
            print(f"[FAIL] {lang} compilation failed:\n{res.stderr}\n{res.stdout}")
            sys.exit(1)
        size_kb = os.path.getsize(meta["binary"]) / 1024.0
        print(f"  ✔ {lang} ready | Binary size: {size_kb:.1f} KB")

def run_suite():
    print("\n" + "="*65)
    print("🚀 EXECUTING BRUTAL BENCHMARK SUITE (5 Iterations per Language)")
    print("="*65)

    results = {}

    for lang, meta in LANGS.items():
        print(f"\n--- [{lang}] ---")
        ray_times = []
        tree_times = []
        hft_times = []
        checksums = {}

        for it in range(1, ITERATIONS + 1):
            proc = subprocess.run([os.path.abspath(meta["binary"])], capture_output=True, text=True)
            out = (proc.stdout or "") + "\n" + (proc.stderr or "")

            m_ray = re.search(r"1\.\s+Raymarcher.*?:\s+([\d\.]+)\s+ms.*?Checksum:\s+(\d+)", out)
            m_tree = re.search(r"2\.\s+Binary Trees.*?:\s+([\d\.]+)\s+ms.*?Checksum:\s+([-\d]+)", out)
            m_hft = re.search(r"3\.\s+HFT Order.*?:\s+([\d\.]+)\s+ms.*?(?:Vol:\s+(\d+)|Trades:\s+\d+\s+\|\s+Vol:\s+(\d+))", out)

            if not (m_ray and m_tree and m_hft):
                print(f"  ❌ Output parsing failed on iteration {it}:\n{out}")
                continue

            t_ray = float(m_ray.group(1))
            c_ray = m_ray.group(2)
            t_tree = float(m_tree.group(1))
            c_tree = m_tree.group(2)
            t_hft = float(m_hft.group(1))
            c_hft = m_hft.group(2) if m_hft.group(2) else m_hft.group(3)

            ray_times.append(t_ray)
            tree_times.append(t_tree)
            hft_times.append(t_hft)

            checksums["Raymarch_Checksum"] = c_ray
            checksums["BinaryTrees_Checksum"] = c_tree
            checksums["HFT_Volume_Checksum"] = c_hft

            print(f"  Run {it}: Raymarcher={t_ray:>6.2f}ms | BinaryTrees={t_tree:>6.2f}ms | HFT={t_hft:>5.2f}ms")

        size_kb = os.path.getsize(meta["binary"]) / 1024.0
        results[lang] = {
            "raymarcher_3d": {
                "mean_ms": round(statistics.mean(ray_times), 2),
                "min_ms": round(min(ray_times), 2),
                "max_ms": round(max(ray_times), 2),
                "stdev_ms": round(statistics.stdev(ray_times), 2) if len(ray_times) > 1 else 0.0,
            },
            "binary_trees_depth16": {
                "mean_ms": round(statistics.mean(tree_times), 2),
                "min_ms": round(min(tree_times), 2),
                "max_ms": round(max(tree_times), 2),
                "stdev_ms": round(statistics.stdev(tree_times), 2) if len(tree_times) > 1 else 0.0,
            },
            "hft_order_matching": {
                "mean_ms": round(statistics.mean(hft_times), 2),
                "min_ms": round(min(hft_times), 2),
                "max_ms": round(max(hft_times), 2),
                "stdev_ms": round(statistics.stdev(hft_times), 2) if len(hft_times) > 1 else 0.0,
            },
            "binary_size_kb": round(size_kb, 1),
            "checksums": checksums
        }

    return results

if __name__ == "__main__":
    compile_all()
    res = run_suite()
    out_file = os.path.join(BASE_DIR, "brutal_results.json")
    with open(out_file, "w", encoding="utf-8") as f:
        json.dump(res, f, indent=2)
    print("\n" + "="*65)
    print(f"✔ BRUTAL BENCHMARK COMPLETED AND SAVED TO: {out_file}")
    print("="*65)
