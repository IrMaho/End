#!/usr/bin/env python3
"""
Real Distributed Raft Consensus Engine Test Suite (Prompt 22 / Feature F-30)
Tests:
1. Leader Election (< 5 seconds)
2. Log Replication (100 entries committed across nodes < 1 second)
3. Persistent Log Storage backed by SQLite across restarts
4. Follower catch-up after recovery
5. Leader failure recovery & re-election (< 10 seconds)
6. Network partition: minority rejects writes, majority commits, healing restores convergence
7. Consistent reads across nodes
8. End language standard library integration (std/cluster/raft.end)
"""

import subprocess
import sys
import time
import os
import unittest

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
BIN_END = os.path.join(REPO_ROOT, "bin", "end.exe")
ENDC_DIR = os.path.join(REPO_ROOT, "endc")

class TestRaftConsensusEngine(unittest.TestCase):

    def test_01_rust_native_raft_engine_integration_suite(self):
        """Execute 8 comprehensive integration tests inside the Rust Raft runtime."""
        print("\n[Raft Test 01] Running Rust Native Raft Integration Test Suite...")
        cmd = [
            "cargo", "test", "--release", "--package", "endc",
            "--lib", "runtime::raft::raft_tests",
            "--", "--nocapture", "--test-threads=1"
        ]
        env = os.environ.copy()
        env["PATH"] = r"C:\Program Files\LLVM\bin;c:\Users\ASUS\Desktop\flutter_project\end\endc\z3\bin;c:\Users\ASUS\Desktop\flutter_project\end\pgsql\bin;" + env.get("PATH", "")

        start_time = time.time()
        res = subprocess.run(cmd, cwd=ENDC_DIR, capture_output=True, text=True, env=env)
        elapsed = time.time() - start_time

        print(res.stdout)
        if res.returncode != 0:
            print(res.stderr, file=sys.stderr)
        
        self.assertEqual(res.returncode, 0, f"Cargo test failed with code {res.returncode}")
        self.assertIn("test result: ok. 8 passed", res.stdout)
        print(f"[Raft Test 01] Passed in {elapsed:.2f} seconds.")

    def test_02_end_language_raft_cluster_demo(self):
        """Run examples/raft_cluster_demo.end and verify stdout contracts."""
        print("\n[Raft Test 02] Running End Language Raft Cluster Showcase...")
        demo_script = os.path.join(REPO_ROOT, "examples", "raft_cluster_demo.end")
        cmd = [BIN_END, "run", demo_script]
        
        env = os.environ.copy()
        env["PATH"] = r"C:\Program Files\LLVM\bin;c:\Users\ASUS\Desktop\flutter_project\end\endc\z3\bin;c:\Users\ASUS\Desktop\flutter_project\end\pgsql\bin;" + env.get("PATH", "")

        res = subprocess.run(cmd, cwd=REPO_ROOT, capture_output=True, text=True, env=env)
        print(res.stdout)
        if res.returncode != 0:
            print(res.stderr, file=sys.stderr)

        self.assertEqual(res.returncode, 0, f"Demo script failed with code {res.returncode}")
        self.assertIn("End Language Real Distributed Raft Consensus Engine Showcase", res.stdout)
        self.assertIn("Initial Cluster Leader Elected Node ID:", res.stdout)
        self.assertIn("OK:1", res.stdout)
        self.assertIn("OK:2", res.stdout)
        self.assertIn("OK:3", res.stdout)
        self.assertIn("New Leader Elected After Failover:", res.stdout)
        self.assertIn("OK:4", res.stdout)
        self.assertIn("All Raft Distributed Consensus Operations Completed Successfully!", res.stdout)

    def test_03_end_script_custom_cluster_consensus(self):
        """Run inline End code creating a custom Raft cluster with writes and reads."""
        print("\n[Raft Test 03] Verifying End cluster script with consensus operations...")
        temp_script = os.path.join(REPO_ROOT, "scratch_raft_test.end")
        code = '''
import "std/cluster/raft.end"

fn main() void {
    val cluster = raft_cluster_start(3, 24100, ":memory:")
    val leader = raft_cluster_get_leader(cluster)
    println("LEADER:")
    println(leader)
    
    val res1 = raft_cluster_write(cluster, "alpha", "val_alpha")
    println("WRITE1:")
    println(res1)
    val res2 = raft_cluster_write(cluster, "beta", "val_beta")
    println("WRITE2:")
    println(res2)

    val r_alpha = raft_cluster_read(cluster, "alpha")
    println("READ_ALPHA:")
    println(r_alpha)
    val r_beta = raft_cluster_read(cluster, "beta")
    println("READ_BETA:")
    println(r_beta)

    raft_cluster_stop(cluster)
    println("ALL_DONE")
}
'''
        with open(temp_script, "w", encoding="utf-8") as f:
            f.write(code)

        try:
            env = os.environ.copy()
            env["PATH"] = r"C:\Program Files\LLVM\bin;c:\Users\ASUS\Desktop\flutter_project\end\endc\z3\bin;c:\Users\ASUS\Desktop\flutter_project\end\pgsql\bin;" + env.get("PATH", "")
            res = subprocess.run([BIN_END, "run", temp_script], cwd=REPO_ROOT, capture_output=True, text=True, env=env)
            print(res.stdout)
            if res.returncode != 0:
                print(res.stderr, file=sys.stderr)
            self.assertEqual(res.returncode, 0)
            self.assertIn("OK:1", res.stdout)
            self.assertIn("OK:2", res.stdout)
            self.assertIn("val_alpha", res.stdout)
            self.assertIn("val_beta", res.stdout)
            self.assertIn("ALL_DONE", res.stdout)
        finally:
            if os.path.exists(temp_script):
                os.remove(temp_script)

if __name__ == "__main__":
    unittest.main(verbosity=2)
