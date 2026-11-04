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

     /// Compares two blocks in topological order (smallest base first, tallest height first on ties).
     #[must_use]
     fn cmp_topological(&self, other: &Self) -> Ordering {
        self.w
            .cmp(&other.w)
            .then_with(|| self.l.cmp(&other.l))
            .then_with(|| other.h.cmp(&self.h))
    }
}

/// Entry point of the application.
fn main() {
    let rotations = Block::orientations(5, 10, 20);

    for b in rotations {
     debug_assert!(b.w <= b.l, "Normalization invariant violated: w > l.");
     println!("Orientation -> w: {}, l: {}, h: {}", b.w, b.l, b.h);
    }
}