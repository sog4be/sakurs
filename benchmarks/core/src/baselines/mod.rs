//! Baseline tool integration for benchmarking comparisons

mod python_env;

use crate::error::{BenchmarkError, BenchmarkResult};
use serde::{Deserialize, Serialize};

/// Benchmark results from a baseline tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResult {
    pub tool: String,
    pub dataset: String,
    pub text_length: usize,
    pub num_sentences: usize,
    pub processing_time_seconds: f64,
    pub sentences_per_second: f64,
    pub characters_per_second: f64,
    pub metrics: BaselineMetrics,
    pub predicted_boundaries: usize,
    pub actual_boundaries: usize,
}

/// Accuracy metrics from baseline tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
}

/// Run NLTK Punkt benchmark
pub fn run_nltk_punkt_benchmark(subset_size: Option<usize>) -> BenchmarkResult<BaselineResult> {
    let benchmarks_root = crate::paths::benchmarks_root()?;

    // Build Python command using environment detection
    let mut cmd = python_env::build_python_command().map_err(|e| BenchmarkError::Config {
        message: format!("Python environment error: {e}"),
    })?;

    // Add script path
    let script_path = benchmarks_root
        .join("baselines")
        .join("nltk_punkt")
        .join("benchmark.py");

    cmd.arg("-m").arg("baselines.nltk_punkt.benchmark");

    // Add subset argument if specified
    if let Some(size) = subset_size {
        cmd.arg("--subset").arg(size.to_string());
    }

    // Set working directory to benchmarks root
    cmd.current_dir(&benchmarks_root);

    // Execute command
    let output = cmd.output().map_err(|e| BenchmarkError::Io {
        path: script_path.clone(),
        source: e,
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BenchmarkError::Config {
            message: format!("NLTK Punkt benchmark failed: {stderr}"),
        });
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| BenchmarkError::JsonParse {
        path: script_path,
        source: e,
    })
}

/// Check if NLTK Punkt is available
pub fn is_nltk_available() -> bool {
    // Try to import NLTK
    let check_script = r#"
try:
    import nltk
    nltk.data.find('tokenizers/punkt')
    print("available")
except:
    print("not available")
"#;

    // Use the environment detection module
    if let Ok(mut cmd) = python_env::build_python_command() {
        if let Ok(output) = cmd.arg("-c").arg(check_script).output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.trim() == "available";
        }
    }

    false
}
