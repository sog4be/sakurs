//! Focused benchmark for identifying non-linear scaling bottlenecks
//!
//! Run with: cargo bench --bench scaling_bottleneck_analysis

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sakurs_core::{Config, Input, SentenceProcessor};
use std::hint::black_box;

/// Generate realistic test text for scaling analysis
fn generate_realistic_text(base_text: &str, multiplier: usize) -> String {
    // Use spaces between repetitions for more realistic text structure
    vec![base_text; multiplier].join(" ")
}

/// Realistic Japanese text for testing
const JAPANESE_BASE: &str = "文分割OSSの動作確認用サンプルとして、本稿では多様な構文や句読点を織り交ぜた四百文字ぴったりの文章を提示する。まず、冒頭で目的を簡潔に述べ、続けて条件を満たす特殊な鉤括弧表現を挿入する。それが「この節では、システムが正しく文を区切れるかを試すために『入れ子構造が含まれる文です。』と宣言し、内部にも句点を置いた。」という部分だ。さらに、助詞の省略や倒置を利用して自然な日本語を維持しつつ、語彙の重複を避ける。また、読点と中黒を適切に配し、視認性を向上させる。ここまでで約三百文字に満たないため、さらに字数を稼ぐ工夫として、典型的な敬語、引用、列挙の語法も盛り込もう。例えば、開発者は「期待どおりに区切られましたか？」と問い掛け、テスターは「はい、問題ありません！」と応じる対話を想定する。こうした会話体は解析器にとっても挑戦的であり、数字1と英字Aを挿入して補うことで正確性の検証に寄与するだろう！";

/// Realistic English text for testing
const ENGLISH_BASE: &str = "Mr. Baker, a coder from the U.S., drafted the following line: \"Parser ready (v2.3 passes.) now.\" Can the server at 192.168.1.1 parse every case? Yes! Watch it stumble on sequences like e.g. ellipses... or does it!? Despite surprises, the module logs \"Done (all tests ok.)\" before midnight. Each token rides its boundary, yet pesky abbreviations lurk: Prof., Dr., St., etc., all set to trip splitters.";

/// Benchmark scaling behavior with Japanese text (focused on problematic range)
fn bench_japanese_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("japanese_scaling");
    group.sample_size(20); // Reduce sample size for faster completion

    let japanese_config = Config::builder().language("ja").unwrap().build().unwrap();
    let processor = SentenceProcessor::with_config(japanese_config).unwrap();

    // Focus on the range where non-linear behavior starts (10x to 50x multipliers)
    for multiplier in [1, 5, 10, 15, 20, 25, 30, 40, 50] {
        let text = if multiplier == 1 {
            JAPANESE_BASE.to_string()
        } else {
            JAPANESE_BASE.repeat(multiplier) // No spaces for Japanese
        };

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("multiplier", multiplier),
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

/// Benchmark scaling behavior with English text
fn bench_english_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("english_scaling");
    group.sample_size(20);

    let processor = SentenceProcessor::new(); // Default English

    // Focus on same range for comparison
    for multiplier in [1, 5, 10, 15, 20, 25] {
        let text = if multiplier == 1 {
            ENGLISH_BASE.to_string()
        } else {
            vec![ENGLISH_BASE; multiplier].join(" ") // Spaces for English
        };

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("multiplier", multiplier),
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

/// Benchmark different text sizes in a more controlled manner
fn bench_controlled_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("controlled_sizes");
    group.sample_size(20);

    let processor = SentenceProcessor::new();

    // Test specific sizes where we observed non-linear behavior
    for size in [400, 2000, 4000, 8000, 16000, 32000] {
        // Generate text of exact size
        let base = "This is a test sentence. ";
        let repeat_count = size / base.len() + 1;
        let mut text = base.repeat(repeat_count);
        text.truncate(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::new("size_bytes", size), &text, |b, text| {
            b.iter(|| {
                let _ = processor
                    .process(Input::from_text(black_box(text)))
                    .unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_japanese_scaling,
    bench_english_scaling,
    bench_controlled_sizes
);
criterion_main!(benches);
