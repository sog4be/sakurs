//! Result assembly module
//!
//! This module provides functionality to assemble partial results
//! from parallel processing into final sentence boundaries.

use crate::error::Result;
use sakurs_core::{Boundary, DeltaVec, PartialState};

/// Assembler for combining partial results
#[derive(Debug)]
pub struct ResultAssembler {
    // Configuration for assembly process
}

impl ResultAssembler {
    /// Create a new result assembler
    pub fn new() -> Self {
        Self {}
    }

    /// Assemble partial states into final boundaries
    pub fn assemble(&self, states: Vec<PartialState>) -> Result<Vec<Boundary>> {
        if states.is_empty() {
            return Ok(Vec::new());
        }

        // Start with the first state
        let mut result = states[0].clone();

        // Combine all subsequent states
        for state in states.into_iter().skip(1) {
            result = result.combine(&state)?;
        }

        Ok(result.boundaries)
    }

    /// Assemble boundaries with proper offset adjustment
    pub fn assemble_with_offsets(
        &self,
        boundary_chunks: Vec<(Vec<Boundary>, usize)>, // (boundaries, chunk_start_offset)
    ) -> Vec<Boundary> {
        let mut result = Vec::new();

        for (boundaries, base_offset) in boundary_chunks {
            for boundary in boundaries {
                result.push(Boundary::new(
                    boundary.byte_offset + base_offset,
                    boundary.char_offset, // Will be recalculated if needed
                    boundary.kind,
                ));
            }
        }

        result
    }
}

impl Default for ResultAssembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sakurs_core::{Boundary, BoundaryKind};

    #[test]
    fn test_assemble_with_offsets() {
        let assembler = ResultAssembler::new();

        let chunks = vec![
            (vec![Boundary::new(10, 10, BoundaryKind::Strong)], 0),
            (vec![Boundary::new(15, 15, BoundaryKind::Strong)], 20),
        ];

        let result = assembler.assemble_with_offsets(chunks);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].byte_offset, 10);
        assert_eq!(result[1].byte_offset, 35); // 15 + 20
    }
}

/// Merge boundaries with cross-chunk abbreviation fix
///
/// This function handles offset shifting and detects abbreviations that
/// were incorrectly split across chunk boundaries.
///
/// # Arguments
/// * `boundaries` - Vector of boundaries from each chunk
/// * `deltas` - Delta vectors from each chunk for state tracking
/// * `dangling_dots` - Whether each chunk ended with a dot
/// * `head_alphas` - Whether each chunk started with alphabetic
/// * `offsets` - Byte offset where each chunk starts
pub fn merge_boundaries(
    boundaries: Vec<Vec<Boundary>>,
    _deltas: &[DeltaVec],
    dangling_dots: &[bool],
    head_alphas: &[bool],
    offsets: &[usize],
) -> Vec<Boundary> {
    let mut result = Vec::new();

    for (chunk_idx, chunk_boundaries) in boundaries.iter().enumerate() {
        let base_offset = offsets[chunk_idx];

        for boundary in chunk_boundaries {
            let adjusted_boundary = Boundary::new(
                boundary.byte_offset + base_offset,
                boundary.char_offset, // Will need recalculation based on full text
                boundary.kind,
            );

            // Check if this is a false boundary from abbreviation split
            if chunk_idx > 0 && boundary.byte_offset == 0 {
                // This boundary is at the start of the chunk
                if dangling_dots[chunk_idx - 1] && head_alphas[chunk_idx] {
                    // Previous chunk ended with dot, this starts with alpha
                    // This is likely an abbreviation split - skip this boundary
                    continue;
                }
            }

            result.push(adjusted_boundary);
        }
    }

    result
}
