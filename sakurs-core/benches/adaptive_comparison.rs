//! Benchmark comparing adaptive vs standard processing

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_core::application::{ProcessorConfig, TextProcessor};
use sakurs_core::domain::language::EnglishLanguageRules;
use std::hint::black_box;
use std::sync::Arc;

/// Generate test text of specified size
fn generate_text(size_kb: usize) -> String {
    let base_text = "This is a test sentence. It has multiple words! Does it work? ";
    let base_len = base_text.len();
    let target_size = size_kb * 1024;
    let repeat_count = target_size / base_len + 1;
    base_text.repeat(repeat_count)[..target_size].to_string()
}

/// Compare adaptive vs standard processing
fn bench_adaptive_vs_standard(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_vs_standard");

    let sizes_kb = vec![10, 50, 100, 500, 1000, 5000];
    let rules = Arc::new(EnglishLanguageRules::new());

    for size_kb in sizes_kb {
        let text = generate_text(size_kb);
        group.throughput(Throughput::Bytes(text.len() as u64));

        // Standard processing
        group.bench_with_input(
            BenchmarkId::new("standard", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let processor = TextProcessor::new(rules.clone());
                b.iter(|| processor.process_text(black_box(text)));
            },
        );

        // Adaptive processing
        group.bench_with_input(
            BenchmarkId::new("adaptive", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let processor = TextProcessor::new(rules.clone());
                b.iter(|| processor.process_text_adaptive(black_box(text)));
            },
        );
    }

    group.finish();
}

/// Compare with forced parallel processing
fn bench_adaptive_vs_forced_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_vs_forced_parallel");

    let sizes_kb = vec![10, 50, 100]; // Small files where adaptive should be better
    let rules = Arc::new(EnglishLanguageRules::new());

    for size_kb in sizes_kb {
        let text = generate_text(size_kb);
        group.throughput(Throughput::Bytes(text.len() as u64));

        // Forced parallel
        group.bench_with_input(
            BenchmarkId::new("forced_parallel", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let config = ProcessorConfig::builder()
                    .parallel_threshold(0) // Force parallel
                    .build()
                    .expect("Valid config");
                let processor = TextProcessor::with_config(config, rules.clone());
                b.iter(|| processor.process_text(black_box(text)));
            },
        );

        // Adaptive processing
        group.bench_with_input(
            BenchmarkId::new("adaptive", format!("{}KB", size_kb)),
            &text,
            |b, text| {
                let processor = TextProcessor::new(rules.clone());
                b.iter(|| processor.process_text_adaptive(black_box(text)));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_adaptive_vs_standard,
    bench_adaptive_vs_forced_parallel
);
criterion_main!(benches);
