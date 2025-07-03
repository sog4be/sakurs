//! Generate accuracy report for sakurs

use sakurs_benchmarks::data::{brown_corpus, generators};
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};

fn main() {
    println!("\n=== Sakurs Accuracy Report ===\n");

    let processor = create_default_processor();

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

        // Extract boundaries and calculate metrics
        let predicted = extract_boundaries(&output);
        let metrics =
            calculate_complete_metrics(&predicted, &test_data.boundaries, test_data.text.len());

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
