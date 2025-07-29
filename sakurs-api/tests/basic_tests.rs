//! Basic tests for sakurs-api

use sakurs_api::*;

#[test]
fn test_input_text_processing() {
    let input = Input::Text("Hello world.".to_string());
    let text = input.read_text().unwrap();
    assert_eq!(text, "Hello world.");
}

#[test]
fn test_input_bytes_processing() {
    let bytes = b"Hello world.".to_vec();
    let input = Input::Bytes(bytes);
    let text = input.read_text().unwrap();
    assert_eq!(text, "Hello world.");
}

#[test]
fn test_config_builder() {
    let _config = Config::builder()
        .language("en")
        .unwrap()
        .threads(Some(4))
        .chunk_size(1024)
        .build()
        .unwrap();

    // Config fields are private, just verify it builds successfully
}

#[test]
fn test_config_presets() {
    let streaming = Config::streaming();
    let fast = Config::fast();
    let balanced = Config::balanced();

    // Just verify they can be created
    let _ = streaming;
    let _ = fast;
    let _ = balanced;
}

#[test]
fn test_process_text_convenience() {
    let output = process_text("Hello world. This is a test.").unwrap();

    assert!(!output.boundaries.is_empty());
    assert_eq!(output.metadata.total_bytes, 28);
    // Processing time should be recorded
    let _ = output.metadata.processing_time_ms;
}

#[test]
#[cfg(feature = "serde")]
fn test_dto_serialization() {
    use serde_json;

    let boundary = Boundary {
        byte_offset: 10,
        char_offset: 10,
        kind: "strong".to_string(),
    };

    let json = serde_json::to_string(&boundary).unwrap();
    let deserialized: Boundary = serde_json::from_str(&json).unwrap();

    assert_eq!(boundary.byte_offset, deserialized.byte_offset);
    assert_eq!(boundary.kind, deserialized.kind);
}

#[test]
#[cfg(feature = "serde")]
fn test_output_serialization() {
    use serde_json;

    let output = Output {
        boundaries: vec![Boundary {
            byte_offset: 12,
            char_offset: 12,
            kind: "strong".to_string(),
        }],
        metadata: sakurs_api::dto::ProcessingMetadata {
            total_bytes: 100,
            processing_time_ms: 5,
            mode: "sequential".to_string(),
            thread_count: 1,
        },
    };

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: Output = serde_json::from_str(&json).unwrap();

    assert_eq!(output.boundaries.len(), deserialized.boundaries.len());
    assert_eq!(
        output.metadata.total_bytes,
        deserialized.metadata.total_bytes
    );
}

#[test]
fn test_error_conversions() {
    use std::io;

    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let api_error: ApiError = io_error.into();

    match api_error {
        ApiError::Io(_) => (), // Expected
        _ => panic!("Wrong error type"),
    }
}
