//! Tests for the new API implementation

use sakurs_engine::{
    ExecutionMode, Input, ProcessorConfig, SentenceProcessor, SentenceProcessorBuilder,
};

#[test]
fn test_basic_sentence_processing() {
    let processor = SentenceProcessor::new().expect("Failed to create processor");
    let input = Input::from_text("Hello world. How are you?");
    let output = processor.process(input).expect("Failed to process text");

    assert_eq!(output.boundaries.len(), 2);
    assert!(output.metadata.processing_time_ms > 0.0);
    assert!(output.metadata.bytes_processed > 0);
}

#[test]
fn test_adaptive_mode() {
    let processor = SentenceProcessor::with_language("en").expect("Failed to create processor");
    let input = Input::from_text("This is a test. This is another test.");
    let output = processor.process(input).expect("Failed to process text");

    // Small text should use Sequential mode
    assert_eq!(output.metadata.execution_mode, ExecutionMode::Sequential);
    assert_eq!(output.boundaries.len(), 2);
}

#[test]
fn test_explicit_execution_modes() {
    let processor = SentenceProcessor::with_language("en").expect("Failed to create processor");
    let input = Input::from_text("Test sentence. Another sentence.");

    // Test sequential mode
    let sequential_output = processor
        .process_with_mode(input.clone(), ExecutionMode::Sequential)
        .expect("Failed to process with sequential mode");
    assert_eq!(
        sequential_output.metadata.execution_mode,
        ExecutionMode::Sequential
    );

    // Test adaptive mode
    let adaptive_output = processor
        .process_with_mode(
            Input::from_text("Test sentence. Another sentence."),
            ExecutionMode::Adaptive,
        )
        .expect("Failed to process with adaptive mode");

    // Results should be identical
    assert_eq!(
        sequential_output.boundaries.len(),
        adaptive_output.boundaries.len()
    );
}

#[test]
fn test_processor_builder() {
    let processor = SentenceProcessorBuilder::new()
        .language("en")
        .execution_mode(ExecutionMode::Sequential)
        .threads(Some(1))
        .build()
        .expect("Failed to build processor");

    let output = processor
        .process_text("Hello. World.")
        .expect("Failed to process text");

    assert_eq!(output.boundaries.len(), 2);
    assert_eq!(output.metadata.execution_mode, ExecutionMode::Sequential);
}

#[test]
fn test_config_presets() {
    // Test fast preset
    let fast_config = ProcessorConfig::fast("en");
    let processor =
        SentenceProcessor::with_config(fast_config).expect("Failed to create fast processor");

    let output = processor
        .process_text("Fast processing test.")
        .expect("Failed to process with fast config");

    assert_eq!(output.boundaries.len(), 1);
    assert!(output.metadata.bytes_per_second > 0.0);

    // Test streaming preset
    let streaming_config = ProcessorConfig::streaming("en");
    let processor = SentenceProcessor::with_config(streaming_config)
        .expect("Failed to create streaming processor");

    let output = processor
        .process_text("Streaming test.")
        .expect("Failed to process with streaming config");

    assert_eq!(output.boundaries.len(), 1);
    assert_eq!(output.metadata.execution_mode, ExecutionMode::Streaming);
}

#[test]
fn test_input_abstractions() {
    let processor = SentenceProcessor::new().expect("Failed to create processor");

    // Test text input
    let text_input = Input::from_text("Text input test.".to_string());
    let text_output = processor
        .process(text_input)
        .expect("Failed to process text input");
    assert_eq!(text_output.boundaries.len(), 1);

    // Test static str input
    let str_input = Input::from_text_ref("Static str test.");
    let str_output = processor
        .process(str_input)
        .expect("Failed to process str input");
    assert_eq!(str_output.boundaries.len(), 1);

    // Test byte input
    let bytes_input = Input::from_bytes("Bytes input test.".as_bytes().to_vec());
    let bytes_output = processor
        .process(bytes_input)
        .expect("Failed to process bytes input");
    assert_eq!(bytes_output.boundaries.len(), 1);
}

#[test]
fn test_metadata_collection() {
    let processor = SentenceProcessor::new().expect("Failed to create processor");
    let output = processor
        .process_text("Metadata test. Another sentence.")
        .expect("Failed to process text");

    // Check metadata fields
    assert!(output.metadata.processing_time_ms >= 0.0);
    assert_eq!(
        output.metadata.bytes_processed,
        "Metadata test. Another sentence.".len()
    );
    assert!(output.metadata.bytes_per_second > 0.0);
    assert!(output.metadata.thread_efficiency > 0.0 && output.metadata.thread_efficiency <= 1.0);

    // Sequential mode should not have chunks
    if output.metadata.execution_mode == ExecutionMode::Sequential {
        assert!(output.metadata.chunks_processed.is_none());
    }
}
