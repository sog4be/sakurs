//! Basic tests for sakurs-engine

use sakurs_engine::*;

#[test]
fn test_engine_config_creation() {
    let config = EngineConfig::default();
    assert_eq!(config.parallel_threshold, 100_000);

    let streaming = EngineConfig::streaming();
    assert_eq!(streaming.threads, Some(1));

    let fast = EngineConfig::fast();
    match fast.chunk_policy {
        ChunkPolicy::Fixed { size } => assert_eq!(size, 512 * 1024),
        _ => panic!("Expected fixed chunk policy"),
    }
}

#[test]
fn test_execution_mode_selection() {
    use sakurs_engine::executor::auto_select;

    assert_eq!(auto_select(500, 100_000), ExecutionMode::Sequential);
    assert_eq!(auto_select(10_000, 100_000), ExecutionMode::Sequential);

    #[cfg(feature = "parallel")]
    assert_eq!(auto_select(200_000, 100_000), ExecutionMode::Parallel);
}

#[test]
fn test_sequential_executor() {
    use sakurs_engine::executor::{Executor, SequentialExecutor};
    use sakurs_engine::language::EnglishRules;

    let executor = SequentialExecutor;
    let rules = EnglishRules;

    let text = "Hello world. This is a test.";
    let boundaries = executor.process(text, &rules).unwrap();

    assert_eq!(boundaries.len(), 2);
    assert_eq!(boundaries[0].byte_offset, 12); // After "Hello world."
    assert_eq!(boundaries[1].byte_offset, 28); // After "This is a test."
}

#[test]
fn test_sentence_processor_builder() {
    let processor = SentenceProcessorBuilder::new()
        .language("en")
        .threads(Some(4))
        .parallel_threshold(50_000)
        .build()
        .unwrap();

    let boundaries = processor.process("Test.").unwrap();
    assert_eq!(boundaries.len(), 1);
}

#[test]
fn test_english_rules() {
    use sakurs_engine::language::EnglishRules;

    let rules = EnglishRules;

    assert_eq!(rules.is_terminator('.'), true);
    assert_eq!(rules.is_terminator('a'), false);

    assert_eq!(rules.get_enclosure_pair('('), Some((0, true)));
    assert_eq!(rules.get_enclosure_pair(')'), Some((0, false)));
    assert_eq!(rules.get_enclosure_pair('a'), None);

    assert_eq!(rules.is_abbreviation("Dr", 2), true);
    assert_eq!(rules.is_abbreviation("Hello", 5), false);
}

#[test]
fn test_japanese_rules() {
    use sakurs_engine::language::JapaneseRules;

    let rules = JapaneseRules;

    assert_eq!(rules.is_terminator('。'), true);
    assert_eq!(rules.is_terminator('！'), true);
    assert_eq!(rules.is_terminator('a'), false);

    assert_eq!(rules.get_enclosure_pair('「'), Some((0, true)));
    assert_eq!(rules.get_enclosure_pair('」'), Some((0, false)));
}

#[test]
fn test_chunk_manager() {
    use sakurs_engine::chunker::ChunkManager;

    let manager = ChunkManager::new(ChunkPolicy::Fixed { size: 10 });
    let text = "Hello world. This is a test.";
    let chunks = manager.chunk_text(text).unwrap();

    assert!(chunks.len() > 1);

    // Verify chunks cover the entire text
    let mut total_len = 0;
    for chunk in &chunks {
        total_len += chunk.len;
    }
    assert_eq!(total_len, text.len());
}
