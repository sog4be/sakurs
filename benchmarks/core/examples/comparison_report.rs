//! Generate a comparison report between sakurs and baseline tools
//!
//! This example runs benchmarks for both sakurs and NLTK Punkt,
//! then generates a comprehensive comparison report.

use sakurs_benchmarks::baselines::{is_nltk_available, run_nltk_punkt_benchmark};
use sakurs_benchmarks::config;
use sakurs_benchmarks::data::brown_corpus;
use sakurs_benchmarks::{
    calculate_complete_metrics, create_default_processor, extract_boundaries, TestData,
};
use std::time::Instant;

#[derive(Debug, Clone)]
struct BenchmarkResult {
    tool: String,
    subset_size: usize,
    precision: f64,
    recall: f64,
    f1_score: f64,
    processing_time_ms: f64,
    sentences_per_second: f64,
    chars_per_second: f64,
}

fn benchmark_sakurs(test_data: &TestData) -> BenchmarkResult {
    let processor = create_default_processor();

    // Warmup runs
    for _ in 0..config::get_warmup_runs() {
        let _ = processor.process_text(&test_data.text);
    }

    // Timed run
    let start = Instant::now();
    let output = processor
        .process_text(&test_data.text)
        .expect("Processing should not fail");
    let elapsed = start.elapsed();

    let predicted = extract_boundaries(&output);
    let metrics =
        calculate_complete_metrics(&predicted, &test_data.boundaries, test_data.text.len());

    let processing_time_ms = elapsed.as_millis() as f64;
    let processing_time_secs = elapsed.as_secs_f64();

    BenchmarkResult {
        tool: "sakurs".to_string(),
        subset_size: test_data.boundaries.len(),
        precision: metrics.precision,
        recall: metrics.recall,
        f1_score: metrics.f1_score,
        processing_time_ms,
        sentences_per_second: test_data.boundaries.len() as f64 / processing_time_secs,
        chars_per_second: test_data.text.len() as f64 / processing_time_secs,
    }
}

fn format_percentage(value: f64) -> String {
    format!("{:6.2}%", value * 100.0)
}

fn format_speed(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:7.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:7.2}K", value / 1_000.0)
    } else {
        format!("{:7.2}", value)
    }
}

fn print_comparison_table(results: &[BenchmarkResult]) {
    println!("\n╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                        Sentence Segmentation Benchmark Report                  ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Tool       │ Dataset │ Precision │  Recall   │ F1 Score  │ Time(ms) │ Sent/s ║");
    println!("╟────────────┼─────────┼───────────┼───────────┼───────────┼──────────┼────────╢");

    for result in results {
        println!(
            "║ {:10} │ {:7} │ {} │ {} │ {} │ {:8.1} │ {:6.0} ║",
            result.tool,
            format!("{}s", result.subset_size),
            format_percentage(result.precision),
            format_percentage(result.recall),
            format_percentage(result.f1_score),
            result.processing_time_ms,
            result.sentences_per_second,
        );
    }

    println!("╚════════════╧═════════╧═══════════╧═══════════╧═══════════╧══════════╧════════╝");
}

fn print_detailed_comparison(sakurs_results: &[BenchmarkResult], nltk_results: &[BenchmarkResult]) {
    println!("\n## Detailed Comparison");

    for (sakurs, nltk) in sakurs_results.iter().zip(nltk_results.iter()) {
        println!("\n### {} Sentences Dataset", sakurs.subset_size);
        println!("```");
        println!("Accuracy Metrics:");
        println!(
            "  sakurs    - Precision: {}, Recall: {}, F1: {}",
            format_percentage(sakurs.precision),
            format_percentage(sakurs.recall),
            format_percentage(sakurs.f1_score)
        );
        println!(
            "  NLTK Punkt - Precision: {}, Recall: {}, F1: {}",
            format_percentage(nltk.precision),
            format_percentage(nltk.recall),
            format_percentage(nltk.f1_score)
        );

        println!("\nPerformance Metrics:");
        println!(
            "  sakurs    - {:.1}ms ({:.0} sentences/sec, {} chars/sec)",
            sakurs.processing_time_ms,
            sakurs.sentences_per_second,
            format_speed(sakurs.chars_per_second)
        );
        println!(
            "  NLTK Punkt - {:.1}ms ({:.0} sentences/sec, {} chars/sec)",
            nltk.processing_time_ms,
            nltk.sentences_per_second,
            format_speed(nltk.chars_per_second)
        );

        // Calculate relative performance
        let speed_ratio = sakurs.sentences_per_second / nltk.sentences_per_second;
        let f1_diff = sakurs.f1_score - nltk.f1_score;

        println!("\nRelative Performance:");
        println!(
            "  Speed: sakurs is {:.1}x {} than NLTK Punkt",
            if speed_ratio >= 1.0 {
                speed_ratio
            } else {
                1.0 / speed_ratio
            },
            if speed_ratio >= 1.0 {
                "faster"
            } else {
                "slower"
            }
        );
        println!(
            "  F1 Score: sakurs is {:.1} percentage points {} than NLTK Punkt",
            f1_diff.abs() * 100.0,
            if f1_diff >= 0.0 { "higher" } else { "lower" }
        );
        println!("```");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Sentence Segmentation Benchmark Comparison");

    // Check if Brown Corpus is available
    if !brown_corpus::is_available() {
        eprintln!("❌ Brown Corpus data not available.");
        eprintln!("   Please run: cd benchmarks/data/brown_corpus && make download");
        return Ok(());
    }

    // Check if NLTK is available
    let nltk_available = is_nltk_available();
    if !nltk_available {
        eprintln!("⚠️  NLTK Punkt not available. Install with: pip install nltk");
        eprintln!("   Then run: python -c \"import nltk; nltk.download('punkt')\"");
    }

    let subset_sizes = config::get_subset_sizes(false);
    let mut all_results = Vec::new();
    let mut sakurs_results = Vec::new();
    let mut nltk_results = Vec::new();

    for &size in &subset_sizes {
        println!("\n📊 Benchmarking with {} sentences...", size);

        // Load test data
        let test_data = brown_corpus::load_subset(size)?;
        println!("   Loaded {} characters of text", test_data.text.len());

        // Benchmark sakurs
        print!("   Running sakurs benchmark... ");
        let sakurs_result = benchmark_sakurs(&test_data);
        println!("✓ ({:.1}ms)", sakurs_result.processing_time_ms);
        sakurs_results.push(sakurs_result.clone());
        all_results.push(sakurs_result);

        // Benchmark NLTK if available
        if nltk_available {
            print!("   Running NLTK Punkt benchmark... ");
            match run_nltk_punkt_benchmark(Some(size)) {
                Ok(nltk_data) => {
                    let nltk_result = BenchmarkResult {
                        tool: "NLTK Punkt".to_string(),
                        subset_size: size,
                        precision: nltk_data.metrics.precision,
                        recall: nltk_data.metrics.recall,
                        f1_score: nltk_data.metrics.f1_score,
                        processing_time_ms: nltk_data.processing_time_seconds * 1000.0,
                        sentences_per_second: nltk_data.sentences_per_second,
                        chars_per_second: nltk_data.characters_per_second,
                    };
                    println!("✓ ({:.1}ms)", nltk_result.processing_time_ms);
                    nltk_results.push(nltk_result.clone());
                    all_results.push(nltk_result);
                }
                Err(e) => {
                    eprintln!("✗ Error: {}", e);
                }
            }
        }
    }

    // Print results
    if !all_results.is_empty() {
        print_comparison_table(&all_results);

        if nltk_available && !nltk_results.is_empty() {
            print_detailed_comparison(&sakurs_results, &nltk_results);
        }
    }

    println!("\n✅ Benchmark comparison complete!");

    Ok(())
}
