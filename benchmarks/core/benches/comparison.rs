//! Comparison benchmarks between sakurs and baseline tools
//!
//! This benchmark compares sakurs against NLTK Punkt and other
//! sentence segmentation tools using the same test data.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sakurs_benchmarks::baselines::{is_nltk_available, run_nltk_punkt_benchmark};
use sakurs_benchmarks::data::brown_corpus;
use sakurs_benchmarks::harness::configure_criterion;
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};
use std::hint::black_box;
use std::time::Instant;

/// Run comparative accuracy evaluation
fn bench_accuracy_comparison(c: &mut Criterion) {
    // Skip if Brown Corpus data is not available
    if !brown_corpus::is_available() {
        eprintln!("Brown Corpus data not available. Skipping comparison benchmarks.");
        return;
    }

    let mut group = c.benchmark_group("accuracy_comparison");
    let subset_sizes = vec![100, 1000];

    for size in subset_sizes {
        let test_data = match brown_corpus::load_subset(size) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error loading subset of {} sentences: {}", size, e);
                continue;
            }
        };

        // Benchmark sakurs
        group.bench_with_input(
            BenchmarkId::new("sakurs", format!("{}_sentences", size)),
            &test_data,
            |b, test_data| {
                let processor = create_default_processor();
                b.iter(|| {
                    let output = processor
                        .process_text(black_box(&test_data.text))
                        .expect("Processing should not fail");

                    let predicted = extract_boundaries(&output);
                    let metrics = calculate_complete_metrics(
                        &predicted,
                        &test_data.boundaries,
                        test_data.text.len(),
                    );
                    black_box(metrics)
                });
            },
        );

        // Note: NLTK Punkt is benchmarked separately via Python script
        // due to the overhead of Python interop
    }

    group.finish();
}

/// Run comparative throughput evaluation
fn bench_throughput_comparison(c: &mut Criterion) {
    if !brown_corpus::is_available() {
        return;
    }

    let mut group = c.benchmark_group("throughput_comparison");

    // Use a medium-sized dataset for throughput testing
    let test_data = match brown_corpus::load_subset(5000) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading corpus: {}", e);
            return;
        }
    };

    group.throughput(criterion::Throughput::Bytes(test_data.text.len() as u64));

    // Benchmark sakurs throughput
    let processor = create_default_processor();
    group.bench_function("sakurs_5k_sentences", |b| {
        b.iter(|| {
            let output = processor
                .process_text(black_box(&test_data.text))
                .expect("Processing should not fail");
            black_box(output)
        });
    });

    group.finish();
}

/// Print comparison results at the end
fn print_comparison_summary() {
    println!("\n=== Comparison Summary ===");

    // Check if we can run NLTK comparison
    if is_nltk_available() {
        println!("Running NLTK Punkt comparison...");

        // Run NLTK benchmarks for different sizes
        for size in [100, 1000, 5000] {
            match run_nltk_punkt_benchmark(Some(size)) {
                Ok(result) => {
                    println!("\nNLTK Punkt - {} sentences:", size);
                    println!("  Precision: {:.1}%", result.metrics.precision * 100.0);
                    println!("  Recall: {:.1}%", result.metrics.recall * 100.0);
                    println!("  F1 Score: {:.1}%", result.metrics.f1_score * 100.0);
                    println!(
                        "  Throughput: {:.0} sentences/sec",
                        result.sentences_per_second
                    );
                }
                Err(e) => {
                    eprintln!("Error running NLTK benchmark: {}", e);
                }
            }
        }
    } else {
        println!("NLTK Punkt not available. Install with: pip install nltk");
    }

    println!("\nNote: For detailed comparison, run:");
    println!("  cargo run --example comparison_report");
}

criterion_group! {
    name = comparison_benches;
    config = configure_criterion();
    targets = bench_accuracy_comparison, bench_throughput_comparison
}

criterion_main!(comparison_benches);
