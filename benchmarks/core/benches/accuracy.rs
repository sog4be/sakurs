//! Accuracy benchmarks for sakurs sentence boundary detection
//!
//! This benchmark evaluates the accuracy of sakurs against various test cases
//! with known ground truth boundaries.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sakurs_benchmarks::constants::text_sizes;
use sakurs_benchmarks::data::{brown_corpus, generators};
use sakurs_benchmarks::{
    calculate_complete_metrics, create_default_processor, extract_boundaries, AccuracyMetrics,
    TestData,
};
use std::hint::black_box;

/// Run accuracy evaluation on a single test case
fn evaluate_accuracy(
    processor: &sakurs_core::application::TextProcessor,
    test_data: &TestData,
) -> AccuracyMetrics {
    let output = processor
        .process_text(&test_data.text)
        .expect("Processing should not fail");

    // Extract boundaries and calculate all metrics
    let predicted = extract_boundaries(&output);
    calculate_complete_metrics(&predicted, &test_data.boundaries, test_data.text.len())
}

/// Benchmark accuracy on different text types
fn bench_accuracy_by_text_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("accuracy_by_type");

    let processor = create_default_processor();

    let test_cases = vec![
        generators::simple_sentences(10),
        generators::with_abbreviations(),
        generators::with_quotations(),
        generators::with_numbers(),
        generators::complex_mixed(),
    ];

    for test_data in &test_cases {
        // Validate test data
        test_data.validate().expect("Test data should be valid");

        group.bench_with_input(
            BenchmarkId::new("accuracy", &test_data.name),
            test_data,
            |b, test_data| {
                b.iter(|| {
                    let metrics = evaluate_accuracy(&processor, black_box(test_data));
                    black_box(metrics)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark accuracy vs text size
fn bench_accuracy_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("accuracy_by_size");

    let processor = create_default_processor();

    for &size in text_sizes::ACCURACY_SIZES {
        let test_data = generators::large_text(size);
        test_data.validate().expect("Test data should be valid");

        group.bench_with_input(
            BenchmarkId::new("accuracy", format!("{}k", size / 1000)),
            &test_data,
            |b, test_data| {
                b.iter(|| {
                    let metrics = evaluate_accuracy(&processor, black_box(test_data));
                    black_box(metrics)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark Brown Corpus accuracy
fn bench_brown_corpus_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("brown_corpus_accuracy");

    let processor = create_default_processor();

    let test_data = brown_corpus::small_sample();
    test_data.validate().expect("Test data should be valid");

    group.bench_function("brown_sample", |b| {
        b.iter(|| {
            let metrics = evaluate_accuracy(&processor, black_box(&test_data));

            // Print metrics for visibility (only in non-benchmark runs)
            #[cfg(debug_assertions)]
            {
                println!("Brown Corpus Sample Results:");
                println!("  Precision: {:.2}%", metrics.precision * 100.0);
                println!("  Recall: {:.2}%", metrics.recall * 100.0);
                println!("  F1 Score: {:.2}%", metrics.f1_score * 100.0);
                if let Some(pk) = metrics.pk_score {
                    println!("  Pk Score: {:.4}", pk);
                }
                if let Some(wd) = metrics.window_diff {
                    println!("  WindowDiff: {:.4}", wd);
                }
            }

            black_box(metrics)
        });
    });

    group.finish();
}

/// Generate a summary report of accuracy metrics
fn generate_accuracy_report() {
    println!("\n=== Sakurs Accuracy Report ===\n");

    let processor = create_default_processor();

    let test_cases = vec![
        ("Simple Sentences", generators::simple_sentences(10)),
        ("With Abbreviations", generators::with_abbreviations()),
        ("With Quotations", generators::with_quotations()),
        ("With Numbers", generators::with_numbers()),
        ("Complex Mixed", generators::complex_mixed()),
        ("Brown Corpus Sample", brown_corpus::small_sample()),
    ];

    println!(
        "{:<20} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Test Case", "Precision", "Recall", "F1", "Pk", "WinDiff"
    );
    println!("{:-<80}", "");

    for (name, test_data) in test_cases {
        test_data.validate().expect("Test data should be valid");
        let metrics = evaluate_accuracy(&processor, &test_data);

        println!(
            "{:<20} {:>9.1}% {:>9.1}% {:>9.1}% {:>10.4} {:>10.4}",
            name,
            metrics.precision * 100.0,
            metrics.recall * 100.0,
            metrics.f1_score * 100.0,
            metrics.pk_score.unwrap_or(0.0),
            metrics.window_diff.unwrap_or(0.0),
        );
    }

    println!("\n");
}

// Custom configuration for accuracy benchmarks
fn get_criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(50)
        .measurement_time(std::time::Duration::from_secs(5))
}

criterion_group! {
    name = accuracy_benches;
    config = get_criterion_config();
    targets =
        bench_accuracy_by_text_type,
        bench_accuracy_by_size,
        bench_brown_corpus_accuracy
}

criterion_main!(accuracy_benches);

// Optional: Run this with `cargo test --benches` to see the accuracy report
#[test]
fn test_accuracy_report() {
    generate_accuracy_report();
}
