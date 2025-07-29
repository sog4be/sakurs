//! Delta-Stack Monoid algorithm implementation

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::cmp;

use crate::{
    error::{CoreError, Result},
    traits::LanguageRules,
    types::{Boundary, BoundaryKind, Class},
};

/// Maximum supported enclosure types
pub const ENCLOSURE_MAX: usize = 16;

/// Delta vector storing net changes and minimums for each enclosure type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeltaVec {
    /// Array of (net_change, minimum) pairs
    data: [(i32, i32); ENCLOSURE_MAX],
    /// Number of active enclosure types
    pub len: usize,
}

impl DeltaVec {
    /// Create a new delta vector with specified capacity
    pub fn new(len: usize) -> Result<Self> {
        if len > ENCLOSURE_MAX {
            return Err(CoreError::TooManyEnclosureTypes);
        }
        Ok(Self {
            data: [(0, 0); ENCLOSURE_MAX],
            len,
        })
    }

    /// Get the (net, min) pair for an enclosure type
    pub fn get(&self, idx: usize) -> Option<(i32, i32)> {
        if idx < self.len {
            Some(self.data[idx])
        } else {
            None
        }
    }

    /// Set the (net, min) pair for an enclosure type
    pub fn set(&mut self, idx: usize, net: i32, min: i32) -> Result<()> {
        if idx >= self.len {
            return Err(CoreError::TooManyEnclosureTypes);
        }
        self.data[idx] = (net, min);
        Ok(())
    }

    /// Combine two delta vectors (monoid operation)
    pub fn combine(&self, other: &Self) -> Result<Self> {
        if self.len != other.len {
            return Err(CoreError::TooManyEnclosureTypes);
        }

        let mut result = *self;
        for i in 0..self.len {
            let (net1, min1) = self.data[i];
            let (net2, min2) = other.data[i];
            result.data[i] = (
                net1.saturating_add(net2),
                cmp::min(min1, net1.saturating_add(min2)),
            );
        }
        Ok(result)
    }
}

/// Partial state for a text chunk
#[derive(Debug, Clone)]
pub struct PartialState {
    /// Detected boundary candidates
    #[cfg(feature = "alloc")]
    pub boundaries: Vec<Boundary>,
    /// Delta vector for enclosure tracking
    pub deltas: DeltaVec,
    /// Whether chunk ends with a dangling dot
    pub dangling_dot: bool,
    /// Whether chunk starts with alphabetic
    pub head_alpha: bool,
}

impl PartialState {
    /// Create a new partial state
    pub fn new(enclosure_count: usize) -> Result<Self> {
        Ok(Self {
            #[cfg(feature = "alloc")]
            boundaries: Vec::new(),
            deltas: DeltaVec::new(enclosure_count)?,
            dangling_dot: false,
            head_alpha: false,
        })
    }

    /// Identity element for monoid
    pub fn identity(enclosure_count: usize) -> Result<Self> {
        Self::new(enclosure_count)
    }

    /// Combine two partial states (monoid operation)
    pub fn combine(&self, other: &Self) -> Result<Self> {
        let mut result = self.clone();

        // Combine deltas
        result.deltas = self.deltas.combine(&other.deltas)?;

        // Handle cross-chunk abbreviation
        #[cfg(feature = "alloc")]
        if self.dangling_dot && other.head_alpha && !self.boundaries.is_empty() {
            // Remove the last boundary if it's a false positive
            result.boundaries.pop();
        }

        // Append other boundaries with offset adjustment
        #[cfg(feature = "alloc")]
        {
            let offset = self.boundaries.last().map(|b| b.byte_offset).unwrap_or(0);

            for boundary in &other.boundaries {
                result.boundaries.push(Boundary {
                    byte_offset: boundary.byte_offset + offset,
                    char_offset: boundary.char_offset, // Will be recalculated
                    kind: boundary.kind,
                });
            }
        }

        result.dangling_dot = other.dangling_dot;
        result.head_alpha = self.head_alpha && other.boundaries.is_empty();

        Ok(result)
    }
}

/// Streaming delta scanner for character-by-character processing
pub struct DeltaScanner<'r, R: LanguageRules> {
    rules: &'r R,
    state: PartialState,
    depths: [i32; ENCLOSURE_MAX],
    total_depth: i32,
    byte_offset: usize,
    char_offset: usize,
    last_was_dot: bool,
    /// Buffer for tracking recent text (for abbreviation detection)
    #[cfg(feature = "alloc")]
    text_buffer: Vec<char>,
    /// Maximum characters to keep in buffer
    buffer_limit: usize,
}

impl<'r, R: LanguageRules> DeltaScanner<'r, R> {
    /// Create a new scanner
    pub fn new(rules: &'r R) -> Result<Self> {
        Ok(Self {
            rules,
            state: PartialState::new(rules.max_enclosure_pairs())?,
            depths: [0; ENCLOSURE_MAX],
            total_depth: 0,
            byte_offset: 0,
            char_offset: 0,
            last_was_dot: false,
            #[cfg(feature = "alloc")]
            text_buffer: Vec::with_capacity(128),
            buffer_limit: 128, // Keep last 128 chars for abbreviation context
        })
    }

    /// Process a single character and emit boundaries
    pub fn step(&mut self, ch: char, emit: &mut impl FnMut(Boundary)) -> Result<()> {
        let char_len = ch.len_utf8();

        // Update text buffer for abbreviation detection
        #[cfg(feature = "alloc")]
        {
            self.text_buffer.push(ch);
            if self.text_buffer.len() > self.buffer_limit {
                self.text_buffer.remove(0);
            }
        }

        // Update state for head_alpha detection
        if self.byte_offset == 0 && matches!(self.rules.classify_char(ch), Class::Alpha) {
            self.state.head_alpha = true;
        }

        // Handle enclosures
        if let Some((pair_id, is_opening)) = self.rules.get_enclosure_pair(ch) {
            let idx = pair_id as usize;
            if idx >= self.state.deltas.len {
                return Err(CoreError::TooManyEnclosureTypes);
            }

            if is_opening {
                self.depths[idx] = self.depths[idx].saturating_add(1);
                self.total_depth = self.total_depth.saturating_add(1);
            } else {
                self.depths[idx] = self.depths[idx].saturating_sub(1);
                self.total_depth = self.total_depth.saturating_sub(1);

                // Update minimum
                let (net, min) = self.state.deltas.get(idx).unwrap_or((0, 0));
                self.state.deltas.set(
                    idx,
                    net.saturating_sub(1),
                    cmp::min(min, self.depths[idx]),
                )?;
            }

            // Update net change
            let (_, min) = self.state.deltas.get(idx).unwrap_or((0, 0));
            self.state.deltas.set(idx, self.depths[idx], min)?;
        }

        // Check for terminators
        if self.rules.is_terminator(ch) && self.total_depth == 0 {
            let mut is_abbrev = false;

            // Check if it's an abbreviation by looking at the buffer
            #[cfg(feature = "alloc")]
            if ch == '.' && !self.text_buffer.is_empty() {
                // Find the start of the current word
                let mut word_start = self.text_buffer.len().saturating_sub(1);
                while word_start > 0 {
                    let prev_char = self.text_buffer[word_start - 1];
                    if !matches!(self.rules.classify_char(prev_char), Class::Alpha) {
                        break;
                    }
                    word_start -= 1;
                }

                // Extract the word before the dot
                if word_start < self.text_buffer.len() - 1 {
                    let word: String = self.text_buffer[word_start..self.text_buffer.len() - 1]
                        .iter()
                        .collect();

                    // Check if it's an abbreviation
                    is_abbrev = self.rules.abbrev_match(&word);
                }
            }

            // For no_std, fall back to simple heuristic
            #[cfg(not(feature = "alloc"))]
            {
                is_abbrev = ch == '.' && self.last_was_dot;
            }

            let boundary = Boundary::new(
                self.byte_offset + char_len,
                self.char_offset + 1,
                if is_abbrev {
                    BoundaryKind::Abbreviation
                } else {
                    BoundaryKind::Strong
                },
            );

            emit(boundary);
        }

        // Update tracking
        self.last_was_dot = ch == '.';
        self.byte_offset += char_len;
        self.char_offset += 1;

        Ok(())
    }

    /// Finish processing and return final state
    pub fn finish(mut self) -> PartialState {
        self.state.dangling_dot = self.last_was_dot;
        self.state
    }
}

// Utility functions

/// Default emit function that pushes to a vector
#[cfg(feature = "alloc")]
pub fn emit_push(boundaries: &mut Vec<Boundary>) -> impl FnMut(Boundary) + '_ {
    move |boundary| boundaries.push(boundary)
}

/// Sequential emit function that commits immediately if at depth 0
#[cfg(feature = "alloc")]
pub fn emit_commit_if_depth0(
    boundaries: &mut Vec<Boundary>,
    depth: i32,
) -> impl FnMut(Boundary) + '_ {
    move |boundary| {
        if depth == 0 {
            boundaries.push(boundary)
        }
    }
}

/// Scan a chunk of text and collect boundaries
#[cfg(feature = "alloc")]
pub fn scan_chunk<R: LanguageRules>(
    text: &str,
    rules: &R,
    emit: &mut impl FnMut(Boundary),
) -> Result<PartialState> {
    let mut scanner = DeltaScanner::new(rules)?;

    for ch in text.chars() {
        scanner.step(ch, emit)?;
    }

    Ok(scanner.finish())
}

/// Reduce/combine delta vectors
pub fn reduce_deltas(deltas: &[DeltaVec]) -> Result<DeltaVec> {
    if deltas.is_empty() {
        return DeltaVec::new(0);
    }

    let mut result = deltas[0];
    for delta in &deltas[1..] {
        result = result.combine(delta)?;
    }

    Ok(result)
}

/// Reference sequential implementation for testing
#[cfg(feature = "alloc")]
pub fn run<R: LanguageRules>(text: &str, rules: &R) -> Result<Vec<Boundary>> {
    let mut boundaries = Vec::new();
    let mut scanner = DeltaScanner::new(rules)?;

    for ch in text.chars() {
        scanner.step(ch, &mut emit_push(&mut boundaries))?;
    }

    Ok(boundaries)
}
