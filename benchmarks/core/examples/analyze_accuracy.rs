//! Analyze sakurs accuracy issues with Brown Corpus data
//!
//! This tool investigates why sakurs accuracy drops significantly
//! as dataset size increases.

use sakurs_benchmarks::config;
use sakurs_benchmarks::data::brown_corpus;
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};
use std::collections::HashMap;

fn analyze_boundary_patterns(
    text: &str,
    actual_boundaries: &[usize],
    predicted_boundaries: &[usize],
) {
    println!("\n=== Boundary Pattern Analysis ===");

    // Analyze first 10 actual boundaries
    println!("\nFirst 10 actual boundaries:");
    for (i, &boundary) in actual_boundaries.iter().take(10).enumerate() {
        let start = boundary.saturating_sub(20);
        let end = (boundary + 20).min(text.len());
        let context = &text[start..end];
        let offset = boundary - start;

        println!(
            "  {}. Position {}: ...{}|{}...",
            i + 1,
            boundary,
            &context[..offset],
            &context[offset..]
        );
    }

    // Analyze first 10 predicted boundaries
    println!("\nFirst 10 predicted boundaries:");
    for (i, &boundary) in predicted_boundaries.iter().take(10).enumerate() {
        let start = boundary.saturating_sub(20);
        let end = (boundary + 20).min(text.len());
        let context = &text[start..end];
        let offset = boundary - start;

        println!(
            "  {}. Position {}: ...{}|{}...",
            i + 1,
            boundary,
            &context[..offset],
            &context[offset..]
        );
    }

    // Find common false positives
    let actual_set: std::collections::HashSet<_> = actual_boundaries.iter().collect();
    let false_positives: Vec<_> = predicted_boundaries
        .iter()
        .filter(|b| !actual_set.contains(b))
        .take(10)
        .collect();

    println!("\nFirst 10 false positive boundaries:");
    for (i, &&boundary) in false_positives.iter().enumerate() {
        let start = boundary.saturating_sub(20);
        let end = (boundary + 20).min(text.len());
        let context = &text[start..end];
        let offset = boundary - start;

        println!(
            "  {}. Position {}: ...{}|{}...",
            i + 1,
            boundary,
            &context[..offset],
            &context[offset..]
        );
    }

    // Find common false negatives
    let predicted_set: std::collections::HashSet<_> = predicted_boundaries.iter().collect();
    let false_negatives: Vec<_> = actual_boundaries
        .iter()
        .filter(|b| !predicted_set.contains(b))
        .take(10)
        .collect();

    println!("\nFirst 10 false negative boundaries (missed):");
    for (i, &&boundary) in false_negatives.iter().enumerate() {
        let start = boundary.saturating_sub(20);
        let end = (boundary + 20).min(text.len());
        let context = &text[start..end];
        let offset = boundary - start;

        println!(
            "  {}. Position {}: ...{}|{}...",
            i + 1,
            boundary,
            &context[..offset],
            &context[offset..]
        );
    }
}

fn analyze_text_characteristics(text: &str, boundaries: &[usize]) {
    println!("\n=== Text Characteristics ===");

    // Calculate average sentence length
    let mut sentence_lengths = Vec::new();
    let mut last_pos = 0;

    for &boundary in boundaries {
        let length = boundary - last_pos;
        sentence_lengths.push(length);
        last_pos = boundary;
    }

    // Add last sentence
    if last_pos < text.len() {
        sentence_lengths.push(text.len() - last_pos);
    }

    let avg_length = sentence_lengths.iter().sum::<usize>() as f64 / sentence_lengths.len() as f64;
    let min_length = sentence_lengths.iter().min().copied().unwrap_or(0);
    let max_length = sentence_lengths.iter().max().copied().unwrap_or(0);

    println!("Sentence length statistics:");
    println!("  Average: {:.1} characters", avg_length);
    println!("  Min: {} characters", min_length);
    println!("  Max: {} characters", max_length);
    println!("  Total sentences: {}", boundaries.len());

    // Analyze punctuation patterns
    let mut punct_counts = HashMap::new();
    for ch in text.chars() {
        if ch.is_ascii_punctuation() {
            *punct_counts.entry(ch).or_insert(0) += 1;
        }
    }

    println!("\nPunctuation frequency:");
    let mut sorted_puncts: Vec<_> = punct_counts.into_iter().collect();
    sorted_puncts.sort_by_key(|&(_, count)| std::cmp::Reverse(count));

    for (punct, count) in sorted_puncts.iter().take(10) {
        println!("  '{}': {} occurrences", punct, count);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing sakurs accuracy with Brown Corpus data\n");

    // Check if Brown Corpus is available
    if !brown_corpus::is_available() {
        eprintln!("❌ Brown Corpus data not available.");
        eprintln!("   Please run: cd benchmarks/data/brown_corpus && make download");
        return Ok(());
    }

    let processor = create_default_processor();
    let subset_sizes = config::get_subset_sizes(false);

    for size in subset_sizes {
        println!("\n{}", "=".repeat(60));
        println!("Analyzing {} sentence dataset", size);
        println!("{}", "=".repeat(60));

        // Load test data
        let test_data = brown_corpus::load_subset(size)?;

        // Process text
        let output = processor
            .process_text(&test_data.text)
            .expect("Processing should not fail");

        let predicted = extract_boundaries(&output);
        let metrics =
            calculate_complete_metrics(&predicted, &test_data.boundaries, test_data.text.len());

        // Print metrics
        println!("\nAccuracy Metrics:");
        println!("  Precision: {:.2}%", metrics.precision * 100.0);
        println!("  Recall: {:.2}%", metrics.recall * 100.0);
        println!("  F1 Score: {:.2}%", metrics.f1_score * 100.0);
        println!("  True Positives: {}", metrics.true_positives);
        println!("  False Positives: {}", metrics.false_positives);
        println!("  False Negatives: {}", metrics.false_negatives);
        println!("  Predicted boundaries: {}", predicted.len());
        println!("  Actual boundaries: {}", test_data.boundaries.len());

        // Analyze text characteristics
        analyze_text_characteristics(&test_data.text, &test_data.boundaries);

        // Analyze boundary patterns
        analyze_boundary_patterns(&test_data.text, &test_data.boundaries, &predicted);

        // Sample text preview
        println!("\nText preview (first 500 chars):");
        println!("{}", &test_data.text[..500.min(test_data.text.len())]);
    }

    println!("\n✅ Analysis complete!");

    Ok(())
}
