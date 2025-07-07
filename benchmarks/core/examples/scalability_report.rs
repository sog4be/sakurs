//! Generate scalability report for sakurs

use sakurs_benchmarks::constants::{text_sizes, DEFAULT_PARALLEL_THRESHOLD, THREAD_COUNTS};
use sakurs_benchmarks::create_processor_with_config;
use sakurs_benchmarks::data::generators;
use sakurs_core::application::ProcessorConfig;
use std::time::Instant;

fn main() {
    println!("\n=== Sakurs Scalability Report ===\n");

    let test_data = generators::large_text(text_sizes::HUGE);
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

    for &thread_count in THREAD_COUNTS {
        if thread_count > cpu_count {
            continue;
        }

        let config = ProcessorConfig {
            max_threads: Some(thread_count),
            parallel_threshold: if thread_count == 1 {
                usize::MAX
            } else {
                DEFAULT_PARALLEL_THRESHOLD
            },
            ..Default::default()
        };

        let processor = create_processor_with_config(config);

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
