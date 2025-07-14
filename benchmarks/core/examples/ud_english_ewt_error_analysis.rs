//! UD English EWT error pattern analysis tool
//!
//! Analyzes mismatches between expected and predicted sentence boundaries
//! to identify systematic errors in sakurs processing logic.

use sakurs_benchmarks::data::ud_english_ewt;
use sakurs_benchmarks::{calculate_complete_metrics, create_default_processor, extract_boundaries};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 UD English EWT Error Pattern Analysis");
    println!("{}", "=".repeat(50));

    if !ud_english_ewt::is_available() {
        println!("❌ UD English EWT data not available");
        println!("   Please run: cd benchmarks/data/ud_english_ewt && python download.py");
        return Ok(());
    }

    // Load larger subset for comprehensive analysis
    let test_data = match ud_english_ewt::load_subset(1000) {
        Ok(data) => {
            println!("✅ Loaded {} sentences for analysis", data.boundaries.len());
            data
        }
        Err(_) => {
            println!("⚠️  Could not load full subset, using smaller sample");
            match ud_english_ewt::load_subset(100) {
                Ok(data) => data,
                Err(_) => {
                    println!("⚠️  Using minimal sample");
                    ud_english_ewt::small_sample()
                }
            }
        }
    };

    // Process with sakurs
    let processor = create_default_processor();
    let result = processor.process(sakurs_core::Input::from_text(test_data.text))?;

    // Extract boundaries
    let predicted = extract_boundaries(&result);
    let expected = &test_data.boundaries;

    // Calculate metrics
    let metrics = calculate_complete_metrics(&predicted, expected, test_data.text.len());

    println!("\n📊 Overall Performance:");
    println!("   F1 Score: {:.4}", metrics.f1_score);
    println!("   Precision: {:.4}", metrics.precision);
    println!("   Recall: {:.4}", metrics.recall);

    // Analyze error patterns
    analyze_error_patterns(&test_data.text, expected, &predicted);

    Ok(())
}

fn analyze_error_patterns(text: &str, expected: &[usize], predicted: &[usize]) {
    println!("\n🔍 Detailed Error Analysis:");

    // Find false positives and false negatives
    let mut false_positives = Vec::new();
    let mut false_negatives = Vec::new();

    for &p in predicted {
        if !expected.contains(&p) {
            false_positives.push(p);
        }
    }

    for &e in expected {
        if !predicted.contains(&e) {
            false_negatives.push(e);
        }
    }

    println!(
        "\n❌ False Positives: {} (detected but not actual boundaries)",
        false_positives.len()
    );
    if !false_positives.is_empty() {
        analyze_error_contexts(text, &false_positives, "FP");
    }

    println!(
        "\n❌ False Negatives: {} (missed actual boundaries)",
        false_negatives.len()
    );
    if !false_negatives.is_empty() {
        analyze_error_contexts(text, &false_negatives, "FN");
    }
}

fn analyze_error_contexts(text: &str, positions: &[usize], error_type: &str) {
    let mut pattern_counts: HashMap<String, usize> = HashMap::new();
    let chars: Vec<char> = text.chars().collect();

    // Analyze up to 20 errors for patterns
    let sample_size = positions.len().min(20);
    println!("   Analyzing first {} errors:", sample_size);

    for (idx, &pos) in positions.iter().take(sample_size).enumerate() {
        if let Some((before, after, pattern)) = extract_and_categorize(text, &chars, pos) {
            println!("\n   {} #{}: Position {}", error_type, idx + 1, pos);
            println!("   Before: \"{}\"", before);
            println!("   After:  \"{}\"", after);
            println!("   Pattern: {}", pattern);

            *pattern_counts.entry(pattern).or_insert(0) += 1;
        }
    }

    // Summary of patterns
    if !pattern_counts.is_empty() {
        println!("\n   📊 Pattern Summary:");
        let mut patterns: Vec<_> = pattern_counts.into_iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1));

        for (pattern, count) in patterns {
            let percentage = (count as f64 / sample_size as f64) * 100.0;
            println!(
                "      {} - {} occurrences ({:.1}%)",
                pattern, count, percentage
            );
        }
    }
}

fn extract_and_categorize(
    _text: &str,
    chars: &[char],
    pos: usize,
) -> Option<(String, String, String)> {
    if pos >= chars.len() {
        return None;
    }

    // Get 30 chars before and after for context
    let start = pos.saturating_sub(30);
    let end = (pos + 30).min(chars.len());

    let before: String = chars[start..pos].iter().collect();
    let after: String = chars[pos..end].iter().collect();

    // Categorize the pattern
    let pattern = categorize_error_pattern(&before, &after, chars.get(pos).copied());

    Some((before, after, pattern))
}

fn categorize_error_pattern(before: &str, after: &str, boundary_char: Option<char>) -> String {
    let after_trimmed = after.trim_start();
    let before_trimmed = before.trim_end();

    // Web-specific patterns
    if after_trimmed.starts_with("http://") || after_trimmed.starts_with("https://") {
        return "URL after punctuation".to_string();
    }

    if after_trimmed.starts_with('@')
        || (before.contains('@') && after_trimmed.starts_with(char::is_alphabetic))
    {
        return "Email/mention".to_string();
    }

    // Number patterns
    if before_trimmed.ends_with(char::is_numeric) && after_trimmed.starts_with(char::is_numeric) {
        if before.contains('.') {
            return "Decimal number".to_string();
        }
        return "Number continuation".to_string();
    }

    // Ellipsis patterns
    if before.contains("...") || after.starts_with("..") {
        return "Ellipsis".to_string();
    }

    // Abbreviation patterns
    if let Some(last_word) = before_trimmed.split_whitespace().last() {
        if last_word.len() <= 4
            && last_word.ends_with('.')
            && after_trimmed.starts_with(char::is_lowercase)
        {
            return "Potential abbreviation".to_string();
        }
    }

    // Quote patterns
    if after_trimmed.starts_with('"')
        || after_trimmed.starts_with('\'')
        || after_trimmed.starts_with('"')
    {
        return "Quote after punctuation".to_string();
    }

    // Parenthetical patterns
    if after_trimmed.starts_with('(') || after_trimmed.starts_with('[') {
        return "Parenthetical after punctuation".to_string();
    }

    // Missing punctuation
    if let Some(ch) = boundary_char {
        if ch == ' ' && !before.trim_end().ends_with(['.', '!', '?']) {
            return "Missing sentence-ending punctuation".to_string();
        }
    }

    // Special web text patterns
    if after_trimmed.starts_with("lol")
        || after_trimmed.starts_with("LOL")
        || after_trimmed.starts_with("btw")
        || after_trimmed.starts_with("BTW")
    {
        return "Internet slang".to_string();
    }

    // Emoticons
    if after_trimmed.starts_with(':')
        || after_trimmed.starts_with(';')
        || after_trimmed.starts_with("=)")
        || after_trimmed.starts_with(":(")
    {
        return "Emoticon".to_string();
    }

    "Other/Unknown pattern".to_string()
}
