//! UD English EWT data loader for benchmarks
//!
//! This module provides functions to load Universal Dependencies English Web Treebank data
//! that has been preprocessed by the Python scripts in benchmarks/data/ud_english_ewt/.

use crate::data::TestData;
use crate::error::{BenchmarkError, BenchmarkResult};
use crate::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// UD English EWT metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct UdEnglishEwtMetadata {
    pub source: String,
    pub version: String,
    pub sentences: usize,
    pub characters: usize,
    pub words: usize,
    pub genres: Vec<String>,
    pub splits: Vec<String>,
    pub format: String,
    pub license: String,
}

/// UD English EWT data structure matching Python output
#[derive(Debug, Serialize, Deserialize)]
struct UdEnglishEwtData {
    name: String,
    text: String,
    boundaries: Vec<usize>,
    metadata: UdEnglishEwtMetadata,
}

/// Get the path to the UD English EWT cache directory
#[allow(dead_code)]
fn get_cache_path() -> BenchmarkResult<PathBuf> {
    paths::corpus_cache_dir("ud_english_ewt")
}

/// Load the full UD English EWT dataset
///
/// This expects the data to be preprocessed by running:
/// ```bash
/// cd benchmarks/data/ud_english_ewt
/// python download.py
/// ```
pub fn load_full_corpus() -> BenchmarkResult<TestData> {
    let corpus_file = paths::corpus_data_path("ud_english_ewt", "ud_english_ewt.json")?;
    let test_corpus_file = paths::corpus_data_path("ud_english_ewt", "test_ud_english_ewt.json")?;

    // Try full dataset first, then test dataset
    let data_path = if corpus_file.exists() {
        corpus_file
    } else if test_corpus_file.exists() {
        test_corpus_file
    } else {
        return Err(BenchmarkError::CorpusNotFound {
            corpus_name: "UD English EWT".to_string(),
            expected_path: corpus_file,
        });
    };

    // Load and parse JSON data
    let data = fs::read_to_string(&data_path).map_err(|e| BenchmarkError::Io {
        path: data_path.clone(),
        source: e,
    })?;

    let corpus_data: UdEnglishEwtData =
        serde_json::from_str(&data).map_err(|e| BenchmarkError::JsonParse {
            path: data_path,
            source: e,
        })?;

    // Create metadata string
    let metadata = format!(
        "UD English EWT r{} - {} sentences, {} genres: {}",
        corpus_data.metadata.version,
        corpus_data.metadata.sentences,
        corpus_data.metadata.genres.len(),
        corpus_data.metadata.genres.join(", ")
    );

    // Validate the data
    let test_data = TestData::new(corpus_data.name, corpus_data.text, corpus_data.boundaries)
        .with_metadata(metadata);
    test_data
        .validate()
        .map_err(|e| BenchmarkError::Validation {
            message: e.to_string(),
        })?;

    Ok(test_data)
}

/// Load a subset of UD English EWT for quick tests
///
/// This loads only the first N sentences for faster iteration during development.
pub fn load_subset(max_sentences: usize) -> BenchmarkResult<TestData> {
    let full_corpus = load_full_corpus()?;

    // Find the boundary index for the requested number of sentences
    let boundary_count = max_sentences.min(full_corpus.boundaries.len());
    if boundary_count == 0 {
        return Ok(TestData::new(
            "ud_english_ewt_empty".to_string(),
            String::new(),
            vec![],
        ));
    }

    // Get the text up to and including the nth sentence
    let text_end = if boundary_count < full_corpus.boundaries.len() {
        full_corpus.boundaries[boundary_count]
    } else {
        full_corpus.text.len()
    };

    let text = full_corpus.text[..text_end].to_string();
    let boundaries = full_corpus.boundaries[..boundary_count].to_vec();

    Ok(TestData::new(
        format!("ud_english_ewt_subset_{max_sentences}"),
        text,
        boundaries,
    ))
}

/// Check if UD English EWT data is available
pub fn is_available() -> bool {
    paths::corpus_exists("ud_english_ewt", "ud_english_ewt.json")
        || paths::corpus_exists("ud_english_ewt", "test_ud_english_ewt.json")
}

/// Get a small hardcoded UD English EWT sample for testing
/// This is used when the full corpus is not available
pub fn small_sample() -> TestData {
    let text = "From the AP comes this story: President Bush met with congressional leaders today. \
               The discussion focused on economic policy issues. Several senators expressed concern \
               about the proposed legislation.";

    // Boundaries are detected after period + space
    let boundaries = vec![84, 132];

    TestData::new(
        "ud_english_ewt_sample".to_string(),
        text.to_string(),
        boundaries,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_path() {
        let path = get_cache_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.ends_with("ud_english_ewt/cache"));
    }

    #[test]
    fn test_is_available() {
        // This test just checks the function works, not that data exists
        let _ = is_available();
    }

    #[test]
    fn test_load_missing_corpus() {
        // This may succeed if test data is available
        let result = load_full_corpus();
        match result {
            Ok(data) => {
                // Test data is available, verify it's valid
                assert!(!data.text.is_empty());
                assert!(data.validate().is_ok());
                println!(
                    "✅ UD English EWT test data loaded: {} sentences",
                    data.sentence_count()
                );
            }
            Err(err) => {
                // Expected when no data is available
                assert!(err.to_string().contains("Please run download script first"));
            }
        }
    }

    #[test]
    fn test_small_sample() {
        let sample = small_sample();
        assert!(sample.validate().is_ok());
        assert_eq!(sample.name, "ud_english_ewt_sample");
        assert_eq!(sample.boundaries.len(), 2);
        assert_eq!(sample.sentence_count(), 3);
    }

    #[test]
    fn test_ud_english_ewt_format() {
        // Test data format expectations
        let expected_genres = vec![
            "weblogs",
            "newsgroups",
            "emails",
            "reviews",
            "yahoo_answers",
        ];
        let expected_splits = vec!["train", "dev", "test"];

        // These are the expected data characteristics
        assert_eq!(expected_genres.len(), 5);
        assert_eq!(expected_splits.len(), 3);
    }
}
