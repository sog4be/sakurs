//! Benchmark to compare performance before and after SmallVec optimization

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sakurs_core::{
    application::parser::scan_chunk,
    domain::{language::EnglishLanguageRules, state::PartialState, MonoidReduce},
};
use std::hint::black_box;

/// Generate test text with specified number of sentences
fn generate_text(num_sentences: usize) -> String {
    let sentences = vec![
        "This is a simple sentence.",
        "Dr. Smith works at the hospital.",
        "The company earned $1.5 million last year.",
        "She said \"Hello!\" and waved.",
        "The meeting is at 3:00 p.m. today.",
    ];

    sentences
        .iter()
        .cycle()
        .take(num_sentences)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn benchmark_scan_phase(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_phase");
    let rules = EnglishLanguageRules::new();

    for size in [10, 50, 100, 500].iter() {
        let text = generate_text(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &text, |b, text| {
            b.iter(|| {
                let _state = scan_chunk(black_box(text), &rules);
            });
        });
    }

    group.finish();
}

fn benchmark_monoid_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("monoid_reduce");
    let rules = EnglishLanguageRules::new();

    for num_chunks in [2, 4, 8, 16].iter() {
        let chunk_text = generate_text(20);
        let states: Vec<PartialState> = (0..*num_chunks)
            .map(|_| scan_chunk(&chunk_text, &rules))
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(num_chunks),
            &states,
            |b, states| {
                b.iter(|| {
                    let _result = PartialState::reduce(black_box(states.clone()));
                });
            },
        );
    }

    group.finish();
}

fn benchmark_memory_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocations");
    let rules = EnglishLanguageRules::new();

    // Test with different text sizes to see allocation patterns
    for size_kb in [1, 10, 64, 128].iter() {
        let text_size = size_kb * 1024;
        let text = generate_text(text_size / 30); // Roughly 30 chars per sentence

        group.bench_with_input(
            BenchmarkId::new("scan_chunk", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                b.iter(|| {
                    let _state = scan_chunk(black_box(text), &rules);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_parallel_processing(c: &mut Criterion) {
    use sakurs_core::application::TextProcessor;
    use std::sync::Arc;

    let mut group = c.benchmark_group("parallel_processing");
    let rules = Arc::new(EnglishLanguageRules::new());
    let processor = TextProcessor::new(rules);

    // Test different file sizes to see impact of SmallVec on parallel performance
    for size_mb in [1, 5, 10].iter() {
        let text_size = size_mb * 1024 * 1024;
        let text = generate_text(text_size / 30);

        group.bench_with_input(
            BenchmarkId::new("process_text", format!("{}MB", size_mb)),
            &text,
            |b, text| {
                b.iter(|| {
                    let _sentences = processor.process_text(black_box(text));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_scan_phase,
    benchmark_monoid_reduce,
    benchmark_memory_allocations,
    benchmark_parallel_processing
);
criterion_main!(benches);
