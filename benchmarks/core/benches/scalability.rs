//! Scalability benchmarks for sakurs parallel processing
//!
//! This benchmark evaluates how well sakurs scales with multiple threads
//! and different workload characteristics.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_benchmarks::data::generators;
use sakurs_core::application::ProcessorConfig;
use sakurs_core::application::TextProcessor;
use sakurs_core::domain::language::EnglishLanguageRules;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

/// Benchmark parallel scaling with different thread counts
#[cfg(feature = "parallel")]
fn bench_thread_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_scaling");

    let rules = Arc::new(EnglishLanguageRules::new());

    // Test with a large text that benefits from parallelization
    let test_data = generators::large_text(1_000_000);

    group.throughput(Throughput::Bytes(test_data.text.len() as u64));

    for thread_count in [1, 2, 4, 8] {
        // Skip if we don't have enough CPU cores
        if thread_count > num_cpus::get() {
            continue;
        }

        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            &test_data.text,
            |b, text| {
                let mut config = ProcessorConfig::default();
                config.max_threads = Some(thread_count);
                config.parallel_threshold = 10_000; // Low threshold to ensure parallel execution

                let processor = TextProcessor::with_config(config, rules.clone());

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

/// Benchmark the effect of chunk size on parallel performance
#[cfg(feature = "parallel")]
fn bench_chunk_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_size_impact");

    let rules = Arc::new(EnglishLanguageRules::new());
    let test_data = generators::large_text(500_000);

    group.throughput(Throughput::Bytes(test_data.text.len() as u64));

    for chunk_size in [4_096, 16_384, 65_536, 262_144] {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            &test_data.text,
            |b, text| {
                let mut config = ProcessorConfig::default();
                config.chunk_size = chunk_size;
                config.parallel_threshold = 10_000;
                config.max_threads = Some(4); // Fixed thread count

                let processor = TextProcessor::with_config(config, rules.clone());

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

/// Benchmark parallel efficiency with different text sizes
#[cfg(feature = "parallel")]
fn bench_parallel_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_efficiency");

    let rules = Arc::new(EnglishLanguageRules::new());

    for size in [100_000, 500_000, 1_000_000, 5_000_000] {
        let test_data = generators::large_text(size);

        // Measure sequential performance
        let mut seq_config = ProcessorConfig::default();
        seq_config.parallel_threshold = usize::MAX; // Force sequential
        let seq_processor = TextProcessor::with_config(seq_config, rules.clone());

        // Measure parallel performance with optimal threads
        let mut par_config = ProcessorConfig::default();
        par_config.parallel_threshold = 10_000;
        par_config.max_threads = Some(num_cpus::get());
        let par_processor = TextProcessor::with_config(par_config, rules.clone());

        group.throughput(Throughput::Bytes(size as u64));

        // Sequential baseline
        group.bench_with_input(
            BenchmarkId::new(format!("{}k_sequential", size / 1000), "seq"),
            &test_data.text,
            |b, text| {
                b.iter(|| {
                    let result = seq_processor
                        .process_text(black_box(text))
                        .expect("Processing should not fail");
                    black_box(result)
                });
            },
        );

        // Parallel version
        group.bench_with_input(
            BenchmarkId::new(format!("{}k_parallel", size / 1000), "par"),
            &test_data.text,
            |b, text| {
                b.iter(|| {
                    let result = par_processor
                        .process_text(black_box(text))
                        .expect("Processing should not fail");
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Measure and report scalability metrics
#[cfg(feature = "parallel")]
fn measure_scalability_metrics() {
    println!("\n=== Sakurs Scalability Report ===\n");

    let rules = Arc::new(EnglishLanguageRules::new());
    let test_data = generators::large_text(1_000_000);
    let cpu_count = num_cpus::get();

    println!("System: {} CPU cores available", cpu_count);
    println!(
        "Test data: 1M characters, {} sentences\n",
        test_data.sentence_count()
    );

    println!(
        "{:<10} {:>15} {:>15} {:>15} {:>15}",
        "Threads", "Time (ms)", "Throughput", "Speedup", "Efficiency"
    );
    println!("{:-<70}", "");

    let mut baseline_time = 0.0;

    for thread_count in [1, 2, 4, 8] {
        if thread_count > cpu_count {
            continue;
        }

        let mut config = ProcessorConfig::default();
        config.max_threads = Some(thread_count);
        config.parallel_threshold = if thread_count == 1 {
            usize::MAX
        } else {
            10_000
        };

        let processor = TextProcessor::with_config(config, rules.clone());

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

        if thread_count == 1 {
            baseline_time = avg_time_ms;
        }

        let speedup = baseline_time / avg_time_ms;
        let efficiency = speedup / thread_count as f64 * 100.0;
        let throughput_mb_s = (test_data.text.len() as f64 / 1_000_000.0) / (avg_time_ms / 1000.0);

        println!(
            "{:<10} {:>15.2} {:>12.2} MB/s {:>15.2}x {:>14.1}%",
            thread_count, avg_time_ms, throughput_mb_s, speedup, efficiency,
        );
    }

    println!("\n");
}

// Fallback for when parallel feature is not enabled
#[cfg(not(feature = "parallel"))]
fn bench_no_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("no_parallel");

    let rules = Arc::new(EnglishLanguageRules::new());
    let processor = TextProcessor::new(rules);

    let test_data = generators::large_text(100_000);

    group.bench_function("sequential_only", |b| {
        b.iter(|| {
            let result = processor
                .process_text(black_box(&test_data.text))
                .expect("Processing should not fail");
            black_box(result)
        });
    });

    group.finish();

    println!("\nNote: Parallel feature is not enabled. ");
    println!("Run with --features parallel to test scalability.\n");
}

// Configure criterion for scalability benchmarks
fn get_criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(15))
        .warm_up_time(std::time::Duration::from_secs(5))
}

#[cfg(feature = "parallel")]
criterion_group! {
    name = scalability_benches;
    config = get_criterion_config();
    targets =
        bench_thread_scaling,
        bench_chunk_size_impact,
        bench_parallel_efficiency
}

#[cfg(not(feature = "parallel"))]
criterion_group! {
    name = scalability_benches;
    config = get_criterion_config();
    targets = bench_no_parallel
}

criterion_main!(scalability_benches);

// Optional: Run this with `cargo test --benches` to see scalability metrics
#[test]
#[cfg(feature = "parallel")]
fn test_scalability_report() {
    measure_scalability_metrics();
}

#[test]
#[cfg(not(feature = "parallel"))]
fn test_no_parallel_notice() {
    println!("\nParallel feature is not enabled for scalability testing.");
    println!("Run with: cargo test --features parallel\n");
}
