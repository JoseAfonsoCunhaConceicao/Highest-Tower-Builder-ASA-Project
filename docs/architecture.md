# Architecture & Algorithmic Design — 3D Box Stacking

High-performance, memory-safe, and deterministic solution for the **3D Box Stacking Problem**, built in compliance with NASA JPL's "Power of Ten" safety-critical engineering guidelines.

---

## 1. Problem Formulation and Invariants

Given $N$ types of rectangular 3D blocks with dimensions $(d_1, d_2, d_3)$ available in unlimited supply:

* **Objective:** Find an ordered sequence of blocks that maximizes total vertical height $\sum h_i$.
* **Strict Stability Constraint:** A block $A = (w_a, l_a, h_a)$ can be stacked on top of block $B = (w_b, l_b, h_b)$ if and only if both base dimensions are strictly smaller:
  $$w_a < w_b \quad \land \quad l_a < l_b$$
* **Canonical Normalization Invariant:** For every rotation, we enforce $w \le l$. This reduces the 6 spatial orientations to at most 3 non-equivalent canonical rotations per block type:
  1. Base $(d_1, d_2)$ with height $d_3$
  2. Base $(d_1, d_3)$ with height $d_2$ (if $d_2 \neq d_3$)
  3. Base $(d_2, d_3)$ with height $d_1$ (if $d_1 \neq d_2$)

---

## 2. Algorithmic Pipeline: $\mathcal{O}(M \log M)$

While the textbook dynamic programming solution models box stacking as the Longest Path in a Directed Acyclic Graph (DAG) in $\mathcal{O}(N^2)$ time, this implementation achieves optimal **$\mathcal{O}(M \log M)$** time and **$\mathcal{O}(M)$** space (where $M \le 3N$ is the number of valid orientations) via coordinate compression and a 1D range-maximum Fenwick Tree.

```
+--------------------+      +--------------------+      +--------------------------+
|  Input Ingestion   | ---> | Orientation Gen.   | ---> |  Topological Sort        |
|  & Zero-Copy Parse |      | & Deduplication    |      |  (w ASC, l DESC, h DESC) |
+--------------------+      +--------------------+      +--------------------------+
                                                                     |
                                                                     v
+--------------------+      +--------------------+      +--------------------------+
|  Buffered Output   | <--- | Backtracking &     | <--- |  Fenwick Tree Sweep-Line |
|  Decompression     |      | Path Recovery      |      |  (Point Upd, Prefix Max) |
+--------------------+      +--------------------+      +--------------------------+
```

### Step 1: Normalization & Generation — $\mathcal{O}(N)$
Each input triplet $(d_1, d_2, d_3)$ is sorted using a 3-element comparison network ($d_1 \le d_2 \le d_3$). Up to 3 unique canonical rotations with $w \le l$ are pushed into a single pre-allocated contiguous vector.

### Step 2: Base Deduplication (Dominance Pruning) — $\mathcal{O}(M)$
Consecutive orientations sharing identical base dimensions $(w, l)$ are filtered: only the orientation with the maximum height $h$ is retained, pruning strictly dominated choices without sacrificing optimality.

### Step 3: Coordinate Compression — $\mathcal{O}(M \log M)$
The set of unique length values $\{ l_i \}$ is extracted, deduplicated, and sorted in ascending order. Each block's metric length $l$ is replaced by its 1-based compressed rank `idx` in the unique array, allowing direct indexing into the Fenwick Tree.

### Step 4: Topological Sweep-Line with Fenwick Tree — $\mathcal{O}(M \log M)$
Blocks are sorted by:
1. $w$ **ascending** (processes smaller widths first)
2. $l$ **descending** (tie-breaker: prevents blocks with the same $w$ from stacking onto each other)
3. $h$ **descending** (tie-breaker for equal dimensions)

A Binary Indexed Tree (Fenwick Tree) maintains the maximum accumulated tower height for any compressed length prefix $[1, l_{\text{compressed}} - 1]$. For each block:
* **Query:** Query prefix `1..=b.l - 1` to find the tallest compatible sub-tower $(\text{best\_h}, \text{best\_parent})$.
* **Update:** Update position `b.l` with $\text{total\_h} = \text{best\_h} + b.h$ and the current block index.

### Step 5: Reconstruction & Decompression — $\mathcal{O}(M)$
Using an explicit parent pointer vector, the optimal sequence is backtracked from the global maximum index down to `NO_BLOCK`. The original metric dimensions are restored from the coordinate compression table and written via a 64 KB buffered writer.

---

## 3. Low-Level Performance & Cache Optimizations

* **Cache Density:** Struct fields use compact `u32` (4 bytes) rather than `usize` (8 bytes), fitting multiple [`Block`](file:///Users/joseconceicao/Documents/Ensino/Universidade/Ano2/ASA/Projeto/highest-tower-builder/src/main.rs#L28-L37) instances within a single 64-byte L1 CPU cache line.
* **Zero-Copy Bulk I/O:** Standard input is ingested into a contiguous byte buffer using a single `read_to_end()` syscall, parsed by an arithmetic, zero-allocation integer parser (`next_u32`).
* **Direct Output Formatting:** Standard output uses `BufWriter::with_capacity(65_536)` to amortize context switching and syscall overhead across thousands of lines.

---

## 4. NASA JPL "Power of Ten" Compliance

The codebase strictly adheres to the safety-critical programming rules adapted for systems-level Rust:

* [x] **Rule 1 (Simple Control Flow):** Zero recursion. Iterative traversal and sweep-line only (`#![deny(unconditional_recursion)]`).
* [x] **Rule 2 (Fixed Loop Bounds):** All loops are strictly bounded by $N$ or $M \le 3N$. No dynamic loops with non-terminating invariants.
* [x] **Rule 3 (No Dynamic Allocation After Initialization):** All buffers (`blocks`, `parents`, `fenwick`) are pre-allocated with exact capacities. Zero allocations in the core DP sweep engine.
* [x] **Rule 4 (Short Functions):** All functions are single-purpose, modular, and stay well within the 60-line limit.
* [x] **Rule 5 & 7 (Defensive Programming & Explicit Errors):** Absolute prohibition of panics (`#![deny(clippy::unwrap_used, clippy::expect_used)]`). Checked arithmetic operations prevent integer overflow.
* [x] **Rule 8 (Forbid Unsafe Code):** Absolute memory safety enforced at compile-time (`#![forbid(unsafe_code)]`).
* [x] **Rule 10 (Pedantic Static Analysis):** Compiled under strict compiler and linter enforcement (`#![deny(warnings, clippy::all, clippy::pedantic)]`).
