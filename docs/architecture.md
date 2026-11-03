# Architecture & Design — ASA Project (3D Box Stacking)

## 1. Problem Formulation and Invariants

- **Objective:** Find a sequence of 3D blocks $(w_i, l_i, h_i)$, that maximizes total height.

- **Stability Constraint:** A block $(w_a, l_a, h_a)$ can be placed on top of $(w_b, l_b, h_b)$ only if $w_a < w_b \land l_a < l_b$.

- **Normalization Invariant:** Without loss of generality, every generated orientation enforces $w \le l$. This reduces the 6 spatial rotations to at most 3 canonical orientations per block.

## 2. Algorithmic Pipeline

1. **Generation and Normalization:** For each of the $N$ input block types, generate up to 2 canonical rotations ($w \le l$) - $\mathcal{O}(N)$.

2. **Implicit Topologic Sort:** Sort orientation by ascending $w$, ascending $l$ and descending $h$ — $\mathcal{O}(N \log N)$.

3. **Dominance Pruning:** Remove consecutive blocks sharing the exact same base $(w, l)$, keeping only the first (whitch is garanteed to have the maximum height $h$) - $\mathcal{O}(N)$.

4. **Dynamic Programming (Longest Path in DAG):** Compute the tallets sub-tower routed at block $i$ — $\mathcal{O}(n^2)$ time, $\mathcal{O}(N) space$.

## 3. Memory Architecture & Low-Level Optimizations (Rust)

- **Data Types:** Strict use of `u32` (4 bytes) instead of `usize` (8 bytes) as problem bounds fit within 32-bit integers, doubling L1/L2 cache line density.

- **In-Place Reconstruction:** Sorting from top-to-bottom (smallest to largest base) allows printing the final tower from bottom-to-top by following `next` indices directly, avoiding extra allocations to reverse the path.

## 4. NASA JPL "Power of Ten" Compliance

- [ ] **Rule 1 (Simple Control Flow):** Zero recursion (`#![deny(unconditional_recursion)]`).

- [ ] **Rule 2 (Fixed Loop Bounds):** All loops bounded by $N$ or $M \le 3N$.

- [ ] **Rule 3 (No Dynamic Allocation After Init):** Single allocation via `Vec::with_capacity(3 * n)` during parsing; zero allocations in the DP engine.

- [ ] **Rule 4 (Short Functions):** Max 60 lines per function (`parse_input`, `prepare_blocks`, `solve_dp`, `write_output`).

- [ ] **Rules 5 & 7 (Assertions & Error Handling):** `debug_assert!` for invariants ($w \le l$) and `#![deny(clippy::unwrap_used)]`.

- [ ] **Rule 10 (Pedantic Compilation):** `#![deny(warnings, clippy::all, clippy::pedantic)]`.
