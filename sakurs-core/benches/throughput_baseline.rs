//! Baseline throughput benchmarks across text characteristics and chunk sizes.
//!
//! These benchmarks track the end-to-end cost of the scan pipeline. They are
//! intentionally small and deterministic so they can be run manually before
//! and after performance work:
//!
//! ```bash
//! cargo bench --bench throughput_baseline
//! ```
//!
//! Not CI-gated (machine variance would make a hard threshold flaky). Compare
//! criterion's saved baselines instead: `cargo bench --bench throughput_baseline -- --save-baseline <name>`.
//!
//! v0.1.1 context: throughput currently *decreases* as chunk size grows,
//! because several per-terminator / per-enclosure code paths do O(chunk_size)
//! work (full-chunk copies and re-decodes). The `single_chunk` cases pin that
//! quadratic path explicitly; they are the numbers that must improve by orders
//! of magnitude with the v0.2.0 scanner redesign.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_core::{Config, Input, SentenceProcessor};
use std::time::Duration;

const PLAIN_UNIT: &str = "The quick brown fox jumps over the lazy dog near the river bank. \
It was a sunny day and everyone was happy about the weather. \
She walked to the store to buy some fresh bread and milk. ";

const QUOTES_UNIT: &str =
    "He said \"Hello there my friend.\" Then (quite slowly) he left the room. \
She replied \"I will see you tomorrow.\" The others (all of them) nodded in agreement. ";

const ABBR_UNIT: &str = "Dr. Smith met Mr. Jones at the U.S. embassy near St. Paul street. \
Prof. Brown earned his Ph.D. in 1990 from the university. \
The company Inc. reported profits of $2.5 billion this year. ";

fn repeat_to(unit: &str, target: usize) -> String {
    let mut s = String::with_capacity(target + unit.len());
    while s.len() < target {
        s.push_str(unit);
    }
    s
}

fn processor(chunk_size: usize) -> SentenceProcessor {
    let config = Config::builder()
        .language("en")
        .expect("language config should load")
        .chunk_size(chunk_size)
        .threads(Some(1))
        .build()
        .expect("config should validate");
    SentenceProcessor::with_config(config).expect("processor should build")
}

fn bench_throughput(c: &mut Criterion) {
    let size = 256 * 1024;
    let cases: [(&str, &str); 3] = [
        ("plain", PLAIN_UNIT),
        ("quotes", QUOTES_UNIT),
        ("abbr", ABBR_UNIT),
    ];

    let mut group = c.benchmark_group("throughput_baseline");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(8))
        .warm_up_time(Duration::from_secs(1));
    group.throughput(Throughput::Bytes(size as u64));

    for (name, unit) in cases {
        let text = repeat_to(unit, size);

        // Multi-chunk configurations (the production-relevant path).
        for chunk_kb in [16usize, 64] {
            let p = processor(chunk_kb * 1024);
            group.bench_with_input(
                BenchmarkId::new(name, format!("chunk_{chunk_kb}k")),
                &text,
                |b, t| {
                    b.iter(|| {
                        p.process(Input::from_text(t.clone()))
                            .expect("processing should succeed")
                    })
                },
            );
        }

        // Single-chunk configuration: pins the O(chunk_size)-per-terminator
        // cost. Uses a smaller text so v0.1.1 timings stay tractable.
        let small = repeat_to(unit, 64 * 1024);
        let p = processor(small.len() + 1024);
        group.throughput(Throughput::Bytes(small.len() as u64));
        group.bench_with_input(
            BenchmarkId::new(name, "single_chunk_64k"),
            &small,
            |b, t| {
                b.iter(|| {
                    p.process(Input::from_text(t.clone()))
                        .expect("processing should succeed")
                })
            },
        );
        group.throughput(Throughput::Bytes(size as u64));
    }

    group.finish();
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
