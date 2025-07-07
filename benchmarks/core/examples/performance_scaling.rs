//! Performance scaling analysis for sakurs
//!
//! This script runs multiple benchmark iterations for different dataset sizes
//! and analyzes the relationship between dataset size and performance.
//! Results are saved to a timestamped file in the temp directory.

use sakurs_benchmarks::data::generators;
use sakurs_benchmarks::utils::create_default_processor;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct BenchmarkResult {
    dataset_size: usize,
    sentence_count: usize,
    iterations: Vec<Duration>,
    processing_times_ms: Vec<f64>,
    throughput_mb_per_sec: Vec<f64>,
    sentences_per_sec: Vec<f64>,
}

impl BenchmarkResult {
    fn new(dataset_size: usize, sentence_count: usize) -> Self {
        Self {
            dataset_size,
            sentence_count,
            iterations: Vec::new(),
            processing_times_ms: Vec::new(),
            throughput_mb_per_sec: Vec::new(),
            sentences_per_sec: Vec::new(),
        }
    }

    fn add_iteration(&mut self, duration: Duration) {
        self.iterations.push(duration);
        let ms = duration.as_secs_f64() * 1000.0;
        self.processing_times_ms.push(ms);

        // Calculate throughput in MB/s
        let mb = self.dataset_size as f64 / (1024.0 * 1024.0);
        let throughput = mb / duration.as_secs_f64();
        self.throughput_mb_per_sec.push(throughput);

        // Calculate sentences per second
        let sentences_per_sec = self.sentence_count as f64 / duration.as_secs_f64();
        self.sentences_per_sec.push(sentences_per_sec);
    }

    fn avg_processing_time_ms(&self) -> f64 {
        self.processing_times_ms.iter().sum::<f64>() / self.processing_times_ms.len() as f64
    }

    fn std_dev_processing_time_ms(&self) -> f64 {
        let avg = self.avg_processing_time_ms();
        let variance = self
            .processing_times_ms
            .iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f64>()
            / self.processing_times_ms.len() as f64;
        variance.sqrt()
    }

    fn avg_throughput_mb_per_sec(&self) -> f64 {
        self.throughput_mb_per_sec.iter().sum::<f64>() / self.throughput_mb_per_sec.len() as f64
    }

    fn avg_sentences_per_sec(&self) -> f64 {
        self.sentences_per_sec.iter().sum::<f64>() / self.sentences_per_sec.len() as f64
    }

    fn min_processing_time_ms(&self) -> f64 {
        self.processing_times_ms
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min)
    }

    fn max_processing_time_ms(&self) -> f64 {
        self.processing_times_ms
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
    }
}

fn run_benchmark(size: usize, iterations: usize, warmup: usize) -> BenchmarkResult {
    println!(
        "\n=== Benchmarking {} bytes ({:.2} MB) ===",
        size,
        size as f64 / (1024.0 * 1024.0)
    );

    let processor = create_default_processor();
    let test_data = generators::large_text(size);
    let sentence_count = test_data.sentence_count();

    println!(
        "Text size: {} bytes, Sentences: {}",
        test_data.text.len(),
        sentence_count
    );
    println!("Running {} warmup iterations...", warmup);

    // Warmup runs
    for _ in 0..warmup {
        let _ = processor
            .process_text(&test_data.text)
            .expect("Processing should not fail");
    }

    println!("Running {} benchmark iterations...", iterations);
    let mut result = BenchmarkResult::new(test_data.text.len(), sentence_count);

    // Actual benchmark runs
    for i in 0..iterations {
        let start = Instant::now();
        let _ = processor
            .process_text(&test_data.text)
            .expect("Processing should not fail");
        let duration = start.elapsed();

        result.add_iteration(duration);

        // Progress indicator
        if (i + 1) % 10 == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
    println!(" Done!");

    result
}

fn analyze_scaling(results: &[BenchmarkResult]) -> String {
    let mut analysis = String::new();

    analysis.push_str("## Scaling Analysis\n\n");

    // Calculate scaling factor between consecutive sizes
    if results.len() > 1 {
        analysis.push_str("### Size Scaling Factors\n\n");

        for i in 1..results.len() {
            let prev = &results[i - 1];
            let curr = &results[i];

            let size_factor = curr.dataset_size as f64 / prev.dataset_size as f64;
            let time_factor = curr.avg_processing_time_ms() / prev.avg_processing_time_ms();
            let efficiency = size_factor / time_factor;

            analysis.push_str(&format!(
                "- {}k → {}k bytes:\n",
                prev.dataset_size / 1000,
                curr.dataset_size / 1000
            ));
            analysis.push_str(&format!("  - Size increased by: {:.2}x\n", size_factor));
            analysis.push_str(&format!("  - Time increased by: {:.2}x\n", time_factor));
            analysis.push_str(&format!(
                "  - Scaling efficiency: {:.2}% (100% = perfect linear)\n",
                efficiency * 100.0
            ));

            if efficiency > 0.95 {
                analysis.push_str("  - ✅ Excellent linear scaling\n");
            } else if efficiency > 0.85 {
                analysis.push_str("  - ✓ Good scaling with minor overhead\n");
            } else if efficiency > 0.70 {
                analysis.push_str("  - ⚠️  Moderate scaling degradation\n");
            } else {
                analysis.push_str("  - ❌ Poor scaling performance\n");
            }
            analysis.push('\n');
        }
    }

    // Analyze overall trend
    analysis.push_str("### Overall Performance Characteristics\n\n");

    // Calculate correlation between size and time
    let sizes: Vec<f64> = results.iter().map(|r| r.dataset_size as f64).collect();
    let times: Vec<f64> = results.iter().map(|r| r.avg_processing_time_ms()).collect();

    let n = sizes.len() as f64;
    let sum_x: f64 = sizes.iter().sum();
    let sum_y: f64 = times.iter().sum();
    let sum_xx: f64 = sizes.iter().map(|x| x * x).sum();
    let sum_xy: f64 = sizes.iter().zip(&times).map(|(x, y)| x * y).sum();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    analysis.push_str(&format!(
        "- Linear regression: time = {:.6} × size + {:.2}\n",
        slope, intercept
    ));
    analysis.push_str(&format!("- Processing rate: {:.2} bytes/ms\n", 1.0 / slope));

    // Calculate average throughput consistency
    let throughputs: Vec<f64> = results
        .iter()
        .map(|r| r.avg_throughput_mb_per_sec())
        .collect();
    let avg_throughput = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
    let throughput_variance = throughputs
        .iter()
        .map(|&x| (x - avg_throughput).powi(2))
        .sum::<f64>()
        / throughputs.len() as f64;
    let throughput_std_dev = throughput_variance.sqrt();
    let throughput_cv = throughput_std_dev / avg_throughput * 100.0;

    analysis.push_str(&format!(
        "\n- Average throughput: {:.2} MB/s\n",
        avg_throughput
    ));
    analysis.push_str(&format!(
        "- Throughput consistency (CV): {:.1}%\n",
        throughput_cv
    ));

    if throughput_cv < 5.0 {
        analysis.push_str("- ✅ Highly consistent throughput across sizes\n");
    } else if throughput_cv < 10.0 {
        analysis.push_str("- ✓ Good throughput consistency\n");
    } else if throughput_cv < 20.0 {
        analysis.push_str("- ⚠️  Moderate throughput variation\n");
    } else {
        analysis.push_str("- ❌ High throughput variation\n");
    }

    analysis
}

fn generate_report(results: &[BenchmarkResult]) -> String {
    let mut report = String::new();

    // Header
    report.push_str("# Sakurs Performance Scaling Analysis\n\n");
    report.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // Summary table
    report.push_str("## Performance Summary\n\n");
    report.push_str("| Dataset Size | Sentences | Avg Time (ms) | Std Dev (ms) | Min (ms) | Max (ms) | Throughput (MB/s) | Sentences/sec |\n");
    report.push_str("|--------------|-----------|---------------|--------------|----------|----------|-------------------|---------------|\n");

    for result in results {
        report.push_str(&format!(
            "| {:>10}k | {:>9} | {:>13.2} | {:>12.2} | {:>8.2} | {:>8.2} | {:>17.2} | {:>13.0} |\n",
            result.dataset_size / 1000,
            result.sentence_count,
            result.avg_processing_time_ms(),
            result.std_dev_processing_time_ms(),
            result.min_processing_time_ms(),
            result.max_processing_time_ms(),
            result.avg_throughput_mb_per_sec(),
            result.avg_sentences_per_sec()
        ));
    }

    report.push('\n');

    // Detailed results for each size
    report.push_str("## Detailed Results\n\n");

    for result in results {
        report.push_str(&format!(
            "### {} bytes ({} sentences)\n\n",
            result.dataset_size, result.sentence_count
        ));
        report.push_str(&format!(
            "- Average processing time: {:.2} ms (±{:.2} ms)\n",
            result.avg_processing_time_ms(),
            result.std_dev_processing_time_ms()
        ));
        report.push_str(&format!(
            "- Processing time range: {:.2} - {:.2} ms\n",
            result.min_processing_time_ms(),
            result.max_processing_time_ms()
        ));
        report.push_str(&format!(
            "- Average throughput: {:.2} MB/s\n",
            result.avg_throughput_mb_per_sec()
        ));
        report.push_str(&format!(
            "- Average rate: {:.0} sentences/second\n",
            result.avg_sentences_per_sec()
        ));
        report.push_str(&format!(
            "- Time per sentence: {:.3} ms\n",
            result.avg_processing_time_ms() / result.sentence_count as f64
        ));
        report.push('\n');
    }

    // Add scaling analysis
    report.push_str(&analyze_scaling(results));

    // Performance expectations vs actual
    report.push_str("\n## Expected vs Actual Performance\n\n");
    report.push_str("For a linear-time algorithm, we expect:\n");
    report.push_str("- Processing time to scale linearly with input size\n");
    report.push_str("- Consistent throughput (MB/s) across all sizes\n");
    report.push_str("- Constant time per sentence regardless of total size\n\n");

    // Calculate if expectations are met
    let throughputs: Vec<f64> = results
        .iter()
        .map(|r| r.avg_throughput_mb_per_sec())
        .collect();
    let min_throughput = throughputs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_throughput = throughputs
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let throughput_variation = (max_throughput - min_throughput) / min_throughput * 100.0;

    report.push_str("Actual results:\n");
    report.push_str(&format!(
        "- Throughput variation: {:.1}% (min: {:.2} MB/s, max: {:.2} MB/s)\n",
        throughput_variation, min_throughput, max_throughput
    ));

    if throughput_variation < 10.0 {
        report.push_str("- ✅ Meets linear scaling expectations\n");
    } else if throughput_variation < 20.0 {
        report.push_str("- ✓ Mostly linear with minor deviations\n");
    } else {
        report.push_str("- ⚠️  Significant deviation from linear scaling\n");
    }

    report
}

fn main() {
    println!("=== Sakurs Performance Scaling Analysis ===\n");

    // Configuration
    let dataset_sizes = vec![100, 1000, 5000];
    let iterations_per_size = 50; // Multiple iterations for statistical significance
    let warmup_iterations = 5;

    println!("Configuration:");
    println!("- Dataset sizes: {:?} bytes", dataset_sizes);
    println!("- Iterations per size: {}", iterations_per_size);
    println!("- Warmup iterations: {}", warmup_iterations);

    // Run benchmarks
    let mut results = Vec::new();

    for &size in &dataset_sizes {
        let result = run_benchmark(size, iterations_per_size, warmup_iterations);

        // Print immediate results
        println!("\nResults for {} bytes:", size);
        println!(
            "  Average time: {:.2} ms (±{:.2} ms)",
            result.avg_processing_time_ms(),
            result.std_dev_processing_time_ms()
        );
        println!(
            "  Throughput: {:.2} MB/s",
            result.avg_throughput_mb_per_sec()
        );
        println!(
            "  Rate: {:.0} sentences/second",
            result.avg_sentences_per_sec()
        );

        results.push(result);
    }

    // Generate report
    let report = generate_report(&results);

    // Save to timestamped file
    let timestamp = chrono::Local::now().format("%Y-%m-%d-%H:%M:%S");
    let filename = format!("temp/{}_performance-scaling-analysis.md", timestamp);

    match fs::write(&filename, &report) {
        Ok(_) => println!("\n✅ Report saved to: {}", filename),
        Err(e) => eprintln!("\n❌ Failed to save report: {}", e),
    }

    // Print summary to console
    println!("\n=== Summary ===\n");
    println!("{}", analyze_scaling(&results));
}

// Add chrono dependency for timestamps
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_calculations() {
        let mut result = BenchmarkResult::new(1000, 10);

        result.add_iteration(Duration::from_millis(10));
        result.add_iteration(Duration::from_millis(12));
        result.add_iteration(Duration::from_millis(11));

        assert_eq!(result.processing_times_ms.len(), 3);
        assert!((result.avg_processing_time_ms() - 11.0).abs() < 0.1);
        assert!(result.std_dev_processing_time_ms() > 0.0);
        assert_eq!(result.min_processing_time_ms(), 10.0);
        assert_eq!(result.max_processing_time_ms(), 12.0);
    }
}
