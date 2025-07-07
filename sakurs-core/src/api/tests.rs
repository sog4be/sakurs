//! Tests for the new unified API

#[cfg(test)]
mod api_tests {
    use crate::api::*;

    #[test]
    fn test_processor_creation() {
        // Default processor
        let processor = SentenceProcessor::new();
        assert_eq!(processor.config().language, Language::default());

        // Language-specific processor
        let ja_processor = SentenceProcessor::for_language("ja").unwrap();
        assert_eq!(ja_processor.config().language, Language::Japanese);

        // Custom config
        let config = Config::builder()
            .language("en")
            .threads(4)
            .chunk_size(1024)
            .build()
            .unwrap();
        let custom_processor = SentenceProcessor::with_config(config).unwrap();
        assert_eq!(custom_processor.config().performance.threads, Some(4));
    }

    #[test]
    fn test_config_presets() {
        let fast = Config::fast();
        assert!(!fast.accuracy.enable_abbreviations);
        assert_eq!(fast.performance.chunk_size_kb, 1024);

        let balanced = Config::balanced();
        assert!(balanced.accuracy.enable_abbreviations);
        assert_eq!(balanced.performance.chunk_size_kb, 512);

        let accurate = Config::accurate();
        assert_eq!(accurate.performance.threads, Some(1));
        assert_eq!(accurate.performance.chunk_size_kb, 256);
    }

    #[test]
    fn test_input_variants() {
        // Text input
        let text_input = Input::from_text("Hello world.");
        let text = text_input.into_text().unwrap();
        assert_eq!(text, "Hello world.");

        // Bytes input
        let bytes_input = Input::from_bytes(b"Hello world.".to_vec());
        let bytes = bytes_input.into_bytes().unwrap();
        assert_eq!(bytes, b"Hello world.");
    }

    #[test]
    fn test_basic_processing() {
        let processor = SentenceProcessor::for_language("en").unwrap();
        let text = "Hello world. This is a test. Another sentence.";
        let input = Input::from_text(text);
        let output = processor.process(input).unwrap();

        assert_eq!(output.boundaries.len(), 3);
        // The boundaries point to the position after the period (one past the punctuation)
        assert_eq!(output.boundaries[0].offset, 12); // After '.' in "Hello world."
        assert_eq!(output.boundaries[1].offset, 28); // After '.' in "This is a test."
        assert_eq!(output.boundaries[2].offset, 46); // After '.' in "Another sentence."

        assert_eq!(output.metadata.stats.sentence_count, 3);
        assert_eq!(output.metadata.stats.bytes_processed, 46); // Total length of text
    }

    #[test]
    fn test_char_offset_calculation() {
        let processor = SentenceProcessor::for_language("ja").unwrap();
        let input = Input::from_text("こんにちは。世界。");
        let output = processor.process(input).unwrap();

        // Verify both byte and character offsets are correct
        assert_eq!(output.boundaries.len(), 2);
        assert_eq!(output.boundaries[0].char_offset, 6); // After "こんにちは。"
        assert_eq!(output.boundaries[1].char_offset, 9); // After "世界。"
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .language("en")
            .threads(8)
            .chunk_size(2048)
            .memory_limit(1024)
            .abbreviations(false)
            .numbers(true)
            .quotes(false)
            .build()
            .unwrap();

        assert_eq!(config.language, Language::English);
        assert_eq!(config.performance.threads, Some(8));
        assert_eq!(config.performance.chunk_size_kb, 2048);
        assert_eq!(config.performance.memory_limit_mb, Some(1024));
        assert!(!config.accuracy.enable_abbreviations);
        assert!(config.accuracy.enable_numbers);
        assert!(!config.accuracy.enable_quotes);
    }
}
