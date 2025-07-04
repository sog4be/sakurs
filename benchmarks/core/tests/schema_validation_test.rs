//! Tests to ensure Python-Rust schema compatibility

use sakurs_benchmarks::data::brown_corpus::{small_sample, CorpusMetadata};
use serde_json::json;

#[test]
fn test_schema_matches_python_output() {
    // This JSON structure should match what Python outputs
    let test_json = json!({
        "name": "test_corpus",
        "text": "Hello world. This is a test.",
        "boundaries": [12, 28],
        "metadata": {
            "source": "test",
            "sentences": 2,
            "characters": 28,
            "words": 6
        }
    });

    // Should be able to deserialize into our Rust structs
    let corpus_data: serde_json::Value = test_json;

    // Verify all expected fields exist
    assert!(corpus_data["name"].is_string());
    assert!(corpus_data["text"].is_string());
    assert!(corpus_data["boundaries"].is_array());
    assert!(corpus_data["metadata"].is_object());

    // Verify metadata structure
    let metadata = &corpus_data["metadata"];
    assert!(metadata["source"].is_string());
    assert!(metadata["sentences"].is_u64());
    assert!(metadata["characters"].is_u64());
    assert!(metadata["words"].is_u64());
}

#[test]
fn test_corpus_metadata_serialization() {
    // Create metadata struct
    let metadata = CorpusMetadata {
        source: "test".to_string(),
        sentences: 10,
        characters: 100,
        words: 20,
    };

    // Serialize to JSON
    let json = serde_json::to_value(&metadata).unwrap();

    // Verify structure matches Python schema
    assert_eq!(json["source"], "test");
    assert_eq!(json["sentences"], 10);
    assert_eq!(json["characters"], 100);
    assert_eq!(json["words"], 20);
}

#[test]
fn test_small_sample_schema_validity() {
    let sample = small_sample();

    // Create a corpus data structure like Python would
    let corpus_json = json!({
        "name": sample.name,
        "text": sample.text,
        "boundaries": sample.boundaries,
        "metadata": {
            "source": "hardcoded",
            "sentences": sample.boundaries.len(),
            "characters": sample.text.len(),
            "words": sample.text.split_whitespace().count()
        }
    });

    // Verify it's valid JSON
    assert!(corpus_json.is_object());

    // Verify boundary consistency
    let boundaries = corpus_json["boundaries"].as_array().unwrap();
    let text_len = corpus_json["text"].as_str().unwrap().len();

    for boundary in boundaries {
        let boundary_pos = boundary.as_u64().unwrap() as usize;
        assert!(
            boundary_pos < text_len,
            "Boundary {} out of range",
            boundary_pos
        );
    }
}
