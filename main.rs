//! # 3D Box Stacking Solver (ASA Project)
//! 
//! High-performance, memory-safe solution for the 3D Box Stacking problem.
//! Built in compliance with NASA JPL's "Power of Ten" safety-critical rules.

// 1. Native `rustc` lints (always enforced, including on Mooshak)
#![forbid(unsafe_code)]                           // Enforce 100% memory safety
#![deny(warnings)]                                // NASA Rule 10: treat all warnings as errors
#![deny(unconditional_recursion)]                 // NASA Rule 1: forbid direct recursion
#![deny(missing_docs)]                            // Enforce '///' documentation on all public items

// Static analysis `clippy` lints (enforced locally via `cargo clippy`)
#![deny(
     clippy::all,                                 // Standard correctness, style and performance lints
     clippy::pedantic,                            // NASA Rule 10: strict pedantic static analysis
     clippy::unwrap_used,                         // NASA Rule 7: forbid '.unwrap()', require explicit handling
     clippy::expect_used,                         // NASA Rule 7: forbid '.expect()', prevent unexpected panics
     clippy::indexing_slicing                     // Enforce bounds-checked access or iterators over raw 'v[i]'
)]

use std::cmp::Ordering;
use std::io::{self, BufWriter, Read, Write};

/// Represents a 3D block in a specific canonical orientation where `w <= l`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Block {
     /// Width of the base (always the smaller or equal base dimension: `w <= l`).
     w: u32,
     /// Length of the base (always the larger or equal base dimension: `l >= w`).
     l: u32,
     /// Vertical height of the block in this orientation.
     h: u32,
}

impl Block {
     /// Creates a new canonical [`Block`] orientation, enforcing the `w <= l` invariant.
     #[must_use]
     const fn new(side_a: u32, side_b: u32, height: u32) -> Self {
          debug_assert!(side_a > 0 && side_b > 0 && height > 0,
               "All block dimensions must be strictly positive."
          );
          
          let (w, l) = if side_a <= side_b {
               (side_a, side_b)
          } else {
               (side_b, side_a)
          };

          Self { w, l, h: height }
     }

     /// Generates the 3 canonical orientations for a raw [`Block`] of dimensions `(d1, d2, d3)`.
     #[must_use]
     const fn orientations(d1: u32, d2: u32, d3: u32) -> [Self; 3] {
          [
               Self::new(d1, d2, d3),
               Self::new(d3, d1, d2),
               Self::new(d3, d2, d1),
          ]
     }

     /// Compares two blocks in topological sweep-line order.
     ///
     /// # Order Invariants:
     /// 1. `w` ascending: Ensures blocks arrive in increasing width order.
     /// 2. `l` descending: Prevents blocks with equal width from stacking on each other.
     /// 3. `h` descending: Groups identical bases with the tallest block appearing first.
     #[must_use]
     fn cmp_topological(&self, other: &Self) -> Ordering {
          self.w
              .cmp(&other.w)
              .then_with(|| other.l.cmp(&self.l))
              .then_with(|| other.h.cmp(&self.h))
     }
}

/// A Fenwick Tree (Binary Indexed Tree) tracking the maximum tower height
/// and the index of the corresponding top block for prefix queries.
struct FenwickTree {
     /// 1-based internal storage: index `i` maps to `(max_height, Option<block_index>)`.
     height_and_block: Vec<(u32, Option<usize>)>,
}

impl FenwickTree {
     /// Creates an empty [`FenwickTree`] sized for `size` compressed coordinates.
     #[must_use]
     fn new(size: usize) -> Self {
          // Extra slot allocated at index 0 to support standard 1-based indexing.
          let height_and_block: Vec<(u32, Option<usize>)> = vec![(0, None); size + 1];
          Self { height_and_block }
     }

     /// Queries the maximum accumulated height and top block index in the prefix `1..=idx`.
     ///
     /// Returns `(0, None)` if `idx == 0` or if no predecessor exists in that range.
     #[must_use]
     fn query(&self, mut idx: usize) -> (u32, Option<usize>) {
          let mut best = (0, None);
          while idx > 0 {
               if let Some((height, block_idx)) = self.height_and_block.get(idx) {
                    if *height > best.0 {
                         best.0 = *height;
                         best.1 = *block_idx;
                    }
               } else {
                    return (0, None);
               }

               // Clears the lowest set bit (LSB) to step back through prefix intervals.
               idx -= idx & (!idx).wrapping_add(1);
          }

          best
     }

     /// Updates the tree at position `idx` if `height` strictly exceeds the stored value.
     fn update(&mut self, mut idx: usize, height: u32, block_idx: usize) {
          while idx < self.height_and_block.len() {
               if let Some((stored_height, stored_block_idx)) = self.height_and_block.get_mut(idx) {
                    if height > *stored_height {
                         *stored_height = height;
                         *stored_block_idx = Some(block_idx);
                    }
               } else {
                    return;
               }

               // Adds the lowest set bit (LSB) to propagate the update upwards.
               idx += idx & (!idx).wrapping_add(1);
          }
     }
}

/// Extracts the next positive integer `u32` from a raw byte buffer.
#[must_use]
fn next_u32(buffer: &[u8], cursor: &mut usize) -> Option<u32> {
     // 1. Skip whitespace characters
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

     // 2. Parse sequential ASCII decimal digits without panicking
     let mut value: u32 = 0;
     while let Some(&byte) = buffer.get(*cursor) {
          if byte.is_ascii_digit() {
               let digit = u32::from(byte - b'0');
               value = value.checked_mul(10)?.checked_add(digit)?;
               *cursor += 1;
          } else {
               break;
          }
     }

     Some(value)
}

/// Entry point of the solver.
fn main() {
     let mut buffer = Vec::new();
     let mut cursor: usize = 0;

     if io::stdin().read_to_end(&mut buffer).is_err() {
          println!("Error while reading input.");
          return;
     }

     let Some(n) = next_u32(&buffer, &mut cursor) else {
          println!("Error while reading input: failed while obtaining N.");
          return;
     };

     let Ok(number_of_blocks) = usize::try_from(n) else {
          println!("Error: N is too large for memory indexing.");
          return;
     };

     // Each 3D box produces 3 canonical orientations
     let number_of_orientations = number_of_blocks * 3;
     let mut blocks: Vec<Block> = Vec::with_capacity(number_of_orientations);

     for _ in 0..number_of_blocks {
          let Some(side_a) = next_u32(&buffer, &mut cursor) else { return; };
          let Some(side_b) = next_u32(&buffer, &mut cursor) else { return; };
          let Some(height) = next_u32(&buffer, &mut cursor) else { return; };

          blocks.extend(Block::orientations(side_a, side_b, height));
     }

     // Step 1: Topological Sort
     // Primary: w ascending. Secondary: l descending. Tertiary: h descending.
     blocks.sort_unstable_by(Block::cmp_topological);

     // Step 2: Deduplication
     // For identical bases (w, l), keep only the first (tallest) block.
     blocks.dedup_by(|a, b| a.w == b.w && a.l == b.l);

     // Step 3: Coordinate Compression on base lengths (l)
     // Maps sparse length values into a contiguous 1-based range for Fenwick indexing.
     let mut unique_l: Vec<u32> = blocks.iter().map(|block| block.l).collect();
     unique_l.sort_unstable();
     unique_l.dedup();

     let mut tree = FenwickTree::new(unique_l.len());
     let mut table: Vec<u32> = Vec::with_capacity(blocks.len());
     let mut parents: Vec<Option<usize>> = vec![None; blocks.len()];

     // Step 4: Sweep-line processing with O(log M) transitions
     for (big_idx, current_block) in blocks.iter().enumerate() {
          let Ok(l_idx) = unique_l.binary_search(&current_block.l) else {
               println!("Error: length not found in unique_l.");
               return;
          };

          // Querying `l_idx` (0-based search index) queries elements strictly smaller
          // than the current length's 1-based tree index (`l_idx + 1`).
          let best_base = tree.query(l_idx);

          // Arithmetic overflow protection per NASA safety rules
          let Some(best_height) = best_base.0.checked_add(current_block.h) else {
               println!("Error while calculating cumulative tower height.");
               return;
          };

          // Register current block into the Fenwick Tree at its 1-based index (`l_idx + 1`)
          tree.update(l_idx + 1, best_height, big_idx);

          table.push(best_height);

          if let Some(parent) = parents.get_mut(big_idx) {
               *parent = best_base.1;
          } else {
               println!("Error while recording block predecessor.");
               return;
          }
     }

     // Step 5: Tower reconstruction from the optimal top block back down to the base
     if let Some((best_idx, &max_height)) =
          table.iter().enumerate().max_by_key(|&(_idx, &val)| val) {
          
          let mut used_blocks: Vec<usize> = Vec::with_capacity(blocks.len());
          let mut current_idx = Some(best_idx);

          while let Some(idx) = current_idx {
               used_blocks.push(idx);
               if let Some(parent) = parents.get(idx) {
                    current_idx = *parent;
               } else {
                    break;
               }
          }

          let stdout = io::stdout();
          let mut writer = BufWriter::new(stdout.lock());
          let number_of_used_blocks = used_blocks.len();

          if writeln!(writer, "{max_height} {number_of_used_blocks}").is_err() {
               return;
          }

          for idx in used_blocks {
               if let Some(b) = blocks.get(idx) {
                    if writeln!(writer, "{} {} {}", b.w, b.l, b.h).is_err() {
                         return;
                    }
               }
          }

          let _ = writer.flush();
     } else {
          println!("0");
     }
}