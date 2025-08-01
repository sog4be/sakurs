//! Profiling benchmark to determine optimal thresholds for adaptive processing

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_core::{Config, Input, SentenceProcessor};
use std::hint::black_box;

/// Generate test text of specified size
fn generate_text(size_kb: usize) -> String {
    let base_text = "This is a test sentence. It has multiple words! Does it work? ";
    let base_len = base_text.len();
    let target_size = size_kb * 1024;
    let repeat_count = target_size / base_len + 1;
    base_text.repeat(repeat_count)[..target_size].to_string()
}

/// Profile different file sizes with sequential processing
fn bench_sequential_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_by_size");

    let sizes_kb = vec![1, 10, 100, 1000, 10000]; // 1KB to 10MB

    for size_kb in sizes_kb {
        let text = generate_text(size_kb);
        group.throughput(Throughput::Bytes(text.len() as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let config = Config::builder().threads(Some(1)).build().unwrap();
                let processor = SentenceProcessor::with_config(config).unwrap();
                b.iter(|| processor.process(Input::from_text(black_box(text))));
            },
        );
    }

    group.finish();
}

/// Profile different file sizes with parallel processing
fn bench_parallel_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_by_size");

    let sizes_kb = vec![100, 1000, 10000, 100000]; // 100KB to 100MB
    let thread_counts = vec![2, 4, 8];

    for size_kb in sizes_kb {
        let text = generate_text(size_kb);

        for threads in &thread_counts {
            group.throughput(Throughput::Bytes(text.len() as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(format!("{}KB_{}threads", size_kb, threads)),
                &text,
                |b, text| {
                    let config = Config::builder().threads(Some(*threads)).build().unwrap();
                    let processor = SentenceProcessor::with_config(config).unwrap();
                    b.iter(|| processor.process(Input::from_text(black_box(text))));
                },
            );
        }
    }

    group.finish();
}

/// Profile chunk size impact
fn bench_chunk_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_size_impact");

    let text = generate_text(10000); // 10MB text
    let chunk_sizes_kb = vec![16, 32, 64, 128, 256, 512, 1024];

    for chunk_kb in chunk_sizes_kb {
        group.throughput(Throughput::Bytes(text.len() as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB_chunks", chunk_kb)),
            &text,
            |b, text| {
                let config = Config::builder()
                    .chunk_size(chunk_kb * 1024)
                    .threads(Some(4))
                    .build()
                    .unwrap();
                let processor = SentenceProcessor::with_config(config).unwrap();
                b.iter(|| processor.process(Input::from_text(black_box(text))));
            },
        );
    }

    group.finish();
}

/// Profile crossover points between sequential and parallel
fn bench_crossover_points(c: &mut Criterion) {
    let mut group = c.benchmark_group("crossover_points");

    // Test sizes around expected crossover points
    let sizes_kb = vec![50, 75, 100, 125, 150, 200, 250, 300];

    for size_kb in sizes_kb {
        let text = generate_text(size_kb);
        group.throughput(Throughput::Bytes(text.len() as u64));

        // Sequential
        group.bench_with_input(
            BenchmarkId::new("sequential", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let config = Config::builder().threads(Some(1)).build().unwrap();
                let processor = SentenceProcessor::with_config(config).unwrap();
                b.iter(|| processor.process(Input::from_text(black_box(text))));
            },
        );

        // Parallel with 2 threads
        group.bench_with_input(
            BenchmarkId::new("parallel_2", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let config = Config::builder().threads(Some(2)).build().unwrap();
                let processor = SentenceProcessor::with_config(config).unwrap();
                b.iter(|| processor.process(Input::from_text(black_box(text))));
            },
        );

        // Parallel with 4 threads
        group.bench_with_input(
            BenchmarkId::new("parallel_4", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let config = Config::builder().threads(Some(4)).build().unwrap();
                let processor = SentenceProcessor::with_config(config).unwrap();
                b.iter(|| processor.process(Input::from_text(black_box(text))));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_by_size,
    bench_parallel_by_size,
    bench_chunk_size_impact,
    bench_crossover_points
);
criterion_main!(benches);
