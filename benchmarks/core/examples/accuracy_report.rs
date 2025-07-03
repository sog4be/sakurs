//! Generate accuracy report for sakurs

use sakurs_benchmarks::data::{brown_corpus, generators};
use sakurs_benchmarks::metrics::{calculate_pk_score, calculate_window_diff};
use sakurs_benchmarks::{calculate_accuracy_metrics, AccuracyMetrics};
use sakurs_core::application::TextProcessor;
use sakurs_core::domain::language::EnglishLanguageRules;
use std::sync::Arc;

fn main() {
    println!("\n=== Sakurs Accuracy Report ===\n");

    let rules = Arc::new(EnglishLanguageRules::new());
    let processor = TextProcessor::new(rules);

    let test_cases = vec![
        ("Simple Sentences", generators::simple_sentences(10)),
        ("With Abbreviations", generators::with_abbreviations()),
        ("With Quotations", generators::with_quotations()),
        ("With Numbers", generators::with_numbers()),
        ("Complex Mixed", generators::complex_mixed()),
        ("Brown Corpus Sample", brown_corpus::small_sample()),
    ];

    println!(
        "{:<20} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Test Case", "Precision", "Recall", "F1", "Pk", "WinDiff"
    );
    println!("{:-<80}", "");

    for (name, test_data) in test_cases {
        test_data.validate().expect("Test data should be valid");

        // Process text
        let output = processor
            .process_text(&test_data.text)
            .expect("Processing should not fail");

        // Convert boundaries to positions
        let predicted: Vec<usize> = output.boundaries.iter().map(|b| b.offset).collect();

        // Calculate metrics
        let mut metrics = calculate_accuracy_metrics(&predicted, &test_data.boundaries);

        // Add Pk and WindowDiff
        let pk = calculate_pk_score(
            &predicted,
            &test_data.boundaries,
            test_data.text.len(),
            None,
        );
        metrics = metrics.with_pk_score(pk);

        let wd = calculate_window_diff(
            &predicted,
            &test_data.boundaries,
            test_data.text.len(),
            None,
        );
        metrics = metrics.with_window_diff(wd);

        // Print results
        println!(
            "{:<20} {:>9.1}% {:>9.1}% {:>9.1}% {:>10.4} {:>10.4}",
            name,
            metrics.precision * 100.0,
            metrics.recall * 100.0,
            metrics.f1_score * 100.0,
            metrics.pk_score.unwrap_or(0.0),
            metrics.window_diff.unwrap_or(0.0),
        );
    }

    println!("\n");
}
