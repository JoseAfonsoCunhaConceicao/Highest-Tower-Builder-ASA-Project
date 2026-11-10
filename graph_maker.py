#!/usr/bin/env python3
"""
ASA 2026/2027 - Benchmark and Plotting Suite for 3D Box Stacking
This script automates:
  1. Generating test instances with varying N
  2. Compiling the Rust binary with release optimizations
  3. Measuring execution times across multiple runs (mean + std dev)
  4. Generating comparison plots (Time vs N and Time vs N log N) matching the template.
"""

import os
import subprocess
import time
import random
import numpy as np
import matplotlib.pyplot as plt

# Configuration
RUST_SRC_DIR = "."
BIN_PATH = "./target/release/highest-tower-builder"
INSTANCES_DIR = "./benchmark_instances"
PLOTS_DIR = "./benchmark_plots"

# Problem sizes to test (at least 10 instances as requested)
INSTANCE_SIZES = [10_000, 20_000, 40_000, 60_000, 80_000, 100_000, 150_000, 200_000, 250_000, 300_000]
RUNS_PER_INSTANCE = 5
MAX_DIM = 1_000_000

def ensure_dirs():
    os.makedirs(INSTANCES_DIR, exist_ok=True)
    os.makedirs(PLOTS_DIR, exist_ok=True)

def compile_rust():
    print("[+] Compiling Rust binary in release mode...")
    cmd = ["cargo", "build", "--release"]
    res = subprocess.run(cmd, cwd=RUST_SRC_DIR, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"Compilation error:\n{res.stderr}")
        exit(1)
    print(f"[+] Binary ready: {BIN_PATH}")

def generate_instance(n: int, filename: str):
    print(f"[*] Generating instance {filename} with N = {n}...")
    with open(filename, "w") as f:
        f.write(f"{n}\n")
        # Generate random box dimensions
        for _ in range(n):
            d1 = random.randint(1, MAX_DIM)
            d2 = random.randint(1, MAX_DIM)
            d3 = random.randint(1, MAX_DIM)
            f.write(f"{d1} {d2} {d3}\n")

def run_benchmark(input_file: str) -> float:
    with open(input_file, "r") as inf:
        start_t = time.perf_counter()
        proc = subprocess.run(
            [BIN_PATH],
            stdin=inf,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
        end_t = time.perf_counter()
        if proc.returncode != 0:
            raise RuntimeError(f"Execution failed on {input_file}")
    return end_t - start_t

def main():
    ensure_dirs()
    compile_rust()

    means = []
    stds = []

    print("\n[+] Starting Benchmarks...")
    for n in INSTANCE_SIZES:
        inst_path = os.path.join(INSTANCES_DIR, f"inst_{n}.in")
        if not os.path.exists(inst_path):
            generate_instance(n, inst_path)

        times = []
        for r in range(RUNS_PER_INSTANCE):
            t = run_benchmark(inst_path)
            times.append(t)
            print(f"  N={n:7d} | Run {r+1}/{RUNS_PER_INSTANCE}: {t*1000:.2f} ms")

        mean_t = float(np.mean(times))
        std_t = float(np.std(times))
        means.append(mean_t)
        stds.append(std_t)
        print(f"  => Mean: {mean_t*1000:.2f} ms (+/- {std_t*1000:.2f} ms)\n")

    # Prepare Theoretical Metrics
    # Total canonical orientations processed: M <= 3N
    # Complexity: M log M
    n_arr = np.array(INSTANCE_SIZES, dtype=np.float64)
    m_arr = 3.0 * n_arr
    theoretical_axis = m_arr * np.log2(m_arr)

    # Plot 1: Time (s) vs N
    plt.figure(figsize=(7, 5))
    plt.errorbar(n_arr, means, yerr=stds, fmt='o-', color='#1f77b4',
                 ecolor='#6baed6', elinewidth=1.5, capsize=3, markersize=4, label='Experimental')
    plt.xlabel('# Caixas (N)', fontsize=11)
    plt.ylabel('Time (s)', fontsize=11)
    plt.title('Tempo de Execução vs. Número de Caixas (N)', fontsize=12)
    plt.grid(True, linestyle='--', alpha=0.5)
    plt.ylim(bottom=0)
    plt.tight_layout()
    plot1_path = os.path.join(PLOTS_DIR, "plot_time_vs_n.png")
    plt.savefig(plot1_path, dpi=300)
    plt.close()
    print(f"[+] Saved {plot1_path}")

    # Plot 2: Time (s) vs Theoretical Axis (M log M)
    plt.figure(figsize=(7, 5))
    plt.errorbar(theoretical_axis, means, yerr=stds, fmt='o-', color='#2ca02c',
                 ecolor='#98df8a', elinewidth=1.5, capsize=3, markersize=4, label='Experimental')

    # Linear regression fit for validation
    slope, intercept = np.polyfit(theoretical_axis, means, 1)
    fit_line = slope * theoretical_axis + intercept
    plt.plot(theoretical_axis, fit_line, '--', color='#d62728', label=f'Ajuste Linear (R²: {np.corrcoef(theoretical_axis, means)[0,1]**2:.4f})')

    plt.xlabel('Complexidade Teórica: $M \\log_2(M)$  ($M \\approx 3N$)', fontsize=11)
    plt.ylabel('Time (s)', fontsize=11)
    plt.title('Validação da Complexidade: Tempo vs. $M \\log_2(M)$', fontsize=12)
    plt.grid(True, linestyle='--', alpha=0.5)
    plt.legend()
    plt.ylim(bottom=0)
    plt.tight_layout()
    plot2_path = os.path.join(PLOTS_DIR, "plot_time_vs_theoretical.png")
    plt.savefig(plot2_path, dpi=300)
    plt.close()
    print(f"[+] Saved {plot2_path}")

    print("\n[+] Benchmark finished successfully. Ready for report embedding.")

if __name__ == "__main__":
    main()