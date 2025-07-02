//! Performance benchmarks for the application layer
//!
//! Run with: cargo bench --bench application_benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_core::application::{ProcessorConfig, TextProcessor};
use sakurs_core::domain::language::EnglishLanguageRules;
use std::sync::Arc;

/// Generate test text of specified size
fn generate_text(size: usize) -> String {
    let base_sentence = "This is a test sentence with some reasonable length. ";
    let sentence_len = base_sentence.len();
    let repeat_count = size / sentence_len + 1;

    let mut text = base_sentence.repeat(repeat_count);
    text.truncate(size);
    text
}

/// Benchmark sequential vs parallel processing
fn bench_processing_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("processing_modes");

    let rules = Arc::new(EnglishLanguageRules::new());

    for size in [1024, 10_240, 102_400, 1_024_000] {
        let text = generate_text(size);

        group.throughput(Throughput::Bytes(size as u64));

        // Sequential processing
        group.bench_with_input(BenchmarkId::new("sequential", size), &text, |b, text| {
            let mut config = ProcessorConfig::default();
            config.parallel_threshold = usize::MAX; // Force sequential
            let processor = TextProcessor::with_config(config, rules.clone());

            b.iter(|| {
                let _ = processor.process_text(black_box(text)).unwrap();
            });
        });

        // Parallel processing
        #[cfg(feature = "parallel")]
        group.bench_with_input(BenchmarkId::new("parallel", size), &text, |b, text| {
            let mut config = ProcessorConfig::default();
            config.parallel_threshold = 1024; // Low threshold for parallel
            config.chunk_size = 8192;
            let processor = TextProcessor::with_config(config, rules.clone());

            b.iter(|| {
                let _ = processor.process_text(black_box(text)).unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark different chunk sizes
fn bench_chunk_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_sizes");

    let rules = Arc::new(EnglishLanguageRules::new());
    let text = generate_text(100_000); // 100KB text

    group.throughput(Throughput::Bytes(text.len() as u64));

    for chunk_size in [1024, 4096, 16384, 65536] {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            &text,
            |b, text| {
                let mut config = ProcessorConfig::default();
                config.chunk_size = chunk_size;
                config.parallel_threshold = 50_000; // Enable parallel for this size
                let processor = TextProcessor::with_config(config, rules.clone());

                b.iter(|| {
                    let _ = processor.process_text(black_box(text)).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark different overlap sizes
fn bench_overlap_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("overlap_sizes");

    let rules = Arc::new(EnglishLanguageRules::new());
    let text = generate_text(50_000); // 50KB text

    group.throughput(Throughput::Bytes(text.len() as u64));

    for overlap_size in [0, 64, 256, 512] {
        group.bench_with_input(
            BenchmarkId::new("overlap", overlap_size),
            &text,
            |b, text| {
                let mut config = ProcessorConfig::default();
                config.chunk_size = 4096;
                config.overlap_size = overlap_size;
                let processor = TextProcessor::with_config(config, rules.clone());

                b.iter(|| {
                    let _ = processor.process_text(black_box(text)).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark streaming vs batch processing
fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    let rules = Arc::new(EnglishLanguageRules::new());
    let text = generate_text(50_000);

    group.throughput(Throughput::Bytes(text.len() as u64));

    // Batch processing
    group.bench_function("batch", |b| {
        let processor = TextProcessor::new(rules.clone());
        b.iter(|| {
            let _ = processor.process_text(black_box(&text)).unwrap();
        });
    });

    // Streaming processing
    group.bench_function("streaming", |b| {
        let processor = TextProcessor::new(rules.clone());
        b.iter(|| {
            // Simulate streaming with 1KB chunks
            let chunks: Vec<String> = text
                .as_bytes()
                .chunks(1024)
                .map(|chunk| String::from_utf8_lossy(chunk).to_string())
                .collect();

            let _ = processor.process_streaming(chunks.into_iter()).unwrap();
        });
    });

    group.finish();
}

/// Benchmark text with different language complexities
fn bench_text_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_complexity");

    let rules = Arc::new(EnglishLanguageRules::new());

    // Simple text - just periods
    let simple_text = "This is a sentence. ".repeat(1000);

    // Complex text - abbreviations, quotes, numbers
    let complex_text =
        r#"Dr. Smith said, "The U.S. economy grew 3.5% in Q1." Mr. Jones disagreed. "#.repeat(500);

    // Very complex - nested quotes, many abbreviations
    let very_complex = r#"The C.E.O. announced, "Our Q4 earnings of $1.2M (a 15% increase) exceeded expectations." The C.F.O. added, "We're projecting 20% growth." "#.repeat(300);

    let texts = [
        ("simple", simple_text),
        ("complex", complex_text),
        ("very_complex", very_complex),
    ];

    for (name, text) in texts {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::new("complexity", name), &text, |b, text| {
            let processor = TextProcessor::new(rules.clone());
            b.iter(|| {
                let _ = processor.process_text(black_box(text)).unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    let rules = Arc::new(EnglishLanguageRules::new());

    // Test different text sizes to observe memory scaling
    for size_mb in [1, 5, 10] {
        let size = size_mb * 1024 * 1024;
        let text = generate_text(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("memory", format!("{}MB", size_mb)),
            &text,
            |b, text| {
                let processor = TextProcessor::new(rules.clone());
                b.iter(|| {
                    let _ = processor.process_text(black_box(text)).unwrap();
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "parallel")]
/// Benchmark thread scaling
fn bench_thread_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_scaling");

    let rules = Arc::new(EnglishLanguageRules::new());
    let text = generate_text(1_000_000); // 1MB text

    group.throughput(Throughput::Bytes(text.len() as u64));

    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &text,
            |b, text| {
                let mut config = ProcessorConfig::default();
                config.max_threads = Some(num_threads);
                config.parallel_threshold = 1024; // Low threshold
                config.chunk_size = text.len() / (num_threads * 4); // Ensure enough chunks

                let processor = TextProcessor::with_config(config, rules.clone());

                b.iter(|| {
                    let _ = processor.process_text(black_box(text)).unwrap();
                });
            },
        );
    }

    group.finish();
}

// Configure criterion
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets =
        bench_processing_modes,
        bench_chunk_sizes,
        bench_overlap_sizes,
        bench_streaming,
        bench_text_complexity,
        bench_memory_patterns
}

#[cfg(feature = "parallel")]
criterion_group! {
    name = parallel_benches;
    config = Criterion::default().sample_size(10);
    targets = bench_thread_scaling
}

#[cfg(not(feature = "parallel"))]
criterion_main!(benches);

#[cfg(feature = "parallel")]
criterion_main!(benches, parallel_benches);
