//! Result assembly module
//!
//! This module provides functionality to assemble partial results
//! from parallel processing into final sentence boundaries.

use crate::error::Result;
use sakurs_core::{Boundary, PartialState};

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
