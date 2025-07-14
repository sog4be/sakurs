//! Performance benchmarks for SentenceProcessor
//!
//! Run with: cargo bench --bench sentence_processor_benchmarks

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_core::{Config, Input, SentenceProcessor};
use std::hint::black_box;

/// Generate test text of specified size
fn generate_text(size: usize) -> String {
    let base_sentence = "This is a test sentence with some reasonable length. ";
    let sentence_len = base_sentence.len();
    let repeat_count = size / sentence_len + 1;

    let mut text = base_sentence.repeat(repeat_count);
    text.truncate(size);
    text
}

/// Benchmark different text sizes
fn bench_text_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_sizes");

    let processor = SentenceProcessor::new();

    for size in [1024, 10_240, 102_400, 1_024_000] {
        let text = generate_text(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("process", size), &text, |b, text| {
            b.iter(|| {
                let _ = processor
                    .process(Input::from_text(black_box(text)))
                    .unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark parallel processing with different thread counts
fn bench_thread_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_counts");

    // Use a large text to ensure parallel processing is triggered
    let text = generate_text(1_024_000); // 1MB

    for threads in [1, 2, 4, 8] {
        let config = Config::builder()
            .language("en")
            .unwrap()
            .threads(Some(threads))
            .build()
            .unwrap();

        let processor = SentenceProcessor::with_config(config).unwrap();

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::new("threads", threads), &text, |b, text| {
            b.iter(|| {
                let _ = processor
                    .process(Input::from_text(black_box(text)))
                    .unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark different languages
fn bench_languages(c: &mut Criterion) {
    let mut group = c.benchmark_group("languages");

    let text_size = 102_400; // 100KB

    // English
    let english_text = generate_text(text_size);
    let english_processor = SentenceProcessor::new();

    group.throughput(Throughput::Bytes(text_size as u64));
    group.bench_with_input(
        BenchmarkId::new("language", "english"),
        &english_text,
        |b, text| {
            b.iter(|| {
                let _ = english_processor
                    .process(Input::from_text(black_box(text)))
                    .unwrap();
            });
        },
    );

    // Japanese
    let japanese_text = "これはテスト文です。".repeat(text_size / 30); // Approximate size
    let japanese_config = Config::builder().language("ja").unwrap().build().unwrap();
    let japanese_processor = SentenceProcessor::with_config(japanese_config).unwrap();

    group.throughput(Throughput::Bytes(japanese_text.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("language", "japanese"),
        &japanese_text,
        |b, text| {
            b.iter(|| {
                let _ = japanese_processor
                    .process(Input::from_text(black_box(text)))
                    .unwrap();
            });
        },
    );

    group.finish();
}

/// Benchmark chunking efficiency
fn bench_chunk_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_sizes");

    let text = generate_text(1_024_000); // 1MB

    for chunk_size in [4096, 16_384, 65_536, 262_144] {
        let config = Config::builder()
            .language("en")
            .unwrap()
            .chunk_size(chunk_size)
            .build()
            .unwrap();

        let processor = SentenceProcessor::with_config(config).unwrap();

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            &text,
            |b, text| {
                b.iter(|| {
                    let _ = processor
                        .process(Input::from_text(black_box(text)))
                        .unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_text_sizes,
    bench_thread_counts,
    bench_languages,
    bench_chunk_sizes
);
criterion_main!(benches);
