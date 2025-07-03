//! Brown Corpus benchmarks for sakurs
//!
//! This benchmark evaluates sakurs against the Brown Corpus dataset,
//! a standard corpus for NLP evaluation.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sakurs_benchmarks::data::brown_corpus;
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};
use std::hint::black_box;

/// Benchmark accuracy on Brown Corpus subsets
fn bench_brown_corpus_subsets(c: &mut Criterion) {
    // Skip if Brown Corpus data is not available
    if !brown_corpus::is_available() {
        eprintln!("Brown Corpus data not available. Skipping benchmarks.");
        eprintln!("Run: cd benchmarks/data/brown_corpus && make download");
        return;
    }

    let mut group = c.benchmark_group("brown_corpus_accuracy");
    let processor = create_default_processor();

    // Test different subset sizes
    let subset_sizes = vec![100, 500, 1000, 5000];

    for size in subset_sizes {
        let test_data = match brown_corpus::load_subset(size) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error loading subset of {} sentences: {}", size, e);
                continue;
            }
        };

        group.bench_with_input(
            BenchmarkId::new("subset", format!("{}_sentences", size)),
            &test_data,
            |b, test_data| {
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
    }

    group.finish();
}

/// Benchmark throughput on full Brown Corpus
fn bench_brown_corpus_throughput(c: &mut Criterion) {
    // Skip if Brown Corpus data is not available
    if !brown_corpus::is_available() {
        return;
    }

    let mut group = c.benchmark_group("brown_corpus_throughput");

    // Use a medium subset for throughput testing (full corpus might be too slow)
    let test_data = match brown_corpus::load_subset(10000) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading Brown Corpus: {}", e);
            return;
        }
    };

    let processor = create_default_processor();

    group.throughput(criterion::Throughput::Bytes(test_data.text.len() as u64));
    group.bench_function("process_10k_sentences", |b| {
        b.iter(|| {
            let output = processor
                .process_text(black_box(&test_data.text))
                .expect("Processing should not fail");
            black_box(output)
        });
    });

    group.finish();
}

/// Benchmark to compare with the hardcoded sample
fn bench_brown_corpus_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("brown_corpus_sample");

    let processor = create_default_processor();
    let test_data = brown_corpus::small_sample();

    group.bench_function("hardcoded_sample", |b| {
        b.iter(|| {
            let output = processor
                .process_text(black_box(&test_data.text))
                .expect("Processing should not fail");

            let predicted = extract_boundaries(&output);
            let metrics =
                calculate_complete_metrics(&predicted, &test_data.boundaries, test_data.text.len());
            black_box(metrics)
        });
    });

    group.finish();
}

// Configure criterion for Brown Corpus benchmarks
fn get_criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .measurement_time(std::time::Duration::from_secs(10))
}

criterion_group! {
    name = brown_corpus_benches;
    config = get_criterion_config();
    targets =
        bench_brown_corpus_sample,
        bench_brown_corpus_subsets,
        bench_brown_corpus_throughput
}

criterion_main!(brown_corpus_benches);
