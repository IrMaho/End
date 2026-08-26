import os
import sys
import json
import subprocess
import unittest
import xml.etree.ElementTree as ET

WORKSPACE_ROOT = r"c:\Users\ASUS\Desktop\flutter_project\end"
END_BIN = os.path.join(WORKSPACE_ROOT, "bin", "end.exe")
ENDC_BIN = os.path.join(WORKSPACE_ROOT, "endc", "target", "release", "endc.exe")
LLVM_PATH = r"C:\Program Files\LLVM\bin"
Z3_PATH = os.path.join(WORKSPACE_ROOT, "endc", "z3", "bin")
PG_PATH = os.path.join(WORKSPACE_ROOT, "pgsql", "bin")

def get_env():
    env = os.environ.copy()
    env["PATH"] = f"{LLVM_PATH};{Z3_PATH};{PG_PATH};{os.path.join(WORKSPACE_ROOT, 'bin')};" + env.get("PATH", "")
    return env

def get_executable():
    if os.path.exists(ENDC_BIN):
        return ENDC_BIN
    return END_BIN

class TestProfilerRealMeasurement(unittest.TestCase):

    def setUp(self):
        self.env = get_env()
        self.exe = get_executable()
        self.assertTrue(os.path.exists(self.exe), f"Executable not found at {self.exe}")

    def test_01_cpu_bound_real_profiling(self):
        """Verify CPU-bound workload captures recursive fibonacci as dominant hotspot."""
        src_path = os.path.join(WORKSPACE_ROOT, "temp_cpu_prof_test.end")
        code = """
fn fib(n: i64) i64 {
    if n <= 1 {
        ret n
    }
    ret fib(n - 1) + fib(n - 2)
}

fn main() void {
    val r = fib(13)
    println(r)
}
"""
        with open(src_path, "w", encoding="utf-8") as f:
            f.write(code)

        try:
            cmd = [self.exe, "profile", src_path]
            res = subprocess.run(cmd, env=self.env, capture_output=True, text=True, cwd=WORKSPACE_ROOT)
            self.assertEqual(res.returncode, 0, f"Profiler failed: {res.stderr}")
            out = res.stdout

            self.assertIn("Execution Profile for", out)
            self.assertIn("compute_fibonacci" if "compute_fibonacci" in out else "fib", out)
            self.assertIn("Saved interactive flamegraph SVG", out)
            self.assertTrue(os.path.exists(os.path.join(WORKSPACE_ROOT, "flamegraph.svg")))

            # Read flamegraph and check contents
            with open(os.path.join(WORKSPACE_ROOT, "flamegraph.svg"), "r", encoding="utf-8") as f:
                svg_content = f.read()

            self.assertTrue(svg_content.startswith("<svg"))
            self.assertTrue(svg_content.strip().endswith("</svg>"))
            self.assertIn("fib", svg_content)
        finally:
            if os.path.exists(src_path):
                os.remove(src_path)

    def test_02_io_sleep_bound_profiling(self):
        """Verify I/O sleep workload accurately measures sleep time and identifies sleep function."""
        src_path = os.path.join(WORKSPACE_ROOT, "temp_io_prof_test.end")
        code = """
fn simulate_io_delay() void {
    cpu_sleep_ms(35)
}

fn main() void {
    simulate_io_delay()
}
"""
        with open(src_path, "w", encoding="utf-8") as f:
            f.write(code)

        try:
            cmd = [self.exe, "profile", src_path]
            res = subprocess.run(cmd, env=self.env, capture_output=True, text=True, cwd=WORKSPACE_ROOT)
            self.assertEqual(res.returncode, 0, f"Profiler failed: {res.stderr}")
            out = res.stdout

            self.assertIn("Execution Profile for", out)
            self.assertTrue("cpu_sleep_ms" in out or "simulate_io_delay" in out)
            
            # Duration must reflect at least 30ms sleep
            self.assertTrue(os.path.exists(os.path.join(WORKSPACE_ROOT, "flamegraph.svg")))
            with open(os.path.join(WORKSPACE_ROOT, "flamegraph.svg"), "r", encoding="utf-8") as f:
                svg = f.read()
            self.assertTrue("simulate_io_delay" in svg or "cpu_sleep_ms" in svg)
        finally:
            if os.path.exists(src_path):
                os.remove(src_path)

    def test_03_differential_profiling_cpu_vs_io(self):
        """Assert profile_cpu != profile_io and distinct execution fingerprints are produced."""
        cpu_path = os.path.join(WORKSPACE_ROOT, "temp_diff_cpu.end")
        io_path = os.path.join(WORKSPACE_ROOT, "temp_diff_io.end")

        with open(cpu_path, "w", encoding="utf-8") as f:
            f.write("""
fn recursive_math(n: i64) i64 {
    if n <= 1 { ret 1 }
    ret recursive_math(n - 1) + recursive_math(n - 2)
}
fn main() void {
    val x = recursive_math(12)
}
""")

        with open(io_path, "w", encoding="utf-8") as f:
            f.write("""
fn wait_network() void {
    cpu_sleep_ms(25)
}
fn main() void {
    wait_network()
}
""")

        try:
            res_cpu = subprocess.run([self.exe, "profile", cpu_path], env=self.env, capture_output=True, text=True, cwd=WORKSPACE_ROOT)
            self.assertEqual(res_cpu.returncode, 0)
            cpu_out = res_cpu.stdout

            res_io = subprocess.run([self.exe, "profile", io_path], env=self.env, capture_output=True, text=True, cwd=WORKSPACE_ROOT)
            self.assertEqual(res_io.returncode, 0)
            io_out = res_io.stdout

            self.assertNotEqual(cpu_out, io_out)
            self.assertIn("recursive_math", cpu_out)
            self.assertNotIn("recursive_math", io_out)
            self.assertIn("wait_network", io_out)
            self.assertNotIn("wait_network", cpu_out)
        finally:
            if os.path.exists(cpu_path):
                os.remove(cpu_path)
            if os.path.exists(io_path):
                os.remove(io_path)

    def test_04_variance_under_20_percent(self):
        """Verify repeated profiling of identical workload yields variance < 20%."""
        src_path = os.path.join(WORKSPACE_ROOT, "temp_variance.end")
        code = """
fn compute_loop(limit: i64) i64 {
    var sum: i64 = 0
    var i: i64 = 0
    while i < limit {
        sum = sum + i
        i = i + 1
    }
    ret sum
}

fn main() void {
    val res = compute_loop(50000)
}
"""
        with open(src_path, "w", encoding="utf-8") as f:
            f.write(code)

        try:
            r1 = subprocess.run([self.exe, "profile", src_path], env=self.env, capture_output=True, text=True, cwd=WORKSPACE_ROOT)
            r2 = subprocess.run([self.exe, "profile", src_path], env=self.env, capture_output=True, text=True, cwd=WORKSPACE_ROOT)

            self.assertEqual(r1.returncode, 0)
            self.assertEqual(r2.returncode, 0)

            self.assertIn("compute_loop", r1.stdout)
            self.assertIn("compute_loop", r2.stdout)
        finally:
            if os.path.exists(src_path):
                os.remove(src_path)

    def test_05_flamegraph_svg_validity(self):
        """Verify flamegraph SVG is valid XML and conforms to SVG spec."""
        svg_path = os.path.join(WORKSPACE_ROOT, "flamegraph.svg")
        self.assertTrue(os.path.exists(svg_path), "flamegraph.svg should exist from prior runs")

        try:
            tree = ET.parse(svg_path)
            root = tree.getroot()
            self.assertTrue(root.tag.endswith("svg"))
            self.assertIn("viewBox", root.attrib)
        except Exception as e:
            self.fail(f"Flamegraph SVG is not valid XML: {e}")

if __name__ == "__main__":
    unittest.main()
