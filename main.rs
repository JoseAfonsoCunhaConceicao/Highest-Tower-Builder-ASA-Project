//! # 3D Box Stacking Solver (ASA Project)
//! 
//! High-performance, memory-safe solution for the 3D Box Stacking problem.
//! Built in complience with NASA JPL's "Power of Ten" safety-critical rules.

// 1. Native `rustc` lints (always enforced, includind on Mooshak)
#![forbid(unsafe_code)]                           // Enforce 100% memory safety
#![deny(warnings)]                                // NASA Rule 10: treat all warnings as errors
#![deny(unconditional_recursion)]                 // NASA Rule 1: forbid direct recursion
#![deny(missing_docs)]                            // Enforce '///' documentation on all public items

// Static analysis `clippy` lints (enforced locally via `cargo clippy`)
#![deny(
     clippy::all,                                 // Standard correctness, style and performance lints
     clippy::pedantic,                            // NASA Rule 10: strict pendantic static analysis
     clippy::unwrap_used,                         // NASA Rule 7: forbid '.unwrap()', require explicit handling
     clippy::expect_used,                         // NASA Rule 7: forbid '.expected()', prevent unexpected panics
     clippy::indexing_slicing                     // Enforce bounds-checked access or iterators over raw 'v[i]'
)]

use std::io::{self, Read, BufWriter, Write};
use std::cmp::Ordering;

/// Represents a 3D block in a specific canonical orientation where `w <= l`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Block {
     /// Width of the base (always the smaller or equal base dimension: `w <= l`).
     w: u32,
     /// Length of the base (always the larger or equal base dimension: `l >= w`).
     l: u32,
     /// Vertical height of the block in this orientation.
     h: u32 
}

impl Block {
     /// Creates a new canonical [`Block`] orientation, enforcing the `w <= l` invariant.
     #[must_use]
     const fn new(side_a: u32, side_b: u32, height: u32) -> Self {
          debug_assert!(side_a > 0 && side_b > 0 && height > 0,
               "All block dimensions must be strictly positive.");
          
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
               Self::new(d3, d2, d1)
          ]
     }

     /// Compares two blocks in topological order (biggest base first, tallest height first on ties).
     #[must_use]
     fn cmp_topological(&self, other: &Self) -> Ordering {
        self.w
            .cmp(&other.w)
            .then_with(|| self.l.cmp(&other.l))
            .then_with(|| other.h.cmp(&self.h))
    }
}


/// Extracts the next integer `u32` from the buffer, starting at the current position
#[must_use]
fn next_u32(buffer: &[u8], cursor: &mut usize) -> Option<u32> {
     // 1. Skip white spaces
     while let Some(&byte) = buffer.get(*cursor) {
          if byte.is_ascii_whitespace() {
               *cursor += 1;
          } else {
               break;
          }
     }

     // If buffer is empty
     if *cursor >= buffer.len() {
          return None;
     }

     // 2. Accumulate the number
     let mut value: u32 = 0;
     while let Some(&byte) = buffer.get(*cursor) {
          if byte.is_ascii_digit() {
               let digit = u32::from(byte - b'0');
               value = value * 10 + digit;
               *cursor += 1;
          } else {
               break;
          }
     }

     Some(value)
}

/// Entry point of the application.
fn main() {
     let mut buffer = Vec::new();
     let mut cursor: usize = 0;

     // Read the input's
     if io::stdin().read_to_end(&mut buffer).is_err() {
          println!("Error while reading input.");
          return;
     }

     // Parse first integer (n)
     let Some(n) = next_u32(&buffer, &mut cursor) else {
          println!("Error while reading input: failed while obtaining N.");
          return;
     };
     
     let Ok(number_of_blocks) = usize::try_from(n) else {
          println!("Error: N is too large for memory indexing.");
          return;
     };

     // The total number of blocks is the number of inputs multiplied by the 3 considered orientations
     let number_of_orientations = number_of_blocks * 3;
     let mut blocks: Vec<Block> = Vec::with_capacity(number_of_orientations);

     // Fill the blocks vector with all considered blocks and all of its orientations
     for _ in 0..number_of_blocks {
          let Some(side_a) = next_u32(&buffer, &mut cursor) else {
               println!("Error while reading input: failed while obtaining block dimensions.");
               return; 
          };
          let Some(side_b) = next_u32(&buffer, &mut cursor) else {
               println!("Error while reading input: failed while obtaining block dimensions.");
               return; 
          };
          let Some(height) = next_u32(&buffer, &mut cursor) else {
               println!("Error while reading input: failed while obtaining block dimensions.");
               return; 
          };

          blocks.extend(Block::orientations(side_a, side_b, height));
     }

     // Sort the blocks vector by decreasing order (width), ties are managed by length and height, in this order
     blocks.sort_unstable_by(Block::cmp_topological);

     // Filter duplicated blocks (same base), keeps the tallest
     blocks.dedup_by(|a, b| { a.w == b.w && a.l == b.l });

     // New array with the block's heights
     let mut table: Vec<u32> = blocks.iter().map(|block| block.h).collect();

     // Auxiliar array which stores the `parent` block index of each block
     let mut parents: Vec<Option<usize>> = vec![None; blocks.len()];

     for big_idx in 0..blocks.len() {
          if let Some(current_block) = blocks.get(big_idx) {
               let mut best_height = current_block.h;
               let mut best_parent = None;

               for little_idx in 0..big_idx {
                    if let Some(block) = blocks.get(little_idx) {
                         if current_block.w <= block.w {
                              break; // Early Break
                         } else if current_block.l > block.l {
                              let base_height;
                              
                              let possible_new_height;
                              
                              if let Some(height) = table.get(little_idx) {
                                   base_height = *height;
                              } else {
                                   println!("Error while calcuating the tower max height.");
                                   return; 
                              }
                              if let Some(height) = base_height.checked_add(current_block.h) {
                                   possible_new_height = height;
                              } else {
                                   println!("Error while calcuating the tower max height.");
                                   return; 
                              }
                              
                              if possible_new_height > best_height {
                                   best_height = possible_new_height;
                                   best_parent = Some(little_idx);
                              }
                         }
                    } else {
                         println!("Error while calcuating the tower max height.");
                         return;
                    }
               }

               if let Some(height) = table.get_mut(big_idx) {
                    *height = best_height;
               } else {
                    println!("Error while calculating the tower max height.");
                    return;
               }

               if let Some(parent) = parents.get_mut(big_idx) {
                    *parent = best_parent;
               } else {
                    println!("Error while identifying block parent.");
                    return;
               }
          } else {
               println!("Error while calcuating the tower max height.");
               return;
          }
     }

     // Obtain max height and the tower top block
     if let Some((best_idx,&max_height)) = table.iter()
          .enumerate()
          .max_by_key(|&(_idx, &val)| val) {
          
          let mut used_blocks: Vec<usize> = Vec::with_capacity(blocks.len());

          let mut current_idx = Some(best_idx);

          while let Some(idx) = current_idx {
               used_blocks.push(idx);
               if let Some(parent) = parents.get(idx) {
                    current_idx = *parent;
               }
          }

          let stdout = io::stdout();
          let mut writer = BufWriter::new(stdout.lock());

          
          let number_of_used_blocks: usize = used_blocks.len();
          
          if writeln!(writer, "{max_height} {number_of_used_blocks}").is_err() {
               println!("Error while writing the output.");
               return;
          }
          
          for idx in used_blocks {
               if let Some(b) = blocks.get(idx) {
                    if writeln!(writer, "{} {} {}", b.w, b.l, b.h).is_err() {
                         println!("Error while writing the output.");
                         return;    
                    }
               }
          }
          if writer.flush().is_err() {
               println!("Error while flushing the output.");
               return;
          }
     } else {
          println!("0");
     }
}