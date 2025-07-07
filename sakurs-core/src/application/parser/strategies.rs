//! Parsing strategies for different processing modes

use crate::domain::{LanguageRules, PartialState};

/// Input for parsing operations
pub enum ParsingInput<'a> {
    /// Complete text in memory
    Text(&'a str),
    /// Streaming chunks
    Chunks(Box<dyn Iterator<Item = &'a str> + 'a>),
}

/// Output from parsing operations
pub enum ParsingOutput {
    /// Single state for complete text
    State(Box<PartialState>),
    /// Multiple states for streaming
    States(Vec<PartialState>),
}

/// Strategy for parsing text into partial states
pub trait ParseStrategy: Send + Sync {
    /// Parse input using this strategy
    fn parse(
        &self,
        input: ParsingInput,
        rules: &dyn LanguageRules,
    ) -> Result<ParsingOutput, ParseError>;

    /// Check if this strategy supports streaming
    fn supports_streaming(&self) -> bool;

    /// Get optimal chunk size for this strategy
    fn optimal_chunk_size(&self) -> usize;
}

/// Error type for parsing operations
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid UTF-8 in input")]
    InvalidUtf8,
    #[error("Empty input")]
    EmptyInput,
    #[error("Parsing failed: {0}")]
    ParsingFailed(String),
}

/// Sequential parsing strategy for small texts
pub struct SequentialParser {
    chunk_size: usize,
}

impl SequentialParser {
    pub fn new() -> Self {
        Self {
            chunk_size: 65536, // 64KB default
        }
    }

    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
}

impl Default for SequentialParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseStrategy for SequentialParser {
    fn parse(
        &self,
        input: ParsingInput,
        rules: &dyn LanguageRules,
    ) -> Result<ParsingOutput, ParseError> {
        match input {
            ParsingInput::Text(text) => {
                if text.is_empty() {
                    return Err(ParseError::EmptyInput);
                }

                // Use scan_chunk from this module
                let state = super::scan_chunk(text, rules);
                Ok(ParsingOutput::State(Box::new(state)))
            }
            ParsingInput::Chunks(chunks) => {
                let mut states = Vec::new();
                for chunk in chunks {
                    if !chunk.is_empty() {
                        let state = super::scan_chunk(chunk, rules);
                        states.push(state);
                    }
                }

                if states.is_empty() {
                    return Err(ParseError::EmptyInput);
                }

                Ok(ParsingOutput::States(states))
            }
        }
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn optimal_chunk_size(&self) -> usize {
        self.chunk_size
    }
}

/// Streaming parsing strategy for large files
pub struct StreamingParser {
    buffer_size: usize,
    overlap_size: usize,
}

impl StreamingParser {
    pub fn new() -> Self {
        Self {
            buffer_size: 1_048_576, // 1MB
            overlap_size: 256,      // For cross-chunk context
        }
    }

    pub fn with_buffer_size(buffer_size: usize, overlap_size: usize) -> Self {
        Self {
            buffer_size,
            overlap_size,
        }
    }
}

impl Default for StreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseStrategy for StreamingParser {
    fn parse(
        &self,
        input: ParsingInput,
        rules: &dyn LanguageRules,
    ) -> Result<ParsingOutput, ParseError> {
        match input {
            ParsingInput::Text(text) => {
                if text.is_empty() {
                    return Err(ParseError::EmptyInput);
                }

                // For streaming parser, we can still handle complete text
                // by treating it as a single chunk
                let state = super::scan_chunk(text, rules);
                Ok(ParsingOutput::State(Box::new(state)))
            }
            ParsingInput::Chunks(chunks) => {
                let mut states = Vec::new();
                let mut overlap = String::new();

                for chunk in chunks {
                    // Combine with overlap from previous chunk
                    let combined = if overlap.is_empty() {
                        chunk.to_string()
                    } else {
                        format!("{}{}", overlap, chunk)
                    };

                    if !combined.is_empty() {
                        let state = super::scan_chunk(&combined, rules);
                        states.push(state);

                        // Keep overlap for next iteration
                        if chunk.len() >= self.overlap_size {
                            overlap = chunk[chunk.len() - self.overlap_size..].to_string();
                        } else {
                            overlap = chunk.to_string();
                        }
                    }
                }

                if states.is_empty() {
                    return Err(ParseError::EmptyInput);
                }

                Ok(ParsingOutput::States(states))
            }
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn optimal_chunk_size(&self) -> usize {
        self.buffer_size
    }
}
