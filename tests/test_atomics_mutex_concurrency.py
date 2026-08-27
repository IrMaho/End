#!/usr/bin/env python3
"""
Test Suite: End Language Prompt 21 — Real C11 Atomics & Mutex Concurrency
Validates:
1. ISO C11 <stdatomic.h> primitives lowering and emission
2. Memory order mappings (Relaxed, Acquire, Release, AcqRel, SeqCst)
3. Native OS Mutex (CriticalSection / pthread_mutex) and SRWLOCK / pthread_rwlock
4. Multi-threaded stress tests, CAS Treiber lock-free operations, and bounded deadlock timeouts
5. Native compilation via GCC / Clang backend
"""

import os
import sys
import subprocess
import time

def run_cmd(cmd, cwd=None):
    env = os.environ.copy()
    env["PATH"] = r"C:\Program Files\LLVM\bin;c:\Users\ASUS\Desktop\flutter_project\end\endc\z3\bin;c:\Users\ASUS\Desktop\flutter_project\end\pgsql\bin;" + env.get("PATH", "")
    print(f"[*] Running: {cmd}")
    res = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=cwd, env=env)
    return res

def test_rust_sync_unit_suite():
    print("\n--- Test 1: Rust Native Sync Unit & Stress Suite ---")
    res = run_cmd("cargo test --release --package endc --lib runtime::sync::tests -- --nocapture", cwd=r"c:\Users\ASUS\Desktop\flutter_project\end\endc")
    print(res.stdout)
    if res.returncode != 0:
        print(res.stderr)
        return False
    assert "test runtime::sync::tests::test_atomic_operations_matrix ... ok" in res.stdout
    assert "test runtime::sync::tests::test_atomic_multithreaded_producer_consumer_stress ... ok" in res.stdout
    assert "test runtime::sync::tests::test_mutex_contention_and_mutual_exclusion ... ok" in res.stdout
    assert "test runtime::sync::tests::test_atomic_cas_lock_free_counter ... ok" in res.stdout
    assert "test runtime::sync::tests::test_rwlock_multiple_readers_single_writer ... ok" in res.stdout
    assert "test runtime::sync::tests::test_deadlock_detection_with_timeout ... ok" in res.stdout
    print("[+] Test 1 Passed: 6/6 Concurrency and Synchronization Tests Passed.")
    return True

def test_end_script_execution():
    print("\n--- Test 2: End Language Interpreter Concurrency Run ---")
    res = run_cmd(r'"c:\Users\ASUS\Desktop\flutter_project\end\bin\end.exe" run examples\concurrency_atomics_demo.end', cwd=r"c:\Users\ASUS\Desktop\flutter_project\end")
    print(res.stdout)
    if res.returncode != 0:
        print(res.stderr)
        return False
    assert "All Concurrency, Atomics & Mutex Verifications Succeeded!" in res.stdout
    print("[+] Test 2 Passed: End Language Interpreter execution succeeded.")
    return True

def test_native_c11_compilation():
    print("\n--- Test 3: Native C11 Compiler Backend Build & Run ---")
    bin_path = r"c:\Users\ASUS\Desktop\flutter_project\end\test_concurrency_native.exe"
    if os.path.exists(bin_path):
        os.remove(bin_path)

    res = run_cmd(f'"c:\\Users\\ASUS\\Desktop\\flutter_project\\end\\bin\\end.exe" build examples\\concurrency_atomics_demo.end -o {bin_path}', cwd=r"c:\Users\ASUS\Desktop\flutter_project\end")
    print(res.stdout)
    if res.returncode != 0:
        print(res.stderr)
        return False
    
    assert os.path.exists(bin_path), f"Expected binary {bin_path} to exist"
    
    # Run the native binary
    res_bin = run_cmd(bin_path, cwd=r"c:\Users\ASUS\Desktop\flutter_project\end")
    print(res_bin.stdout)
    if res_bin.returncode != 0:
        print(res_bin.stderr)
        return False
    assert "All Concurrency, Atomics & Mutex Verifications Succeeded!" in res_bin.stdout
    print("[+] Test 3 Passed: Native C11 Binary Execution succeeded.")
    return True

def test_c11_stdatomic_emission():
    print("\n--- Test 4: Verification of C11 <stdatomic.h> and Primitive Emission ---")
    c_source = r"c:\Users\ASUS\Desktop\flutter_project\end\examples\concurrency_atomics_demo.c"
    assert os.path.exists(c_source), "Generated C file does not exist"
    with open(c_source, "r", encoding="utf-8") as f:
        content = f.read()

    assert "#include <stdatomic.h>" in content, "Missing #include <stdatomic.h>"
    assert "atomic_load_explicit" in content, "Missing atomic_load_explicit"
    assert "atomic_store_explicit" in content, "Missing atomic_store_explicit"
    assert "atomic_fetch_add_explicit" in content, "Missing atomic_fetch_add_explicit"
    assert "atomic_fetch_sub_explicit" in content, "Missing atomic_fetch_sub_explicit"
    assert "atomic_compare_exchange_strong_explicit" in content, "Missing atomic_compare_exchange_strong_explicit"
    assert "memory_order_seq_cst" in content, "Missing memory_order_seq_cst"
    assert "end_mutex_create" in content, "Missing end_mutex_create"
    assert "end_rwlock_create" in content, "Missing end_rwlock_create"
    print("[+] Test 4 Passed: Generated C source includes true ISO C11 stdatomic.h primitives.")
    return True

if __name__ == "__main__":
    t0 = time.time()
    tests = [
        test_rust_sync_unit_suite,
        test_end_script_execution,
        test_native_c11_compilation,
        test_c11_stdatomic_emission,
    ]
    for t in tests:
        if not t():
            print(f"[-] FAILED: {t.__name__}")
            sys.exit(1)
    
    print(f"\n[+] ALL 4 PROMPT 21 CONCURRENCY VERIFICATIONS PASSED in {time.time()-t0:.2f}s!")
    sys.exit(0)
