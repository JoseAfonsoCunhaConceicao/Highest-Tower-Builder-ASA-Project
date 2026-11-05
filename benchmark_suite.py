#!/usr/bin/env python3
import os
import subprocess
import time
import random
import sys

# C signed 32-bit int max is 2,147,483,647.
# To ensure the total tower height h ALWAYS fits comfortably within a standard C int
# even if up to 2*N orientations are stacked, we scale max_dim dynamically based on N.
C_INT_MAX = 2_147_483_647

def generate_stress_input(filename, n):
    """
    Generates a large test file where total stacked height h is guaranteed
    to fit within a 32-bit signed integer (<= 2,147,483,647).
    """
    # Force regeneration so old overflowing files are replaced
    print(f"[*] Generating {filename} with N = {n} blocks (total h <= {C_INT_MAX})...")
    
    # In the worst case, a tower can use up to ~2*n orientations.
    # By setting the maximum dimension of any single block to C_INT_MAX // (2 * n),
    # the sum of heights is strictly guaranteed to be <= C_INT_MAX.
    max_dim = max(1000, C_INT_MAX // (2 * n + 2))
    step = max(1, (max_dim - 500) // max(1, n))

    with open(filename, "w") as f:
        f.write(f"{n}\n")
        for i in range(1, n + 1):
            base = i * step
            a = base + random.randint(1, 50)
            b = base + random.randint(51, 100)
            c = base + random.randint(101, 150)
            f.write(f"{a} {b} {c}\n")
    print(f"[+] Created {filename} successfully (max block dimension ~ {max_dim}).")

def run_benchmark(executable_cmd, input_file):
    print(f"[*] Running benchmark on {input_file}...")
    start_time = time.perf_counter()
    with open(input_file, "r") as infile:
        res = subprocess.run(executable_cmd, stdin=infile, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=True)
    duration = time.perf_counter() - start_time
    
    if res.returncode != 0:
        print(f"[-] Execution failed with code {res.returncode}:")
        print(res.stderr.decode())
        return None
    
    lines = res.stdout.decode().strip().splitlines()
    header = lines[0] if lines else "No output"
    print(f"[+] Output header (h m): {header}")
    print(f"[+] Total output lines: {len(lines)}")
    print(f"[+] Time elapsed: {duration:.4f} seconds ({duration * 1000:.2f} ms)\n")
    return duration

if __name__ == "__main__":
    test_cases = [
        ("test_small_100.in", 100),
        ("test_medium_1000.in", 1_000),
        ("test_stress_5000.in", 5_000),
        ("test_max_10000.in", 10_000),
    ]

    for fname, n in test_cases:
        generate_stress_input(fname, n)

    print("\n=== BENCHMARK SUITE READY ===")
    print("Executing tests...\n")

    bin_name = "highest-tower-builder"
    possible_bins = [
        f"./target/release/{bin_name}",
        "./target/release/box_stacking",
    ]
    cmd = "cargo run --release"
    for b in possible_bins:
        if os.path.exists(b):
            cmd = b
            break

    print(f"Using command: {cmd}\n")
    for fname, _ in test_cases:
        run_benchmark(cmd, fname)
