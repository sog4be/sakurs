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
        let ja_processor = SentenceProcessor::with_language("ja").unwrap();
        assert_eq!(ja_processor.config().language, Language::Japanese);

        // Custom config
        let config = Config::builder()
            .language("en")
            .unwrap()
            .threads(Some(4))
            .chunk_size(1024 * 1024) // 1MB in bytes
            .build()
            .unwrap();
        let custom_processor = SentenceProcessor::with_config(config).unwrap();
        assert_eq!(custom_processor.config().threads, Some(4));
    }

    #[test]
    fn test_config_defaults() {
        let default_config = Config::default();
        assert_eq!(default_config.language, Language::English);
        assert_eq!(default_config.chunk_size, 256 * 1024); // 256KB
        assert_eq!(default_config.parallel_threshold, 1024 * 1024); // 1MB
        assert_eq!(default_config.overlap_size, 256); // 256 bytes
        assert_eq!(default_config.threads, None); // All available threads
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
        let processor = SentenceProcessor::with_language("en").unwrap();
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
        let processor = SentenceProcessor::with_language("ja").unwrap();
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
            .unwrap()
            .threads(Some(8))
            .chunk_size(2 * 1024 * 1024) // 2MB in bytes
            .build()
            .unwrap();

        assert_eq!(config.language, Language::English);
        assert_eq!(config.threads, Some(8));
        assert_eq!(config.chunk_size, 2 * 1024 * 1024);
    }

    #[test]
    fn test_config_validation() {
        // Test invalid chunk size
        let result = Config::builder().chunk_size(0).build();
        assert!(result.is_err());

        // Test invalid thread count
        let result = Config::builder().threads(Some(0)).build();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod input_tests {
    use crate::api::{Error, Input};
    use std::io::{Cursor, Read};

    mod input_construction_tests {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn test_from_text() {
            // Test with String
            let input = Input::from_text(String::from("Hello, world!"));
            match input {
                Input::Text(text) => assert_eq!(text, "Hello, world!"),
                _ => panic!("Expected Input::Text"),
            }

            // Test with &str
            let input = Input::from_text("Hello again!");
            match input {
                Input::Text(text) => assert_eq!(text, "Hello again!"),
                _ => panic!("Expected Input::Text"),
            }

            // Test with empty string
            let input = Input::from_text("");
            match input {
                Input::Text(text) => assert_eq!(text, ""),
                _ => panic!("Expected Input::Text"),
            }
        }

        #[test]
        fn test_from_file() {
            // Test with Path
            let path = std::path::Path::new("/tmp/test.txt");
            let input = Input::from_file(path);
            match input {
                Input::File(file_path) => assert_eq!(file_path, PathBuf::from("/tmp/test.txt")),
                _ => panic!("Expected Input::File"),
            }

            // Test with String path
            let input = Input::from_file("/home/user/document.txt");
            match input {
                Input::File(file_path) => {
                    assert_eq!(file_path, PathBuf::from("/home/user/document.txt"))
                }
                _ => panic!("Expected Input::File"),
            }

            // Test with PathBuf
            let path_buf = PathBuf::from("./relative/path.txt");
            let input = Input::from_file(&path_buf);
            match input {
                Input::File(file_path) => assert_eq!(file_path, path_buf),
                _ => panic!("Expected Input::File"),
            }
        }

        #[test]
        fn test_from_bytes() {
            // Test with ASCII bytes
            let bytes = b"Hello bytes!".to_vec();
            let input = Input::from_bytes(bytes.clone());
            match input {
                Input::Bytes(b) => assert_eq!(b, bytes),
                _ => panic!("Expected Input::Bytes"),
            }

            // Test with UTF-8 bytes
            let utf8_bytes = "こんにちは".as_bytes().to_vec();
            let input = Input::from_bytes(utf8_bytes.clone());
            match input {
                Input::Bytes(b) => assert_eq!(b, utf8_bytes),
                _ => panic!("Expected Input::Bytes"),
            }

            // Test with empty bytes
            let input = Input::from_bytes(vec![]);
            match input {
                Input::Bytes(b) => assert!(b.is_empty()),
                _ => panic!("Expected Input::Bytes"),
            }
        }

        #[test]
        fn test_from_reader() {
            // Test with Cursor reader
            let data = b"Reader data";
            let reader = Cursor::new(data);
            let input = Input::from_reader(reader);
            match input {
                Input::Reader(_) => {} // Can't inspect the reader directly
                _ => panic!("Expected Input::Reader"),
            }
        }

        #[test]
        fn test_debug_impl() {
            // Test Text debug
            let text_input = Input::from_text("Test text");
            let debug_str = format!("{:?}", text_input);
            assert!(debug_str.contains("Input::Text"));
            assert!(debug_str.contains("length"));
            assert!(debug_str.contains("9")); // length of "Test text"

            // Test File debug
            let file_input = Input::from_file("/path/to/file.txt");
            let debug_str = format!("{:?}", file_input);
            assert!(debug_str.contains("Input::File"));
            assert!(debug_str.contains("path"));
            assert!(debug_str.contains("/path/to/file.txt"));

            // Test Bytes debug
            let bytes_input = Input::from_bytes(vec![1, 2, 3, 4, 5]);
            let debug_str = format!("{:?}", bytes_input);
            assert!(debug_str.contains("Input::Bytes"));
            assert!(debug_str.contains("length"));
            assert!(debug_str.contains("5"));

            // Test Reader debug
            let reader_input = Input::from_reader(Cursor::new(b"data"));
            let debug_str = format!("{:?}", reader_input);
            assert!(debug_str.contains("Input::Reader"));
        }
    }

    mod input_conversion_tests {
        use super::*;
        use std::fs;
        use std::io::Write;
        use tempfile::NamedTempFile;

        #[test]
        fn test_text_into_bytes() {
            let input = Input::from_text("Hello, 世界!");
            let bytes = input.into_bytes().unwrap();
            assert_eq!(bytes, "Hello, 世界!".as_bytes());
        }

        #[test]
        fn test_bytes_into_bytes() {
            let original_bytes = vec![72, 101, 108, 108, 111]; // "Hello"
            let input = Input::from_bytes(original_bytes.clone());
            let bytes = input.into_bytes().unwrap();
            assert_eq!(bytes, original_bytes);
        }

        #[test]
        fn test_file_into_bytes_success() {
            // Create a temporary file
            let mut temp_file = NamedTempFile::new().unwrap();
            let content = "File content test";
            temp_file.write_all(content.as_bytes()).unwrap();
            temp_file.flush().unwrap();

            let input = Input::from_file(temp_file.path());
            let bytes = input.into_bytes().unwrap();
            assert_eq!(bytes, content.as_bytes());
        }

        #[test]
        fn test_file_into_bytes_not_found() {
            let input = Input::from_file("/non/existent/file/path/that/should/not/exist.txt");
            let result = input.into_bytes();
            assert!(result.is_err());
            match result {
                Err(Error::Infrastructure(msg)) => {
                    assert!(msg.contains("Failed to read file"));
                }
                _ => panic!("Expected Infrastructure error"),
            }
        }

        #[test]
        fn test_reader_into_bytes() {
            let data = b"Reader content";
            let reader = Cursor::new(data);
            let input = Input::from_reader(reader);
            let bytes = input.into_bytes().unwrap();
            assert_eq!(bytes, data.to_vec());
        }

        #[test]
        fn test_reader_into_bytes_error() {
            // Create a reader that fails
            struct FailingReader;
            impl Read for FailingReader {
                fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                    Err(std::io::Error::other("Read failed"))
                }
            }

            let input = Input::from_reader(FailingReader);
            let result = input.into_bytes();
            assert!(result.is_err());
            match result {
                Err(Error::Infrastructure(msg)) => {
                    assert!(msg.contains("Failed to read from reader"));
                }
                _ => panic!("Expected Infrastructure error"),
            }
        }

        #[test]
        fn test_text_into_text() {
            let original = "Hello, テスト!";
            let input = Input::from_text(original);
            let text = input.into_text().unwrap();
            assert_eq!(text, original);
        }

        #[test]
        fn test_bytes_into_text_valid_utf8() {
            let utf8_bytes = "Valid UTF-8 string".as_bytes().to_vec();
            let input = Input::from_bytes(utf8_bytes);
            let text = input.into_text().unwrap();
            assert_eq!(text, "Valid UTF-8 string");
        }

        #[test]
        fn test_bytes_into_text_invalid_utf8() {
            // Invalid UTF-8 sequence
            let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
            let input = Input::from_bytes(invalid_bytes);
            let result = input.into_text();
            assert!(result.is_err());
            match result {
                Err(Error::Infrastructure(msg)) => {
                    assert!(msg.contains("Invalid UTF-8 encoding"));
                }
                _ => panic!("Expected Infrastructure error"),
            }
        }

        #[test]
        fn test_file_into_text_with_unicode() {
            // Create a temporary file with Unicode content
            let mut temp_file = NamedTempFile::new().unwrap();
            let content = "Unicode: 你好世界 🌍 émojis";
            temp_file.write_all(content.as_bytes()).unwrap();
            temp_file.flush().unwrap();

            let input = Input::from_file(temp_file.path());
            let text = input.into_text().unwrap();
            assert_eq!(text, content);
        }

        #[test]
        fn test_empty_inputs() {
            // Empty text
            let input = Input::from_text("");
            assert_eq!(input.into_bytes().unwrap(), Vec::<u8>::new());

            // Empty bytes
            let input = Input::from_bytes(vec![]);
            assert_eq!(input.into_text().unwrap(), "");

            // Empty reader
            let input = Input::from_reader(Cursor::new(b""));
            assert_eq!(input.into_bytes().unwrap(), Vec::<u8>::new());
        }

        #[test]
        fn test_large_input_handling() {
            // Test with large text input (1MB)
            let large_text = "x".repeat(1024 * 1024);
            let input = Input::from_text(large_text.clone());
            let bytes = input.into_bytes().unwrap();
            assert_eq!(bytes.len(), 1024 * 1024);

            // Convert back to text
            let input = Input::from_bytes(bytes);
            let text = input.into_text().unwrap();
            assert_eq!(text, large_text);
        }

        #[test]
        fn test_file_permissions_error() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                // Create a file with no read permissions
                let mut temp_file = NamedTempFile::new().unwrap();
                temp_file.write_all(b"secret").unwrap();
                temp_file.flush().unwrap();

                let path = temp_file.path().to_path_buf();
                let metadata = fs::metadata(&path).unwrap();
                let mut perms = metadata.permissions();
                perms.set_mode(0o000); // No permissions
                fs::set_permissions(&path, perms).unwrap();

                let input = Input::from_file(&path);
                let result = input.into_bytes();

                // Restore permissions before asserting (cleanup)
                let mut perms = fs::metadata(&path).unwrap().permissions();
                perms.set_mode(0o644);
                fs::set_permissions(&path, perms).ok();

                assert!(result.is_err());
            }
        }

        #[test]
        fn test_reader_with_different_types() {
            // Test with different reader types

            // Cursor with Vec<u8>
            let vec_data = vec![72, 101, 108, 108, 111];
            let input = Input::from_reader(Cursor::new(vec_data.clone()));
            assert_eq!(input.into_bytes().unwrap(), vec_data);

            // Cursor with &[u8]
            let slice_data = b"Slice reader";
            let input = Input::from_reader(Cursor::new(slice_data));
            assert_eq!(input.into_bytes().unwrap(), slice_data.to_vec());

            // Chain reader
            let part1 = Cursor::new(b"Part 1" as &[u8]);
            let part2 = Cursor::new(b" Part 2" as &[u8]);
            let chained = part1.chain(part2);
            let input = Input::from_reader(chained);
            assert_eq!(input.into_text().unwrap(), "Part 1 Part 2");
        }
    }
}

#[cfg(test)]
mod error_tests {
    use crate::api::Error;
    use crate::application::config::ProcessingError;

    mod error_construction_tests {
        use super::*;

        #[test]
        fn test_configuration_error() {
            let error = Error::Configuration("Invalid buffer size".to_string());
            match error {
                Error::Configuration(msg) => assert_eq!(msg, "Invalid buffer size"),
                _ => panic!("Expected Configuration error"),
            }

            // Test Display trait
            let error = Error::Configuration("Test config error".to_string());
            let display = format!("{}", error);
            assert_eq!(display, "Configuration error: Test config error");
        }

        #[test]
        fn test_invalid_language_error() {
            let error = Error::InvalidLanguage("xyz".to_string());
            match error {
                Error::InvalidLanguage(lang) => assert_eq!(lang, "xyz"),
                _ => panic!("Expected InvalidLanguage error"),
            }

            // Test Display trait
            let error = Error::InvalidLanguage("unknown".to_string());
            let display = format!("{}", error);
            assert_eq!(display, "Invalid language: unknown");
        }

        #[test]
        fn test_processing_error_from() {
            // Test conversion from ProcessingError
            let proc_error = ProcessingError::TextTooLarge {
                size: 1000,
                max: 500,
            };
            let error: Error = proc_error.into();
            match error {
                Error::Processing(ProcessingError::TextTooLarge { size, max }) => {
                    assert_eq!(size, 1000);
                    assert_eq!(max, 500);
                }
                _ => panic!("Expected Processing error with TextTooLarge"),
            }

            // Test with different ProcessingError variants
            let proc_error = ProcessingError::InvalidConfig {
                reason: "Test reason".to_string(),
            };
            let error: Error = proc_error.into();
            match error {
                Error::Processing(ProcessingError::InvalidConfig { reason }) => {
                    assert_eq!(reason, "Test reason");
                }
                _ => panic!("Expected Processing error with InvalidConfig"),
            }
        }

        #[test]
        fn test_infrastructure_error() {
            let error = Error::Infrastructure("File not found".to_string());
            match error {
                Error::Infrastructure(msg) => assert_eq!(msg, "File not found"),
                _ => panic!("Expected Infrastructure error"),
            }

            // Test Display trait
            let error = Error::Infrastructure("Network timeout".to_string());
            let display = format!("{}", error);
            assert_eq!(display, "Infrastructure error: Network timeout");
        }

        #[test]
        fn test_invalid_input_error() {
            let error = Error::InvalidInput("Null bytes in input".to_string());
            match error {
                Error::InvalidInput(msg) => assert_eq!(msg, "Null bytes in input"),
                _ => panic!("Expected InvalidInput error"),
            }

            // Test Display trait
            let error = Error::InvalidInput("Binary data".to_string());
            let display = format!("{}", error);
            assert_eq!(display, "Invalid input: Binary data");
        }

        #[test]
        fn test_unsupported_error() {
            let error = Error::Unsupported("Custom delimiters".to_string());
            match error {
                Error::Unsupported(feature) => assert_eq!(feature, "Custom delimiters"),
                _ => panic!("Expected Unsupported error"),
            }

            // Test Display trait
            let error = Error::Unsupported("Streaming mode".to_string());
            let display = format!("{}", error);
            assert_eq!(display, "Feature not supported: Streaming mode");
        }

        #[test]
        fn test_error_debug_impl() {
            // Test Debug implementation for each variant
            let errors = vec![
                Error::Configuration("config issue".to_string()),
                Error::InvalidLanguage("xyz".to_string()),
                Error::Processing(ProcessingError::Utf8Error { position: 42 }),
                Error::Infrastructure("io error".to_string()),
                Error::InvalidInput("bad input".to_string()),
                Error::Unsupported("feature x".to_string()),
            ];

            for error in errors {
                let debug_str = format!("{:?}", error);
                assert!(!debug_str.is_empty());
                // Debug format should contain the variant name
                match &error {
                    Error::Configuration(_) => assert!(debug_str.contains("Configuration")),
                    Error::InvalidLanguage(_) => assert!(debug_str.contains("InvalidLanguage")),
                    Error::Processing(_) => assert!(debug_str.contains("Processing")),
                    Error::Infrastructure(_) => assert!(debug_str.contains("Infrastructure")),
                    Error::InvalidInput(_) => assert!(debug_str.contains("InvalidInput")),
                    Error::Unsupported(_) => assert!(debug_str.contains("Unsupported")),
                }
            }
        }
    }

    mod error_behavior_tests {
        use super::*;

        #[test]
        fn test_error_is_send_and_sync() {
            // Verify that Error implements Send and Sync
            fn assert_send<T: Send>() {}
            fn assert_sync<T: Sync>() {}

            assert_send::<Error>();
            assert_sync::<Error>();
        }

        #[test]
        fn test_processing_error_display() {
            // Test display formatting for processing errors
            let error = Error::Processing(ProcessingError::TextTooLarge {
                size: 1000,
                max: 500,
            });
            let display = format!("{}", error);
            assert!(display.contains("Processing error"));

            let error = Error::Processing(ProcessingError::ChunkingError {
                reason: "Test error".to_string(),
            });
            let display = format!("{}", error);
            assert!(display.contains("Processing error"));
        }

        #[test]
        fn test_error_source_chain() {
            // Test that error source chain works correctly
            let proc_error = ProcessingError::Utf8Error { position: 100 };
            let error: Error = proc_error.into();

            // Check that we can access the source
            use std::error::Error as StdError;
            if let Some(source) = error.source() {
                // ProcessingError should be the source
                assert!(source.is::<ProcessingError>());
            }
        }

        #[test]
        fn test_result_type_usage() {
            // Test the Result type alias
            use crate::api::Result;

            fn returns_ok() -> Result<String> {
                Ok("Success".to_string())
            }

            fn returns_err() -> Result<String> {
                Err(Error::InvalidInput("Test error".to_string()))
            }

            assert!(returns_ok().is_ok());
            assert!(returns_err().is_err());

            match returns_err() {
                Err(Error::InvalidInput(msg)) => assert_eq!(msg, "Test error"),
                _ => panic!("Unexpected result"),
            }
        }

        #[test]
        fn test_error_messages_formatting() {
            // Test that error messages are properly formatted
            let test_cases = vec![
                (
                    Error::Configuration("missing required field: threshold".to_string()),
                    "Configuration error: missing required field: threshold",
                ),
                (
                    Error::InvalidLanguage("unsupported language code: 'xyz'".to_string()),
                    "Invalid language: unsupported language code: 'xyz'",
                ),
                (
                    Error::Infrastructure("failed to open file: /tmp/test.txt".to_string()),
                    "Infrastructure error: failed to open file: /tmp/test.txt",
                ),
                (
                    Error::InvalidInput("input contains null bytes at position 42".to_string()),
                    "Invalid input: input contains null bytes at position 42",
                ),
                (
                    Error::Unsupported(
                        "parallel processing not available on this platform".to_string(),
                    ),
                    "Feature not supported: parallel processing not available on this platform",
                ),
            ];

            for (error, expected) in test_cases {
                assert_eq!(format!("{}", error), expected);
            }
        }

        #[test]
        fn test_error_with_empty_messages() {
            // Test errors with empty messages
            let error = Error::Configuration("".to_string());
            assert_eq!(format!("{}", error), "Configuration error: ");

            let error = Error::InvalidLanguage("".to_string());
            assert_eq!(format!("{}", error), "Invalid language: ");
        }

        #[test]
        fn test_error_with_unicode_messages() {
            // Test errors with Unicode messages
            let error = Error::InvalidInput("Invalid character: '🚀' at position 10".to_string());
            let display = format!("{}", error);
            assert!(display.contains("🚀"));

            let error = Error::Infrastructure("Failed to read file: テスト.txt".to_string());
            let display = format!("{}", error);
            assert!(display.contains("テスト.txt"));
        }

        #[test]
        fn test_error_conversion_chain() {
            // Test that ProcessingError -> Error conversion preserves information
            let processing_errors = vec![
                ProcessingError::TextTooLarge {
                    size: 1000,
                    max: 500,
                },
                ProcessingError::Utf8Error { position: 42 },
                ProcessingError::InvalidConfig {
                    reason: "Test config".to_string(),
                },
                ProcessingError::ChunkingError {
                    reason: "test error".to_string(),
                },
                ProcessingError::Utf8BoundaryError { position: 100 },
                ProcessingError::WordBoundaryError { position: 200 },
            ];

            for proc_err in processing_errors {
                let _proc_err_display = format!("{}", proc_err);
                let api_err: Error = proc_err.into();
                let api_err_display = format!("{}", api_err);

                // API error should contain processing error message
                assert!(api_err_display.contains("Processing error"));
            }
        }
    }
}

#[cfg(test)]
mod language_tests {
    use crate::api::Language;

    mod language_construction_tests {
        use super::*;

        #[test]
        fn test_language_variants() {
            // Test that all variants are distinct
            let english = Language::English;
            let japanese = Language::Japanese;

            assert_ne!(english, japanese);
            assert_eq!(english, Language::English);
            assert_eq!(japanese, Language::Japanese);
        }

        #[test]
        fn test_default_language() {
            // Test that default is English
            let default_lang = Language::default();
            assert_eq!(default_lang, Language::English);
        }

        #[test]
        fn test_from_code_english() {
            // Test various English codes
            let test_cases = vec![
                "en", "EN", "En", "eng", "ENG", "Eng", "english", "ENGLISH", "English",
                "eNgLiSh", // Mixed case
            ];

            for code in test_cases {
                let lang = Language::from_code(code);
                assert_eq!(lang, Language::English, "Failed for code: {}", code);
            }
        }

        #[test]
        fn test_from_code_japanese() {
            // Test various Japanese codes
            let test_cases = vec![
                "ja", "JA", "Ja", "jpn", "JPN", "Jpn", "japanese", "JAPANESE", "Japanese",
                "jApAnEsE", // Mixed case
            ];

            for code in test_cases {
                let lang = Language::from_code(code);
                assert_eq!(lang, Language::Japanese, "Failed for code: {}", code);
            }
        }

        #[test]
        fn test_from_code_unknown() {
            // Test unknown codes default to English
            let test_cases = vec![
                "",
                "unknown",
                "xyz",
                "de",
                "fr",
                "es",
                "zh",
                "ko",
                "ru",
                "ar",
                "hi",
                "123",
                "!@#",
                "english-US", // Not exact match
                "ja-JP",      // Not exact match
            ];

            for code in test_cases {
                let lang = Language::from_code(code);
                assert_eq!(lang, Language::English, "Failed for code: {}", code);
            }
        }

        #[test]
        fn test_from_code_with_whitespace() {
            // Test codes with whitespace - should trim and match correctly
            assert_eq!(Language::from_code(" en "), Language::English);
            assert_eq!(Language::from_code("\ten\t"), Language::English);
            assert_eq!(Language::from_code("\nja\n"), Language::Japanese);
            assert_eq!(Language::from_code("en "), Language::English);
            assert_eq!(Language::from_code(" ja"), Language::Japanese);

            // Test with multiple whitespace types
            assert_eq!(Language::from_code("  \t english \n "), Language::English);
            assert_eq!(Language::from_code("\r\njapanese\r\n"), Language::Japanese);
        }

        #[test]
        fn test_language_code() {
            // Test code getter
            assert_eq!(Language::English.code(), "en");
            assert_eq!(Language::Japanese.code(), "ja");
        }

        #[test]
        fn test_language_name() {
            // Test name getter
            assert_eq!(Language::English.name(), "English");
            assert_eq!(Language::Japanese.name(), "Japanese");
        }

        #[test]
        fn test_display_trait() {
            // Test Display implementation
            assert_eq!(format!("{}", Language::English), "English");
            assert_eq!(format!("{}", Language::Japanese), "Japanese");

            // Test in formatted strings
            let lang = Language::English;
            assert_eq!(format!("Language: {}", lang), "Language: English");
        }

        #[test]
        fn test_debug_trait() {
            // Test Debug implementation
            let debug_en = format!("{:?}", Language::English);
            assert!(debug_en.contains("English"));

            let debug_ja = format!("{:?}", Language::Japanese);
            assert!(debug_ja.contains("Japanese"));
        }

        #[test]
        fn test_clone_and_copy() {
            // Test that Language implements Clone and Copy
            let original = Language::Japanese;
            let cloned = original;
            let copied = original; // Copy

            assert_eq!(original, cloned);
            assert_eq!(original, copied);
            assert_eq!(cloned, copied);
        }

        #[test]
        fn test_language_equality() {
            // Test PartialEq and Eq
            let en1 = Language::English;
            let en2 = Language::English;
            let ja1 = Language::Japanese;
            let ja2 = Language::Japanese;

            // Same language should be equal
            assert_eq!(en1, en2);
            assert_eq!(ja1, ja2);

            // Different languages should not be equal
            assert_ne!(en1, ja1);
            assert_ne!(en2, ja2);

            // Test with default
            assert_eq!(Language::default(), Language::English);
            assert_ne!(Language::default(), Language::Japanese);
        }
    }

    mod language_behavior_tests {
        use super::*;

        #[test]
        fn test_code_consistency() {
            // Test that from_code and code are consistent
            let languages = vec![Language::English, Language::Japanese];

            for lang in languages {
                let code = lang.code();
                let reconstructed = Language::from_code(code);
                assert_eq!(lang, reconstructed);
            }
        }

        #[test]
        fn test_language_is_send_and_sync() {
            // Verify that Language implements Send and Sync
            fn assert_send<T: Send>() {}
            fn assert_sync<T: Sync>() {}

            assert_send::<Language>();
            assert_sync::<Language>();
        }

        #[test]
        fn test_language_size() {
            // Language enum should be small (single byte)
            use std::mem::size_of;
            assert_eq!(size_of::<Language>(), 1);
        }

        #[test]
        fn test_language_in_collections() {
            // Language can be used in vectors and other collections
            let languages = vec![Language::English, Language::Japanese];
            assert!(languages.contains(&Language::English));
            assert!(languages.contains(&Language::Japanese));

            // Can be used in match expressions
            for lang in &languages {
                match lang {
                    Language::English => assert_eq!(lang.code(), "en"),
                    Language::Japanese => assert_eq!(lang.code(), "ja"),
                }
            }
        }

        #[test]
        fn test_from_code_special_cases() {
            // Test some edge cases
            assert_eq!(Language::from_code("eN"), Language::English);
            assert_eq!(Language::from_code("jA"), Language::Japanese);
            assert_eq!(Language::from_code("EnG"), Language::English);
            assert_eq!(Language::from_code("JpN"), Language::Japanese);

            // Numbers and special characters
            assert_eq!(Language::from_code("en123"), Language::English); // defaults
            assert_eq!(Language::from_code("123ja"), Language::English); // defaults
            assert_eq!(Language::from_code("en-US"), Language::English); // defaults
            assert_eq!(Language::from_code("ja_JP"), Language::English); // defaults
        }

        #[test]
        fn test_language_conversion_roundtrip() {
            // Test multiple roundtrips
            let original = Language::Japanese;

            // First roundtrip
            let code1 = original.code();
            let lang1 = Language::from_code(code1);
            assert_eq!(original, lang1);

            // Second roundtrip
            let code2 = lang1.code();
            let lang2 = Language::from_code(code2);
            assert_eq!(lang1, lang2);

            // Name roundtrip (lowercase)
            let name = original.name();
            let lang_from_name = Language::from_code(&name.to_lowercase());
            assert_eq!(original, lang_from_name);
        }

        #[test]
        fn test_language_formatting_in_errors() {
            // Test how Language looks in error messages
            let lang = Language::Japanese;
            let error_msg = format!("Unsupported feature for language: {}", lang);
            assert_eq!(error_msg, "Unsupported feature for language: Japanese");
        }

        #[test]
        fn test_from_code_unicode() {
            // Test with Unicode input (should default to English)
            assert_eq!(Language::from_code("英語"), Language::English);
            assert_eq!(Language::from_code("日本語"), Language::English);
            assert_eq!(Language::from_code("🇬🇧"), Language::English);
            assert_eq!(Language::from_code("🇯🇵"), Language::English);
        }

        #[test]
        fn test_pattern_matching() {
            // Test that pattern matching works correctly
            let lang = Language::English;

            let result = match lang {
                Language::English => "en",
                Language::Japanese => "ja",
            };

            assert_eq!(result, "en");

            // Test exhaustive matching
            fn get_greeting(lang: Language) -> &'static str {
                match lang {
                    Language::English => "Hello",
                    Language::Japanese => "こんにちは",
                }
            }

            assert_eq!(get_greeting(Language::English), "Hello");
            assert_eq!(get_greeting(Language::Japanese), "こんにちは");
        }
    }
}
