//! Streaming strategy implementation for processing large files

use super::{buffer::StreamingBuffer, detector::BoundaryDetector, state::StreamingState};
use crate::{
    application::config::{ProcessingError, ProcessingResult as Result},
    domain::language::LanguageRules,
    processing::{InputCharacteristics, ProcessingConfig, ProcessingStrategy},
};
use std::{io::Read, sync::Arc};

/// Strategy for processing very large files with bounded memory usage
pub struct StreamingStrategy {
    language_rules: Arc<dyn LanguageRules>,
    #[allow(dead_code)]
    default_buffer_size: usize,
    look_ahead_size: usize,
}

impl StreamingStrategy {
    /// Create a new streaming strategy
    pub fn new(language_rules: Arc<dyn LanguageRules>) -> Self {
        Self {
            language_rules,
            default_buffer_size: 4 * 1024 * 1024, // 4MB
            look_ahead_size: 1024,                // 1KB look-ahead
        }
    }

    /// Process a reader with streaming
    pub fn process_reader(
        &self,
        reader: &mut impl Read,
        config: &ProcessingConfig,
    ) -> Result<Vec<usize>> {
        let mut buffer = StreamingBuffer::new(config.buffer_size, self.look_ahead_size);
        let mut detector = BoundaryDetector::new(self.language_rules.clone());
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
}

impl ProcessingStrategy for StreamingStrategy {
    fn process(&self, text: &str, config: &ProcessingConfig) -> Result<Vec<usize>> {
        // For string input, create a cursor and use the reader-based implementation
        let mut cursor = std::io::Cursor::new(text.as_bytes());
        self.process_reader(&mut cursor, config)
    }

    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32 {
        if characteristics.is_streaming {
            return 1.0; // Always best for streaming input
        }

        // Score based on file size
        match characteristics.size_bytes {
            0..=10_485_760 => 0.0,              // <10MB: not suitable
            10_485_761..=104_857_600 => 0.5,    // 10-100MB: somewhat suitable
            104_857_601..=1_073_741_824 => 0.8, // 100MB-1GB: very suitable
            _ => 1.0,                           // >1GB: highly suitable
        }
    }

    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig {
        let buffer_size = match characteristics.size_bytes {
            0..=104_857_600 => 4 * 1024 * 1024, // <100MB: 4MB buffer
            104_857_601..=1_073_741_824 => 8 * 1024 * 1024, // 100MB-1GB: 8MB buffer
            _ => 16 * 1024 * 1024,              // >1GB: 16MB buffer
        };

        ProcessingConfig {
            chunk_size: buffer_size,
            thread_count: 1, // Streaming is sequential
            buffer_size,
            prefetch_distance: 64, // Prefetch for cache optimization
        }
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
        let rules = Arc::new(EnglishLanguageRules::new());
        let strategy = StreamingStrategy::new(rules);

        let text = "This is a test. It has multiple sentences. Each one ends with a period.";
        let config = ProcessingConfig::default();

        let boundaries = strategy.process(text, &config).unwrap();
        assert_eq!(boundaries.len(), 3);
    }

    #[test]
    fn test_suitability_scoring() {
        let rules = Arc::new(EnglishLanguageRules::new());
        let strategy = StreamingStrategy::new(rules);

        // Small file - not suitable
        let small_characteristics = InputCharacteristics {
            size_bytes: 1_000_000, // 1MB
            estimated_char_count: 1_000_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
        };
        assert_eq!(strategy.suitability_score(&small_characteristics), 0.0);

        // Very large file - highly suitable
        let large_characteristics = InputCharacteristics {
            size_bytes: 2_000_000_000, // 2GB
            estimated_char_count: 2_000_000_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
        };
        assert_eq!(strategy.suitability_score(&large_characteristics), 1.0);

        // Streaming input - always suitable
        let streaming_characteristics = InputCharacteristics {
            size_bytes: 0,
            estimated_char_count: 0,
            is_streaming: true,
            available_memory: 1_073_741_824,
            cpu_count: 8,
        };
        assert_eq!(strategy.suitability_score(&streaming_characteristics), 1.0);
    }
}
