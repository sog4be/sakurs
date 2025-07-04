//! Path resolution utilities for benchmark data

use crate::error::{BenchmarkError, BenchmarkResult};
use std::env;
use std::path::PathBuf;

/// Get the root directory of the benchmarks crate
pub fn benchmarks_root() -> BenchmarkResult<PathBuf> {
    // Try multiple strategies to find the benchmarks root

    // Strategy 1: Check if we're running from the benchmarks/core directory
    if let Ok(current_dir) = env::current_dir() {
        if current_dir.ends_with("benchmarks/core") {
            return Ok(current_dir.parent().unwrap().to_path_buf());
        }

        // Strategy 2: Look for benchmarks directory in current path
        let mut path = current_dir.as_path();
        while let Some(parent) = path.parent() {
            let benchmarks_path = parent.join("benchmarks");
            if benchmarks_path.exists() && benchmarks_path.join("core").exists() {
                return Ok(benchmarks_path);
            }
            path = parent;
        }
    }

    // Strategy 3: Use CARGO_MANIFEST_DIR if available
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        if manifest_path.ends_with("benchmarks/core") {
            return Ok(manifest_path.parent().unwrap().to_path_buf());
        }
    }

    // Strategy 4: Use relative path from binary location
    if let Ok(exe_path) = env::current_exe() {
        // Go up from target/debug or target/release
        if let Some(target_dir) = exe_path
            .ancestors()
            .find(|p| p.file_name() == Some("target".as_ref()))
        {
            if let Some(workspace_root) = target_dir.parent() {
                let benchmarks_path = workspace_root.join("benchmarks");
                if benchmarks_path.exists() {
                    return Ok(benchmarks_path);
                }
            }
        }
    }

    Err(BenchmarkError::Config {
        message: "Could not determine benchmarks root directory. \
                 Please run from the sakurs workspace root or benchmarks directory."
            .to_string(),
    })
}

/// Get the data directory for benchmarks
pub fn data_dir() -> BenchmarkResult<PathBuf> {
    Ok(benchmarks_root()?.join("data"))
}

/// Get the cache directory for a specific corpus
pub fn corpus_cache_dir(corpus_name: &str) -> BenchmarkResult<PathBuf> {
    Ok(data_dir()?.join(corpus_name).join("cache"))
}

/// Get the expected path for a corpus data file
pub fn corpus_data_path(corpus_name: &str, filename: &str) -> BenchmarkResult<PathBuf> {
    Ok(corpus_cache_dir(corpus_name)?.join(filename))
}

/// Check if a corpus data file exists
pub fn corpus_exists(corpus_name: &str, filename: &str) -> bool {
    corpus_data_path(corpus_name, filename)
        .map(|p| p.exists())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmarks_root_from_test() {
        // When running tests, we should be able to find the benchmarks root
        let root = benchmarks_root();
        assert!(
            root.is_ok(),
            "Failed to find benchmarks root: {:?}",
            root.err()
        );

        let root_path = root.unwrap();
        assert!(
            root_path.join("core").exists(),
            "benchmarks/core should exist"
        );
        assert!(
            root_path.join("data").exists(),
            "benchmarks/data should exist"
        );
    }

    #[test]
    fn test_data_paths() {
        let data_dir = data_dir().unwrap();
        assert!(data_dir.exists(), "Data directory should exist");

        let cache_dir = corpus_cache_dir("brown_corpus").unwrap();
        assert_eq!(cache_dir.file_name().unwrap(), "cache");

        let data_path = corpus_data_path("brown_corpus", "test.json").unwrap();
        assert_eq!(data_path.file_name().unwrap(), "test.json");
    }
}
