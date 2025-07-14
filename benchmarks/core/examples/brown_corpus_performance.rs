//! Analyze performance scaling with Brown Corpus datasets
//!
//! This tool measures how sakurs performance scales with increasing
//! dataset sizes using actual Brown Corpus data.

use chrono::Local;
use sakurs_benchmarks::data::brown_corpus;
use sakurs_benchmarks::{create_default_processor, extract_boundaries};
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Brown Corpus Performance Scaling Analysis ===\n");

    // Check if Brown Corpus is available
    if !brown_corpus::is_available() {
        eprintln!("❌ Brown Corpus data not available.");
        eprintln!("   Please run: cd benchmarks/data/brown_corpus && make download");
        return Ok(());
    }

    // Configuration
    let dataset_sizes = vec![100, 1000, 5000];
    let iterations = 10; // Multiple runs for accuracy
    let warmup = 3;

    println!("Configuration:");
    println!("- Dataset sizes: {:?} sentences", dataset_sizes);
    println!("- Iterations per size: {}", iterations);
    println!("- Warmup iterations: {}\n", warmup);

    let processor = create_default_processor();
    let mut results = Vec::new();

    for &size in &dataset_sizes {
        println!("=== Benchmarking {} sentences ===", size);

        // Load dataset
        let test_data = brown_corpus::load_subset(size)?;
        println!(
            "Loaded {} bytes ({:.2} MB), {} sentences",
            test_data.text.len(),
            test_data.text.len() as f64 / 1_000_000.0,
            test_data.boundaries.len()
        );

        // Warmup
        print!("Running {} warmup iterations...", warmup);
        for _ in 0..warmup {
            let _ = processor.process(sakurs_core::Input::from_text(test_data.text));
        }
        println!(" Done!");

        // Benchmark
        print!("Running {} benchmark iterations...", iterations);
        let mut times = Vec::new();
        let mut boundary_counts = Vec::new();

        for _ in 0..iterations {
            let start = Instant::now();
            let output = processor.process(sakurs_core::Input::from_text(test_data.text))?;
            let elapsed = start.elapsed();

            let boundaries = extract_boundaries(&output);
            boundary_counts.push(boundaries.len());
            times.push(elapsed.as_secs_f64() * 1000.0); // Convert to milliseconds
            print!(".");
        }
        println!(" Done!");

        // Calculate statistics
        let avg_time = times.iter().sum::<f64>() / times.len() as f64;
        let min_time = times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_time = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Standard deviation
        let variance =
            times.iter().map(|&t| (t - avg_time).powi(2)).sum::<f64>() / times.len() as f64;
        let std_dev = variance.sqrt();

        // Throughput calculations
        let bytes_per_ms = test_data.text.len() as f64 / avg_time;
        let mb_per_s = bytes_per_ms / 1000.0; // Convert to MB/s
        let sentences_per_s = test_data.boundaries.len() as f64 / (avg_time / 1000.0);
        let avg_boundaries =
            boundary_counts.iter().sum::<usize>() as f64 / boundary_counts.len() as f64;

        println!("\nResults for {} sentences:", size);
        println!("  Average time: {:.2} ms (±{:.2} ms)", avg_time, std_dev);
        println!("  Min/Max time: {:.2} / {:.2} ms", min_time, max_time);
        println!("  Throughput: {:.2} MB/s", mb_per_s);
        println!("  Rate: {:.0} sentences/second", sentences_per_s);
        println!(
            "  Detected boundaries: {:.0} (actual: {})",
            avg_boundaries,
            test_data.boundaries.len()
        );
        println!();

        results.push((
            size,
            test_data.text.len(),
            test_data.boundaries.len(),
            avg_time,
            std_dev,
            min_time,
            max_time,
            mb_per_s,
            sentences_per_s,
        ));
    }

    // Analyze scaling
    println!("=== Scaling Analysis ===\n");

    let mut report = String::new();
    report.push_str("# Brown Corpus Performance Scaling Analysis\n\n");
    report.push_str(&format!(
        "Generated: {}\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    report.push_str("## Performance Summary\n\n");
    report.push_str("| Sentences | Text Size | Avg Time (ms) | Std Dev | Throughput (MB/s) | Sentences/sec | Time/Sentence (μs) |\n");
    report.push_str("|-----------|-----------|---------------|---------|-------------------|---------------|--------------------|\n");

    for &(sentences, text_size, _, avg_time, std_dev, _, _, throughput, sent_per_sec) in &results {
        let time_per_sentence = (avg_time * 1000.0) / sentences as f64; // microseconds
        report.push_str(&format!(
            "| {:>9} | {:>8.1}k | {:>13.2} | {:>7.2} | {:>17.2} | {:>13.0} | {:>18.1} |\n",
            sentences,
            text_size as f64 / 1000.0,
            avg_time,
            std_dev,
            throughput,
            sent_per_sec,
            time_per_sentence
        ));
    }

    report.push_str("\n## Scaling Factors\n\n");

    for i in 1..results.len() {
        let (prev_sentences, prev_text_size, _, prev_time, _, _, _, _, _) = results[i - 1];
        let (sentences, text_size, _, time, _, _, _, _, _) = results[i];

        let size_factor = text_size as f64 / prev_text_size as f64;
        let time_factor = time / prev_time;
        let efficiency = (size_factor / time_factor) * 100.0;

        report.push_str(&format!(
            "### {} → {} sentences\n",
            prev_sentences, sentences
        ));
        report.push_str(&format!("- Text size increased: {:.1}x\n", size_factor));
        report.push_str(&format!(
            "- Processing time increased: {:.1}x\n",
            time_factor
        ));
        report.push_str(&format!("- Scaling efficiency: {:.1}%\n", efficiency));

        if efficiency > 90.0 {
            report.push_str("- ✅ Excellent scaling\n");
        } else if efficiency > 70.0 {
            report.push_str("- ✓ Good scaling\n");
        } else if efficiency > 50.0 {
            report.push_str("- ⚠️ Suboptimal scaling\n");
        } else {
            report.push_str("- ❌ Poor scaling\n");
        }
        report.push('\n');
    }

    // Calculate overall throughput consistency
    let throughputs: Vec<f64> = results.iter().map(|r| r.7).collect();
    let avg_throughput = throughputs.iter().sum::<f64>() / throughputs.len() as f64;
    let throughput_variance = throughputs
        .iter()
        .map(|&t| (t - avg_throughput).powi(2))
        .sum::<f64>()
        / throughputs.len() as f64;
    let throughput_cv = (throughput_variance.sqrt() / avg_throughput) * 100.0;

    report.push_str("## Performance Characteristics\n\n");
    report.push_str(&format!(
        "- Average throughput: {:.2} MB/s\n",
        avg_throughput
    ));
    report.push_str(&format!(
        "- Throughput coefficient of variation: {:.1}%\n",
        throughput_cv
    ));

    if throughput_cv < 10.0 {
        report.push_str("- ✅ Highly consistent performance across dataset sizes\n");
    } else if throughput_cv < 25.0 {
        report.push_str("- ✓ Reasonably consistent performance\n");
    } else {
        report.push_str("- ⚠️ Performance varies significantly with dataset size\n");
    }

    // Save report
    let timestamp = Local::now().format("%Y-%m-%d-%H:%M:%S");
    let filename = format!("temp/{}_brown-corpus-performance.md", timestamp);
    fs::create_dir_all("temp")?;
    fs::write(&filename, report)?;

    println!("✅ Report saved to: {}", filename);

    Ok(())
}
