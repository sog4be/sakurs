//! File pattern resolution using glob

use anyhow::{Context, Result};
use glob::glob;
use std::path::PathBuf;

/// Resolve file patterns to actual file paths
pub fn resolve_patterns(patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        let paths = glob(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?;

        for path_result in paths {
            let path =
                path_result.with_context(|| format!("Error resolving pattern: {}", pattern))?;

            if path.is_file() {
                files.push(path);
            }
        }
    }

    if files.is_empty() {
        anyhow::bail!("No files found matching the provided patterns");
    }

    // Remove duplicates and sort
    files.sort();
    files.dedup();

    Ok(files)
}
