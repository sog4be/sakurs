//! Streaming strategy implementation for processing large files

use super::{buffer::StreamingBuffer, detector::BoundaryDetector, state::StreamingState};
use crate::{
    application::{
        config::{ProcessingError, ProcessingResult as Result},
        strategies::traits::{
            InputCharacteristics, ProcessingConfig, ProcessingStrategy, StrategyInput,
            StrategyOutput,
        },
    },
    domain::language::LanguageRules,
};
use std::{io::Read, sync::Arc};

/// Strategy for processing very large files with bounded memory usage
pub struct StreamingStrategy {
    look_ahead_size: usize,
}

impl StreamingStrategy {
    /// Create a new streaming strategy
    pub fn new() -> Self {
        Self {
            look_ahead_size: 1024, // 1KB look-ahead
        }
    }

    /// Process a reader with streaming
    pub fn process_reader(
        &self,
        reader: &mut dyn Read,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<Vec<usize>> {
        let mut buffer = StreamingBuffer::new(config.buffer_size, self.look_ahead_size);
        let mut detector = BoundaryDetector::new(language_rules);
        let mut state = StreamingState::new();
        let mut all_boundaries = Vec::new();
        let mut total_bytes_processed = 0;

        loop {
            // Fill buffer
            let bytes_read = buffer.fill(reader)?;

            if bytes_read == 0 {
                break; // EOF
            }

            // Get processable chunk (excluding look-ahead)
            let chunk_data = buffer.processable_chunk();

            // If chunk is empty (buffer smaller than look-ahead), use full buffer
            let data_to_process = if chunk_data.is_empty() && !buffer.is_empty() {
                buffer.full_buffer()
            } else {
                chunk_data
            };

            // Skip if no data to process
            if data_to_process.is_empty() {
                break;
            }

            // Convert to string
            let chunk_str =
                std::str::from_utf8(data_to_process).map_err(|e| ProcessingError::Utf8Error {
                    position: e.valid_up_to(),
                })?;

            // Detect boundaries in this chunk
            let chunk_boundaries = detector.detect_boundaries(chunk_str);

            // Convert chunk-relative boundaries to absolute positions
            let absolute_boundaries: Vec<usize> = chunk_boundaries
                .iter()
                .map(|&b| b + total_bytes_processed)
                .collect();

            // Update state
            state.update(&chunk_boundaries, total_bytes_processed);

            // Collect boundaries
            all_boundaries.extend(absolute_boundaries);

            // Update position
            total_bytes_processed += data_to_process.len();

            // For small texts that fit in one buffer, we're done
            if bytes_read < config.buffer_size {
                break;
            }
        }

        Ok(all_boundaries)
    }

    /// Process text string by converting to reader
    fn process_text(
        &self,
        text: &str,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<Vec<usize>> {
        let mut cursor = std::io::Cursor::new(text.as_bytes());
        self.process_reader(&mut cursor, language_rules, config)
    }

    /// Process file by opening and streaming
    fn process_file(
        &self,
        path: std::path::PathBuf,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<Vec<usize>> {
        let file = std::fs::File::open(&path)
            .map_err(|e| ProcessingError::Other(format!("Failed to open file: {e}")))?;
        let mut reader = std::io::BufReader::new(file);
        self.process_reader(&mut reader, language_rules, config)
    }
}

impl Default for StreamingStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessingStrategy for StreamingStrategy {
    fn process(
        &self,
        input: StrategyInput,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<StrategyOutput> {
        let boundaries = match input {
            StrategyInput::Text(text) => self.process_text(text, language_rules, config)?,
            StrategyInput::File(path) => self.process_file(path, language_rules, config)?,
            StrategyInput::Stream(mut reader) => {
                self.process_reader(reader.as_mut(), language_rules, config)?
            }
            StrategyInput::Chunks(chunks) => {
                // Process chunks as a concatenated stream
                let text = chunks.join("");
                self.process_text(&text, language_rules, config)?
            }
        };

        Ok(StrategyOutput::Boundaries(boundaries))
    }

    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32 {
        if characteristics.requires_streaming() {
            1.0 // Perfect for streaming scenarios
        } else if characteristics.is_large() {
            0.8 // Good for large files
        } else if characteristics.is_medium() {
            0.4 // Can handle but not optimal
        } else {
            0.1 // Too much overhead for small files
        }
    }

    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig {
        let buffer_size = if characteristics.is_streaming {
            8_388_608 // 8MB for true streaming
        } else if characteristics.size_bytes > 100_000_000 {
            4_194_304 // 4MB for very large files
        } else {
            1_048_576 // 1MB for large files
        };

        ProcessingConfig {
            chunk_size: buffer_size / 4, // Process in quarters
            thread_count: 1,             // Streaming is sequential
            buffer_size,
            overlap_size: 1024,                  // 1KB overlap
            prefetch_distance: 0,                // No prefetching in streaming
            memory_limit: Some(buffer_size * 2), // Limit memory usage
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_parallel(&self) -> bool {
        false // Streaming is inherently sequential
    }

    fn name(&self) -> &'static str {
        "streaming"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::EnglishLanguageRules;

    #[test]
    fn test_streaming_strategy() {
        let strategy = StreamingStrategy::new();
        let language_rules = Arc::new(EnglishLanguageRules::new());

        let text = "First sentence. Second sentence. Third sentence.";
        let result = strategy.process(
            StrategyInput::Text(text),
            language_rules,
            &ProcessingConfig::streaming(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            StrategyOutput::Boundaries(boundaries) => {
                assert_eq!(boundaries.len(), 3);
            }
            _ => panic!("Expected Boundaries output"),
        }
    }

    #[test]
    fn test_streaming_suitability() {
        let strategy = StreamingStrategy::new();

        // Streaming input
        let streaming = InputCharacteristics::streaming();
        assert_eq!(strategy.suitability_score(&streaming), 1.0);

        // Very large file
        let very_large = InputCharacteristics {
            size_bytes: 500_000_000,
            estimated_char_count: 500_000_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
            language_hint: None,
        };
        assert!(strategy.suitability_score(&very_large) > 0.7);

        // Small file
        let small = InputCharacteristics::from_text("small");
        assert!(strategy.suitability_score(&small) < 0.2);
    }
}
