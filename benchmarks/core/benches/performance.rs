//! Performance benchmarks for sakurs sentence boundary detection
//!
//! This benchmark measures throughput, latency, and memory characteristics
//! of the sakurs text processor.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_benchmarks::constants::{sentence_lengths, text_sizes};
use sakurs_benchmarks::create_default_processor;
use sakurs_benchmarks::data::generators;
use std::hint::black_box;
use std::time::Instant;

/// Benchmark throughput for different text sizes
fn bench_throughput_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let processor = create_default_processor();

    for &size in text_sizes::THROUGHPUT_SIZES {
        let test_data = generators::large_text(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}k", size / 1000)),
            &test_data.text,
            |b, text| {
                b.iter(|| {
                    let result = processor
                        .process_text(black_box(text))
                        .expect("Processing should not fail");
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark throughput for different text complexities
fn bench_throughput_by_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_complexity");

    let processor = create_default_processor();

    // Create texts of same size but different complexity
    let size = 50_000;

    // Simple: just periods
    let simple_text = "This is a simple sentence. ".repeat(size / sentence_lengths::SIMPLE);

    // Medium: some abbreviations
    let medium_text =
        "Dr. Smith said this. The U.S. is great. ".repeat(size / sentence_lengths::MEDIUM);

    // Complex: quotes, abbreviations, numbers
    let complex_text = r#"Dr. Smith said, "The U.S. GDP grew 3.5% in Q1." "#
        .repeat(size / sentence_lengths::COMPLEX);

    let test_cases = vec![
        ("simple", simple_text),
        ("medium", medium_text),
        ("complex", complex_text),
    ];

    for (name, text) in test_cases {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::new("complexity", name), &text, |b, text| {
            b.iter(|| {
                let result = processor
                    .process_text(black_box(text))
                    .expect("Processing should not fail");
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark latency for small inputs
fn bench_latency_small_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_small");

    let processor = create_default_processor();

    let test_cases = vec![
        ("single_sentence", "This is a single sentence."),
        ("two_sentences", "First sentence. Second sentence."),
        (
            "paragraph",
            "This is the first sentence. Here is the second one. And a third. Finally, the fourth.",
        ),
    ];

    for (name, text) in test_cases {
        group.bench_with_input(BenchmarkId::new("latency", name), text, |b, text| {
            b.iter(|| {
                let result = processor
                    .process_text(black_box(text))
                    .expect("Processing should not fail");
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    for size in [10_000, 100_000, 1_000_000] {
        let test_data = generators::large_text(size);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}k", size / 1000)),
            &test_data.text,
            |b, text| {
                b.iter(|| {
                    // Create new processor each time to measure full allocation
                    let processor = create_default_processor();
                    let result = processor
                        .process_text(black_box(text))
                        .expect("Processing should not fail");
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Measure sentences per second for reporting
fn measure_sentences_per_second() {
    println!("\n=== Sakurs Performance Metrics ===\n");

    let processor = create_default_processor();

    let test_sizes = vec![
        ("Small (1K)", 1_000),
        ("Medium (100K)", 100_000),
        ("Large (1M)", 1_000_000),
    ];

    println!(
        "{:<15} {:>15} {:>15} {:>20}",
        "Text Size", "Sentences", "Time (ms)", "Sentences/sec"
    );
    println!("{:-<65}", "");

    for (name, size) in test_sizes {
        let test_data = generators::large_text(size);
        let sentence_count = test_data.sentence_count();

        // Warm up
        for _ in 0..3 {
            let _ = processor.process_text(&test_data.text);
        }

        // Measure
        let start = Instant::now();
        let iterations = 10;

        for _ in 0..iterations {
            let _ = processor
                .process_text(&test_data.text)
                .expect("Processing should not fail");
        }

        let elapsed = start.elapsed();
        let avg_time_ms = elapsed.as_millis() as f64 / iterations as f64;
        let sentences_per_sec = (sentence_count as f64 * 1000.0) / avg_time_ms;

        println!(
            "{:<15} {:>15} {:>15.2} {:>20.0}",
            name, sentence_count, avg_time_ms, sentences_per_sec,
        );
    }

    println!("\n");
}

// Configure criterion for performance benchmarks
fn get_criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3))
}

criterion_group! {
    name = performance_benches;
    config = get_criterion_config();
    targets =
        bench_throughput_by_size,
        bench_throughput_by_complexity,
        bench_latency_small_inputs,
        bench_memory_patterns
}

criterion_main!(performance_benches);

// Optional: Run this with `cargo test --benches` to see performance metrics
#[test]
fn test_performance_metrics() {
    measure_sentences_per_second();
}
