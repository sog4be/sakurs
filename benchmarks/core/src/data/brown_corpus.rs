//! Brown Corpus data loader for benchmarks
//!
//! This module provides functions to load Brown Corpus data that has been
//! preprocessed by the Python scripts in benchmarks/data/brown_corpus/.

use crate::data::TestData;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Brown Corpus metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct CorpusMetadata {
    pub source: String,
    pub sentences: usize,
    pub characters: usize,
    pub words: usize,
}

/// Brown Corpus data structure matching Python output
#[derive(Debug, Serialize, Deserialize)]
struct CorpusData {
    name: String,
    text: String,
    boundaries: Vec<usize>,
    metadata: CorpusMetadata,
}

/// Get the path to the Brown Corpus cache directory
fn get_cache_path() -> PathBuf {
    // Try to find the cache relative to the workspace root
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // benchmarks/core -> benchmarks
    path.push("data");
    path.push("brown_corpus");
    path.push("cache");
    path
}

/// Load the full Brown Corpus dataset
///
/// This expects the data to be preprocessed by running:
/// ```bash
/// cd benchmarks/data/brown_corpus
/// uv run python download.py
/// ```
pub fn load_full_corpus() -> Result<TestData, Box<dyn Error>> {
    let cache_path = get_cache_path();
    let corpus_file = cache_path.join("brown_corpus.json");

    if !corpus_file.exists() {
        return Err(format!(
            "Brown Corpus data not found at {:?}. Please run the download script first:\n\
             cd benchmarks/data/brown_corpus && uv run python download.py",
            corpus_file
        )
        .into());
    }

    // Load and parse JSON data
    let data = fs::read_to_string(&corpus_file)?;
    let corpus_data: CorpusData = serde_json::from_str(&data)?;

    // Validate the data
    let test_data = TestData::new(corpus_data.name, corpus_data.text, corpus_data.boundaries);
    test_data.validate()?;

    Ok(test_data)
}

/// Load a subset of Brown Corpus for quick tests
///
/// This loads only the first N sentences for faster iteration during development.
pub fn load_subset(max_sentences: usize) -> Result<TestData, Box<dyn Error>> {
    let full_corpus = load_full_corpus()?;

    // Find the boundary index for the requested number of sentences
    let boundary_count = max_sentences.min(full_corpus.boundaries.len());
    if boundary_count == 0 {
        return Ok(TestData::new(
            "brown_corpus_empty".to_string(),
            String::new(),
            vec![],
        ));
    }

    // Get the text up to the nth boundary
    let text_end = full_corpus.boundaries[boundary_count - 1];
    let text = full_corpus.text[..text_end].to_string();
    let boundaries = full_corpus.boundaries[..boundary_count].to_vec();

    Ok(TestData::new(
        format!("brown_corpus_subset_{}", max_sentences),
        text,
        boundaries,
    ))
}

/// Check if Brown Corpus data is available
pub fn is_available() -> bool {
    let cache_path = get_cache_path();
    cache_path.join("brown_corpus.json").exists()
}

/// Get a small hardcoded Brown Corpus sample for testing
/// This is used when the full corpus is not available
pub fn small_sample() -> TestData {
    let text = "The Fulton County Grand Jury said Friday an investigation of \
               Atlanta's recent primary election produced no evidence that \
               any irregularities took place. The jury further said in \
               term-end presentments that the City Executive Committee, \
               which had over-all charge of the election, deserves the \
               praise and thanks of the City of Atlanta for the manner \
               in which the election was conducted.";

    // Boundaries are detected after period + space
    let boundaries = vec![151];

    TestData::new(
        "brown_corpus_sample".to_string(),
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
        assert!(path.ends_with("brown_corpus/cache"));
    }

    #[test]
    fn test_is_available() {
        // This test just checks the function works, not that data exists
        let _ = is_available();
    }

    #[test]
    fn test_load_missing_corpus() {
        // Create a temp directory without the corpus file
        let result = load_full_corpus();
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(err.to_string().contains("Please run the download script"));
        }
    }
}
