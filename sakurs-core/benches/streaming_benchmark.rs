//! Benchmarks for streaming processing performance

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use sakurs_core::{
    domain::language::EnglishLanguageRules,
    processing::{ProcessingConfig, ProcessingStrategy, StreamingStrategy},
};
use std::hint::black_box;
use std::sync::Arc;

/// Generate test text of specified size
fn generate_test_text(size_mb: usize) -> String {
    let base_text = "This is a test sentence. It contains multiple words and ends with a period. ";
    let base_len = base_text.len();
    let target_size = size_mb * 1024 * 1024;
    let repetitions = target_size / base_len;

    base_text.repeat(repetitions)
}

fn benchmark_streaming_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_small");

    let text_1mb = generate_test_text(1);
    let text_10mb = generate_test_text(10);

    let rules = Arc::new(EnglishLanguageRules::new());
    let strategy = StreamingStrategy::new(rules);

    group.throughput(Throughput::Bytes(text_1mb.len() as u64));
    group.bench_function("1MB", |b| {
        b.iter(|| {
            let config = ProcessingConfig::default();
            let _boundaries = strategy.process(black_box(&text_1mb), &config).unwrap();
        });
    });

    group.throughput(Throughput::Bytes(text_10mb.len() as u64));
    group.bench_function("10MB", |b| {
        b.iter(|| {
            let config = ProcessingConfig::default();
            let _boundaries = strategy.process(black_box(&text_10mb), &config).unwrap();
        });
    });

    group.finish();
}

fn benchmark_streaming_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_medium");
    group.sample_size(10); // Reduce sample size for larger texts

    let text_50mb = generate_test_text(50);
    let text_100mb = generate_test_text(100);

    let rules = Arc::new(EnglishLanguageRules::new());
    let strategy = StreamingStrategy::new(rules);

    group.throughput(Throughput::Bytes(text_50mb.len() as u64));
    group.bench_function("50MB", |b| {
        b.iter(|| {
            let config = ProcessingConfig {
                buffer_size: 8 * 1024 * 1024, // 8MB buffer
                ..Default::default()
            };
            let _boundaries = strategy.process(black_box(&text_50mb), &config).unwrap();
        });
    });

    group.throughput(Throughput::Bytes(text_100mb.len() as u64));
    group.bench_function("100MB", |b| {
        b.iter(|| {
            let config = ProcessingConfig {
                buffer_size: 8 * 1024 * 1024, // 8MB buffer
                ..Default::default()
            };
            let _boundaries = strategy.process(black_box(&text_100mb), &config).unwrap();
        });
    });

    group.finish();
}

fn benchmark_buffer_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_buffer_sizes");
    group.sample_size(10);

    let text_50mb = generate_test_text(50);
    let rules = Arc::new(EnglishLanguageRules::new());
    let strategy = StreamingStrategy::new(rules);

    let buffer_sizes = vec![(1, "1MB"), (4, "4MB"), (8, "8MB"), (16, "16MB")];

    for (size_mb, label) in buffer_sizes {
        group.throughput(Throughput::Bytes(text_50mb.len() as u64));
        group.bench_function(label, |b| {
            b.iter(|| {
                let config = ProcessingConfig {
                    buffer_size: size_mb * 1024 * 1024,
                    ..Default::default()
                };
                let _boundaries = strategy.process(black_box(&text_50mb), &config).unwrap();
            });
        });
    }

    group.finish();
}

fn benchmark_streaming_vs_adaptive(c: &mut Criterion) {
    use sakurs_core::processing::AdaptiveProcessor;

    let mut group = c.benchmark_group("streaming_vs_adaptive");
    group.sample_size(10);

    let text_50mb = generate_test_text(50);
    let rules = Arc::new(EnglishLanguageRules::new());

    group.throughput(Throughput::Bytes(text_50mb.len() as u64));

    // Benchmark streaming strategy
    group.bench_function("streaming", |b| {
        let strategy = StreamingStrategy::new(rules.clone());
        b.iter(|| {
            let config = ProcessingConfig::default();
            let _boundaries = strategy.process(black_box(&text_50mb), &config).unwrap();
        });
    });

    // Benchmark adaptive processor
    group.bench_function("adaptive", |b| {
        let processor = AdaptiveProcessor::new(rules.clone());
        b.iter(|| {
            let _boundaries = processor.process(black_box(&text_50mb)).unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_streaming_small,
    benchmark_streaming_medium,
    benchmark_buffer_sizes,
    benchmark_streaming_vs_adaptive
);
criterion_main!(benches);
