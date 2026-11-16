//! # 3D Box Stacking Solver (ASA Project)
//! 
//! High-performance, memory-safe solution for the 3D Box Stacking problem.
//! Built in compliance with NASA JPL's "Power of Ten" safety-critical rules.

// 1. Native `rustc` compiler lints
#![deny(unconditional_recursion)]       // NASA Rule 1: Avoid complex flow constructs (forbid direct/indirect unbounded recursion)
#![forbid(unsafe_code)]                 // NASA Rule 3: Do not rely on dynamic features/memory bugs (enforce absolute memory safety)
#![deny(warnings)]                      // NASA Rule 10: All warnings must be treated as errors and resolved before flight
#![deny(missing_docs)]                  // NASA Rule 10: Code clarity and documentation on all accessible structures

// 2. Static analysis `clippy` lints
#![deny(
     clippy::all,                       // NASA Rule 10: Enable all standard correctness, performance, and correctness checks
     clippy::pedantic,                  // NASA Rule 10: Strict pedantic static analysis (stricter than default standards)
     clippy::unwrap_used,               // NASA Rule 7: Forbid `.unwrap()`, ensuring all error flows and returns are explicitly checked
     clippy::expect_used,               // NASA Rule 7: Forbid `.expect()`, preventing unexpected panic panics and crashes
     clippy::indexing_slicing           // NASA Rule 2: Give all loops/arrays fixed bounds; prevent unchecked raw slices or `v[i]`
)]

use std::cmp::Ordering;
use std::io::{self, BufWriter, Read, Write};

/// Sentinel value indicating the absence of a parent block index (equivalent to `None`).
const NO_BLOCK: u32 = u32::MAX;

/// Represents a 3D block in a canonical orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Block {
     /// Width of the base (enforced invariant: `w <= l`).
     w: u32,
     /// Length of the base (or compressed rank during the dynamic programming phase).
     l: u32,
     /// Vertical height of the block in this orientation.
     h: u32,
}

impl Block {
     /// Creates a new canonical [`Block`] with given dimensions.
     #[must_use]
     #[inline]
     const fn new(w: u32, l: u32, h: u32) -> Self {
          Self { w, l, h }
     }

     /// Compares two blocks in topological sweep-line order.
     ///
     /// Sorting Rationale:
     /// 1. `w` ASC: processes smaller base widths first.
     /// 2. `l` DESC: for identical widths `w`, processes larger lengths first.
     ///    This prevents boxes with the same width and different lengths from illegally
     ///    stacking on top of each other inside the Fenwick Tree.
     /// 3. `h` DESC: tie-breaker by descending height.
     #[must_use]
     #[inline]
     fn cmp_topological(&self, other: &Self) -> Ordering {
          self.w
              .cmp(&other.w)
              .then_with(|| other.l.cmp(&self.l))
              .then_with(|| other.h.cmp(&self.h))
     }
}

/// A cache-optimized Fenwick Tree (Binary Indexed Tree) using Structure of Arrays (`SoA`).
struct FenwickTree {
     /// Stores maximum prefix heights per coordinate rank.
     heights: Vec<u32>,
     /// Stores the 32-bit index of the top block producing the corresponding height.
     blocks: Vec<u32>,
}

impl FenwickTree {
     /// Creates an empty [`FenwickTree`] sized for `size` compressed coordinates.
     #[must_use]
     fn new(size: usize) -> Self {
          let capacity = size.saturating_add(1);
          Self {
               heights: vec![0; capacity],
               blocks: vec![NO_BLOCK; capacity],
          }
     }

     /// Queries the maximum accumulated height and corresponding top block in the prefix `1..=idx`.
     #[must_use]
     #[inline]
     fn query(&self, mut idx: usize) -> (u32, u32) {
          let mut best_h = 0;
          let mut best_b = NO_BLOCK;

          while idx > 0 {
               if let Some(&h) = self.heights.get(idx) {
                    if h > best_h {
                         best_h = h;
                         if let Some(&b) = self.blocks.get(idx) {
                              best_b = b;
                         }
                    }
               } else {
                    break;
               }

               // Clears the least significant bit (LSB) in a single native instruction (e.g. BLSR on x86_64).
               // Equivalent to: idx -= idx & (!idx + 1), but without signed overflow.
               idx &= idx.wrapping_sub(1);
          }

          (best_h, best_b)
     }

     /// Updates the Fenwick Tree at position `idx` if `height` strictly exceeds the stored value.
     #[inline]
     fn update(&mut self, mut idx: usize, height: u32, block_idx: u32) {
          let len = self.heights.len();
          while idx < len {
               // Inspects first via a shared read to avoid heap writes
               // if the new height does not exceed the node's current best.
               let needs_update = self.heights.get(idx).is_some_and(|&h| height > h);
               if needs_update {
                    if let Some(h) = self.heights.get_mut(idx) {
                         *h = height;
                    }
                    if let Some(b) = self.blocks.get_mut(idx) {
                         *b = block_idx;
                    }
               }

               // Adds the LSB to propagate the update to ancestor nodes in the tree:
               // idx += idx & (-idx) safely implemented without signed integer conversion.
               idx = idx.wrapping_add(idx & (!idx).wrapping_add(1));
          }
     }
}

/// Extracts the next positive integer `u32` from a raw byte buffer.
///
/// Implements linear-time parsing with zero intermediate allocations.
#[must_use]
#[inline]
fn next_u32(buffer: &[u8], cursor: &mut usize) -> Option<u32> {
     // Consumes any leading whitespace characters or line breaks
     while let Some(&byte) = buffer.get(*cursor) {
          if byte.is_ascii_whitespace() {
               *cursor += 1;
          } else {
               break;
          }
     }

     if *cursor >= buffer.len() {
          return None;
     }

     // Accumulates digits with strict arithmetic checks against overflow
     let mut value: u32 = 0;
     while let Some(&byte) = buffer.get(*cursor) {
          if byte.is_ascii_digit() {
               value = value
                    .checked_mul(10)?
                    .checked_add(u32::from(byte - b'0'))?;
               *cursor += 1;
          } else {
               break;
          }
     }

     Some(value)
}

/// Normalizes dimensions and pushes up to 3 non-duplicate orientations directly into the target buffer.
#[inline]
fn push_orientations(target: &mut Vec<Block>, mut d1: u32, mut d2: u32, mut d3: u32) {
     // 3-element sorting network ensuring d1 <= d2 <= d3 in just 3 comparisons.
     // This guarantees the canonical invariant w <= l without requiring subsequent checks.
     if d1 > d2 { std::mem::swap(&mut d1, &mut d2); }
     if d2 > d3 { std::mem::swap(&mut d2, &mut d3); }
     if d1 > d2 { std::mem::swap(&mut d1, &mut d2); }

     // 1. Orientation using d1 and d2 as the base, and d3 as the height
     target.push(Block::new(d1, d2, d3));

     // 2. If d2 != d3, rotating to place d3 on the base yields a distinct orientation
     if d2 != d3 {
          target.push(Block::new(d1, d3, d2));
     }

     // 3. If d1 != d2, using d2 and d3 for the base yields another non-equivalent rotation
     if d1 != d2 {
          target.push(Block::new(d2, d3, d1));
     }
}

/// Ingests data from standard input and prepares canonical sorted blocks with compressed coordinates.
#[must_use]
#[inline]
fn prepare_blocks() -> Option<(Vec<Block>, Vec<u32>)> {
     let mut buffer = Vec::new();
     let mut cursor: usize = 0;

     // Locks the input stream for bulk reading without repeated locking overhead
     io::stdin().lock().read_to_end(&mut buffer).ok()?;

     let n = next_u32(&buffer, &mut cursor)?;
     let n_usize = usize::try_from(n).ok()?;
     let max_orientations = n_usize.checked_mul(3)?;

     let mut blocks: Vec<Block> = Vec::with_capacity(max_orientations);

     for _ in 0..n_usize {
          let s1 = next_u32(&buffer, &mut cursor)?;
          let s2 = next_u32(&buffer, &mut cursor)?;
          let s3 = next_u32(&buffer, &mut cursor)?;
          push_orientations(&mut blocks, s1, s2, s3);
     }

     // Collects and sorts distinct L values for coordinate compression (offline ranking)
     let mut unique_l: Vec<u32> = blocks.iter().map(|b| b.l).collect();
     unique_l.sort_unstable();
     unique_l.dedup();
     
     // Directly remaps each 'l' to its index [0..unique_l.len()-1].
     // This completely eliminates calls to `binary_search` during the main DP loop.
     for b in &mut blocks {
          let Ok(idx) = unique_l.binary_search(&b.l) else { return None; };
          b.l = u32::try_from(idx).ok()?;
     }

     // Topologically sorts to enable a sweep-line scan
     blocks.sort_unstable_by(Block::cmp_topological);
     // Removes boxes with strictly identical bases and orientations
     blocks.dedup_by(|a, b| a.w == b.w && a.l == b.l);

     Some((blocks, unique_l))
}

/// Solves the 3D box stacking dynamic programming sequence using coordinate sweep-line.
#[must_use]
#[inline]
fn solve(blocks: &[Block], l_domain_size: usize) -> Option<(Vec<usize>, u32)> {
     if blocks.is_empty() {
          return Some((Vec::new(), 0));
     }

     let mut tree = FenwickTree::new(l_domain_size);
     // Predecessor array for path reconstruction at the end (4 bytes per block)
     let mut parents: Vec<u32> = vec![NO_BLOCK; blocks.len()];

     let mut max_h: u32 = 0;
     let mut best_idx: u32 = NO_BLOCK;

     for (idx, b) in blocks.iter().enumerate() {
          let idx_u32 = u32::try_from(idx).ok()?;
          let l_rank = usize::try_from(b.l).ok()?;

          // Queries the tree for the best tower whose base is strictly smaller (1..=l_rank)
          let (base_h, parent) = tree.query(l_rank);
          let total_h = base_h.checked_add(b.h)?;

          // Updates position (l_rank + 1) in the Fenwick Tree (1-based indexing)
          tree.update(l_rank.checked_add(1)?, total_h, idx_u32);

          if let Some(p) = parents.get_mut(idx) {
               *p = parent;
          }

          if total_h > max_h {
               max_h = total_h;
               best_idx = idx_u32;
          }
     }

     // Backtracks to reconstruct the optimal block stack
     let mut path = Vec::new();
     let mut curr = best_idx;
     while curr != NO_BLOCK {
          let curr_usize = usize::try_from(curr).ok()?;
          path.push(curr_usize);
          curr = parents.get(curr_usize).copied()?;
     }

     Some((path, max_h))
}

/// Formats and writes the reconstructed tower to standard output with decompressed dimensions.
#[inline]
fn write_result(blocks: &[Block], unique_l: &[u32], path: &[usize], total_h: u32) {
     let stdout = io::stdout();
     // Generous 64 KB buffer to minimize context switches across I/O syscalls
     let mut writer = BufWriter::with_capacity(65_536, stdout.lock());

     if writeln!(writer, "{total_h} {}", path.len()).is_err() {
          return;
     }

     for &idx in path {
          let Some(b) = blocks.get(idx) else { return; };
          let Ok(l_idx) = usize::try_from(b.l) else { return; };
          // Retrieves the actual metric value of L from the compressed rank
          let Some(&actual_l) = unique_l.get(l_idx) else { return; };

          if writeln!(writer, "{} {} {}", b.w, actual_l, b.h).is_err() {
               return;
          }
     }

     let _ = writer.flush();
}

/// Entry point of the solver.
fn main() {
     let Some((blocks, unique_l)) = prepare_blocks() else {
          println!("0");
          return;
     };

     if let Some((path, total_h)) = solve(&blocks, unique_l.len()) {
          write_result(&blocks, &unique_l, &path, total_h);
     } else {
          println!("0");
     }
}