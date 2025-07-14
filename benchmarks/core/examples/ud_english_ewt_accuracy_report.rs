//! UD English EWT accuracy analysis example
//!
//! This example analyzes sakurs accuracy on UD English EWT dataset
//! and compares with Brown Corpus results.

use sakurs_benchmarks::data::{brown_corpus, ud_english_ewt};
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 UD English EWT vs Brown Corpus Accuracy Analysis");
    println!("{}", "=".repeat(60));

    // Test both datasets if available
    analyze_ud_english_ewt()?;
    println!();
    analyze_brown_corpus()?;

    Ok(())
}

fn analyze_ud_english_ewt() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 UD English EWT Analysis");
    println!("{}", "-".repeat(30));

    if !ud_english_ewt::is_available() {
        println!("❌ UD English EWT data not available");
        println!("   Run: cd benchmarks/data/ud_english_ewt && python download.py");
        return Ok(());
    }

    let processor = create_default_processor();

    // Test different subset sizes
    let sizes = vec![1]; // Limited to test data available

    for size in sizes {
        match ud_english_ewt::load_subset(size) {
            Ok(test_data) => {
                let start = Instant::now();
                let result = processor.process(sakurs_core::Input::from_text(test_data.text))?;
                let processing_time = start.elapsed();

                let predicted_boundaries = extract_boundaries(&result);
                let metrics = calculate_complete_metrics(
                    &predicted_boundaries,
                    &test_data.boundaries,
                    test_data.text.len(),
                );

                println!("\n🎯 UD English EWT Subset ({} sentences):", size);
                println!("   📝 Text length: {} characters", test_data.text.len());
                println!(
                    "   🎯 Expected boundaries: {} (ground truth)",
                    test_data.boundaries.len()
                );
                println!("   🔍 Predicted boundaries: {}", predicted_boundaries.len());
                println!("   ⚡ Processing time: {:?}", processing_time);
                println!("   📊 Metrics:");
                println!("      • F1 Score: {:.4}", metrics.f1_score);
                println!("      • Precision: {:.4}", metrics.precision);
                println!("      • Recall: {:.4}", metrics.recall);
                println!("      • Pk Score: {:.4}", metrics.pk_score.unwrap_or(0.0));
                println!(
                    "      • WindowDiff: {:.4}",
                    metrics.window_diff.unwrap_or(0.0)
                );

                if metrics.f1_score < 0.9 {
                    println!("   ⚠️  Low F1 score detected (< 0.9)");
                }

                // Show boundary details for small samples
                if test_data.boundaries.len() <= 5 {
                    println!("   🔍 Boundary Details:");
                    println!("      Expected: {:?}", test_data.boundaries);
                    println!("      Predicted: {:?}", predicted_boundaries);
                }
            }
            Err(e) => {
                println!("❌ Error loading UD English EWT subset {}: {}", size, e);
            }
        }
    }

    // Test the hardcoded sample
    let sample = ud_english_ewt::small_sample();
    let start = Instant::now();
    let result = processor.process(sakurs_core::Input::from_text(sample.text))?;
    let processing_time = start.elapsed();

    let predicted_boundaries = extract_boundaries(&result);
    let metrics =
        calculate_complete_metrics(&predicted_boundaries, &sample.boundaries, sample.text.len());

    println!("\n🎯 UD English EWT Hardcoded Sample:");
    println!("   📝 Text: \"{}\"", sample.text));
    println!("   📝 Text length: {} characters", sample.text.len());
    println!("   🎯 Expected boundaries: {:?}", sample.boundaries);
    println!("   🔍 Predicted boundaries: {:?}", predicted_boundaries);
    println!("   ⚡ Processing time: {:?}", processing_time);
    println!("   📊 Metrics:");
    println!("      • F1 Score: {:.4}", metrics.f1_score);
    println!("      • Precision: {:.4}", metrics.precision);
    println!("      • Recall: {:.4}", metrics.recall);

    Ok(())
}

fn analyze_brown_corpus() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Brown Corpus Analysis (for comparison)");
    println!("{}", "-".repeat(40));

    if !brown_corpus::is_available() {
        println!("❌ Brown Corpus data not available");
        println!("   Run: cd benchmarks/data/brown_corpus && make download");
        return Ok(());
    }

    let processor = create_default_processor();

    // Test a small subset for comparison
    match brown_corpus::load_subset(100) {
        Ok(test_data) => {
            let start = Instant::now();
            let result = processor.process(sakurs_core::Input::from_text(test_data.text))?;
            let processing_time = start.elapsed();

            let predicted_boundaries = extract_boundaries(&result);
            let metrics = calculate_complete_metrics(
                &predicted_boundaries,
                &test_data.boundaries,
                test_data.text.len(),
            );

            println!("\n🎯 Brown Corpus Subset (100 sentences):");
            println!("   📝 Text length: {} characters", test_data.text.len());
            println!("   🎯 Expected boundaries: {}", test_data.boundaries.len());
            println!("   🔍 Predicted boundaries: {}", predicted_boundaries.len());
            println!("   ⚡ Processing time: {:?}", processing_time);
            println!("   📊 Metrics:");
            println!("      • F1 Score: {:.4}", metrics.f1_score);
            println!("      • Precision: {:.4}", metrics.precision);
            println!("      • Recall: {:.4}", metrics.recall);
            println!("      • Pk Score: {:.4}", metrics.pk_score.unwrap_or(0.0));
            println!(
                "      • WindowDiff: {:.4}",
                metrics.window_diff.unwrap_or(0.0)
            );
        }
        Err(e) => {
            println!("❌ Error loading Brown Corpus: {}", e);
        }
    }

    // Test the hardcoded sample
    let sample = brown_corpus::small_sample();
    let start = Instant::now();
    let result = processor.process(sakurs_core::Input::from_text(sample.text))?;
    let processing_time = start.elapsed();

    let predicted_boundaries = extract_boundaries(&result);
    let metrics =
        calculate_complete_metrics(&predicted_boundaries, &sample.boundaries, sample.text.len());

    println!("\n🎯 Brown Corpus Hardcoded Sample:");
    println!("   📝 Text length: {} characters", sample.text.len());
    println!("   🎯 Expected boundaries: {:?}", sample.boundaries);
    println!("   🔍 Predicted boundaries: {:?}", predicted_boundaries);
    println!("   ⚡ Processing time: {:?}", processing_time);
    println!("   📊 Metrics:");
    println!("      • F1 Score: {:.4}", metrics.f1_score);
    println!("      • Precision: {:.4}", metrics.precision);
    println!("      • Recall: {:.4}", metrics.recall);

    Ok(())
}
