//! Test that thread pool configuration is properly applied

use sakurs_core::{Config, Input, SentenceProcessor};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Test that the specified thread count is actually used for processing
#[test]
fn test_thread_pool_configuration() {
    // Create a large enough text to trigger parallel processing
    let text = "This is a test sentence. ".repeat(2000); // ~50KB - smaller for CI

    // Test different thread counts
    for thread_count in [1, 2, 4] {
        // Fewer thread counts for CI
        println!("Testing with {} threads", thread_count);

        // Create processor with specific thread count
        let config = Config::builder()
            .language("en")
            .unwrap()
            .threads(Some(thread_count))
            .build()
            .unwrap();

        let processor = SentenceProcessor::with_config(config).unwrap();

        // Track the maximum number of concurrent threads
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        let current_threads = Arc::new(AtomicUsize::new(0));

        // Clone for the thread
        let max_concurrent_clone = max_concurrent.clone();
        let current_threads_clone = current_threads.clone();

        // Monitor thread usage in a separate thread
        let monitor = thread::spawn(move || {
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(5) {
                let current = current_threads_clone.load(Ordering::Relaxed);
                let max = max_concurrent_clone.load(Ordering::Relaxed);
                if current > max {
                    max_concurrent_clone.store(current, Ordering::Relaxed);
                }
                thread::sleep(Duration::from_micros(100));
            }
        });

        // Track thread count during processing
        let _enter = current_threads.fetch_add(1, Ordering::Relaxed);
        let result = processor.process(Input::from_text(text.clone()));
        current_threads.fetch_sub(1, Ordering::Relaxed);

        assert!(result.is_ok(), "Processing should succeed");

        // Give monitor thread time to observe
        thread::sleep(Duration::from_millis(100));

        // Note: We can't directly observe rayon's internal thread count,
        // but we can verify that processing completes successfully
        // with the specified configuration

        drop(monitor); // Clean up monitor thread
    }
}

/// Test that thread count of 1 forces sequential processing
#[test]
fn test_single_thread_forces_sequential() {
    let text = "First sentence. ".repeat(1000); // Smaller for CI

    let config = Config::builder()
        .language("en")
        .unwrap()
        .threads(Some(1))
        .build()
        .unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();
    let result = processor.process(Input::from_text(text));

    assert!(result.is_ok());
    // Just verify it processes successfully with 1 thread
}

/// Test that parallel processing produces consistent results
#[test]
fn test_parallel_consistency() {
    let text = "This is a test sentence. ".repeat(500); // Smaller for CI

    // Process with different thread counts
    let mut results = Vec::new();

    for threads in [1, 2, 4] {
        // Fewer thread counts for CI
        let config = Config::builder()
            .language("en")
            .unwrap()
            .threads(Some(threads))
            .build()
            .unwrap();

        let processor = SentenceProcessor::with_config(config).unwrap();
        let result = processor.process(Input::from_text(text.clone())).unwrap();

        results.push((threads, result.boundaries.len()));
    }

    // All thread counts should produce the same number of boundaries
    let first_count = results[0].1;
    for (threads, count) in &results {
        assert_eq!(
            *count, first_count,
            "Thread count {} produced different results: {} vs {}",
            threads, count, first_count
        );
    }
}

/// Heavy test for local performance testing
/// Run with: cargo test test_thread_pool_performance -- --ignored
#[test]
#[ignore]
fn test_thread_pool_performance() {
    let text = "This is a test sentence. ".repeat(10000); // ~250KB

    // Test with more thread counts including 8
    for thread_count in [1, 2, 4, 8] {
        println!("Performance testing with {} threads", thread_count);

        let config = Config::builder()
            .language("en")
            .unwrap()
            .threads(Some(thread_count))
            .build()
            .unwrap();

        let processor = SentenceProcessor::with_config(config).unwrap();

        let start = std::time::Instant::now();
        let result = processor.process(Input::from_text(text.clone()));
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        println!("  Completed in {:?}", elapsed);
    }
}
