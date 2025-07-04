//! UD English EWT benchmark
//!
//! This benchmark tests sakurs performance on the Universal Dependencies English Web Treebank.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sakurs_benchmarks::data::ud_english_ewt;
use sakurs_benchmarks::harness::configure_criterion;
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};
use std::hint::black_box;

/// Benchmark accuracy on UD English EWT subsets
fn bench_ud_english_ewt_subsets(c: &mut Criterion) {
    // Skip if UD English EWT data is not available
    if !ud_english_ewt::is_available() {
        eprintln!("UD English EWT data not available. Skipping benchmarks.");
        eprintln!("Run: cd benchmarks/data/ud_english_ewt && python download.py");
        return;
    }

    let mut group = c.benchmark_group("ud_english_ewt_accuracy");
    let processor = create_default_processor();

    // Test different subset sizes
    let subset_sizes = vec![100, 500, 1000];

    for size in subset_sizes {
        let test_data = match ud_english_ewt::load_subset(size) {
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

/// Benchmark throughput on UD English EWT
fn bench_ud_english_ewt_throughput(c: &mut Criterion) {
    // Skip if UD English EWT data is not available
    if !ud_english_ewt::is_available() {
        return;
    }

    let mut group = c.benchmark_group("ud_english_ewt_throughput");

    // Use full corpus or largest subset for throughput testing
    let test_data = match ud_english_ewt::load_full_corpus() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading UD English EWT: {}", e);
            return;
        }
    };

    let processor = create_default_processor();

    group.throughput(criterion::Throughput::Bytes(test_data.text.len() as u64));
    group.bench_function("process_full_corpus", |b| {
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
fn bench_ud_english_ewt_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("ud_english_ewt_sample");

    let processor = create_default_processor();
    let test_data = ud_english_ewt::small_sample();

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

criterion_group! {
    name = ud_english_ewt_benches;
    config = configure_criterion();
    targets =
        bench_ud_english_ewt_sample,
        bench_ud_english_ewt_subsets,
        bench_ud_english_ewt_throughput
}

criterion_main!(ud_english_ewt_benches);
