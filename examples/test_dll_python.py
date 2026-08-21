# Python Application calling End Language Compiled DLL via ctypes FFI
import ctypes
import os
import sys

dll_path = os.path.abspath("mathlib.dll")
if not os.path.exists(dll_path):
    print(f"Error: {dll_path} not found.")
    sys.exit(1)

print("=== Python FFI Calling End Language Compiled DLL ===")
mathlib = ctypes.CDLL(dll_path)

# Configure function prototypes
mathlib.end_add.argtypes = [ctypes.c_int64, ctypes.c_int64]
mathlib.end_add.restype = ctypes.c_int64

mathlib.end_multiply.argtypes = [ctypes.c_int64, ctypes.c_int64]
mathlib.end_multiply.restype = ctypes.c_int64

mathlib.end_compute_hash.argtypes = [ctypes.c_uint64, ctypes.c_int32]
mathlib.end_compute_hash.restype = ctypes.c_int64

mathlib.end_process_batch.argtypes = [ctypes.c_int32]
mathlib.end_process_batch.restype = ctypes.c_int64

# Execute calls
res_add = mathlib.end_add(1500, 2500)
print(f"1. Python -> end_add(1500, 2500): {res_add}")

res_mul = mathlib.end_multiply(25, 4)
print(f"2. Python -> end_multiply(25, 4): {res_mul}")

res_hash = mathlib.end_compute_hash(999, 16)
print(f"3. Python -> end_compute_hash(999, 16): {res_hash}")

res_batch = mathlib.end_process_batch(50000)
print(f"4. Python -> end_process_batch(50000): {res_batch}")

print("=== SUCCESS: Python seamlessly executed native End DLL functions! ===")
