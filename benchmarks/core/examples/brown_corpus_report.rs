//! Generate detailed accuracy report using Brown Corpus data

use sakurs_benchmarks::data::brown_corpus;
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};

fn main() {
    println!("\n=== Brown Corpus Accuracy Report ===\n");

    // Check if Brown Corpus data is available
    if !brown_corpus::is_available() {
        println!("⚠️  Brown Corpus data not found!");
        println!("   Please run the download script first:");
        println!("   cd benchmarks/data/brown_corpus && make download\n");

        println!("Running with small sample instead...\n");
        run_sample_test();
        return;
    }

    // Load and test with different corpus sizes
    println!("Loading Brown Corpus data...\n");

    let test_sizes = vec![
        ("Small (100 sentences)", 100),
        ("Medium (1000 sentences)", 1000),
        ("Large (10000 sentences)", 10000),
    ];

    let processor = create_default_processor();

    println!(
        "{:<25} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Test Size", "Precision", "Recall", "F1", "Pk", "WinDiff"
    );
    println!("{:-<85}", "");

    for (name, size) in test_sizes {
        match brown_corpus::load_subset(size) {
            Ok(test_data) => {
                let output = processor
                    .process_text(&test_data.text)
                    .expect("Processing should not fail");

                let predicted = extract_boundaries(&output);
                let metrics = calculate_complete_metrics(
                    &predicted,
                    &test_data.boundaries,
                    test_data.text.len(),
                );

                println!(
                    "{:<25} {:>9.1}% {:>9.1}% {:>9.1}% {:>10.4} {:>10.4}",
                    name,
                    metrics.precision * 100.0,
                    metrics.recall * 100.0,
                    metrics.f1_score * 100.0,
                    metrics.pk_score.unwrap_or(0.0),
                    metrics.window_diff.unwrap_or(0.0),
                );
            }
            Err(e) => {
                println!("{:<25} Error: {}", name, e);
            }
        }
    }

    // Test full corpus
    println!("\nLoading full corpus...");
    match brown_corpus::load_full_corpus() {
        Ok(test_data) => {
            let start = std::time::Instant::now();
            let output = processor
                .process_text(&test_data.text)
                .expect("Processing should not fail");
            let elapsed = start.elapsed();

            let predicted = extract_boundaries(&output);
            let metrics =
                calculate_complete_metrics(&predicted, &test_data.boundaries, test_data.text.len());

            println!("\n=== Full Corpus Results ===");
            println!("Sentences: {}", test_data.boundaries.len());
            println!("Characters: {}", test_data.text.len());
            println!("Processing time: {:.2}s", elapsed.as_secs_f32());
            println!("\nAccuracy Metrics:");
            println!("  Precision: {:.1}%", metrics.precision * 100.0);
            println!("  Recall: {:.1}%", metrics.recall * 100.0);
            println!("  F1 Score: {:.1}%", metrics.f1_score * 100.0);
            println!("  Pk Score: {:.4}", metrics.pk_score.unwrap_or(0.0));
            println!("  WindowDiff: {:.4}", metrics.window_diff.unwrap_or(0.0));

            // Analyze errors
            analyze_errors(&test_data.text, &predicted, &test_data.boundaries);
        }
        Err(e) => {
            println!("Error loading full corpus: {}", e);
        }
    }

    println!();
}

fn run_sample_test() {
    let processor = create_default_processor();
    let test_data = brown_corpus::small_sample();

    let output = processor
        .process_text(&test_data.text)
        .expect("Processing should not fail");

    let predicted = extract_boundaries(&output);
    let metrics =
        calculate_complete_metrics(&predicted, &test_data.boundaries, test_data.text.len());

    println!("Sample Results:");
    println!("  Precision: {:.1}%", metrics.precision * 100.0);
    println!("  Recall: {:.1}%", metrics.recall * 100.0);
    println!("  F1 Score: {:.1}%", metrics.f1_score * 100.0);
}

fn analyze_errors(text: &str, predicted: &[usize], actual: &[usize]) {
    use std::collections::HashSet;

    let predicted_set: HashSet<_> = predicted.iter().cloned().collect();
    let actual_set: HashSet<_> = actual.iter().cloned().collect();

    let false_positives: Vec<_> = predicted
        .iter()
        .filter(|&p| !actual_set.contains(p))
        .cloned()
        .collect();

    let false_negatives: Vec<_> = actual
        .iter()
        .filter(|&a| !predicted_set.contains(a))
        .cloned()
        .collect();

    println!("\n=== Error Analysis ===");
    println!(
        "False positives: {} ({:.1}%)",
        false_positives.len(),
        false_positives.len() as f32 / predicted.len() as f32 * 100.0
    );
    println!(
        "False negatives: {} ({:.1}%)",
        false_negatives.len(),
        false_negatives.len() as f32 / actual.len() as f32 * 100.0
    );

    // Show a few examples
    if !false_positives.is_empty() {
        println!("\nExample false positives (first 3):");
        for &pos in false_positives.iter().take(3) {
            show_context(text, pos, "FP");
        }
    }

    if !false_negatives.is_empty() {
        println!("\nExample false negatives (first 3):");
        for &pos in false_negatives.iter().take(3) {
            show_context(text, pos, "FN");
        }
    }
}

fn show_context(text: &str, pos: usize, error_type: &str) {
    let start = pos.saturating_sub(40);
    let end = (pos + 40).min(text.len());

    let context = &text[start..end];
    let relative_pos = pos - start;

    println!(
        "  {} at position {}: ...{}⟨HERE⟩{}...",
        error_type,
        pos,
        &context[..relative_pos],
        &context[relative_pos..]
    );
}
