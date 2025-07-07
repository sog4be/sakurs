//! Example of using the new unified API

use sakurs_core::{Config, Input, SentenceProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Simple usage with default configuration
    println!("=== Example 1: Simple Usage ===");
    let processor = SentenceProcessor::for_language("en")?;
    let text = "Dr. Smith went to the store. He bought milk. The price was $5.99.";
    let output = processor.process(Input::from_text(text))?;

    println!("Input text: {}", text);
    println!("Found {} sentences", output.boundaries.len());
    for (i, boundary) in output.boundaries.iter().enumerate() {
        println!("  Sentence {}: ends at byte {}", i + 1, boundary.offset);
    }

    // Example 2: Japanese text processing
    println!("\n=== Example 2: Japanese Text ===");
    let ja_processor = SentenceProcessor::for_language("ja")?;
    let ja_text = "これはテストです。日本語も正しく処理できます。素晴らしい！";
    let ja_output = ja_processor.process(Input::from_text(ja_text))?;

    println!("Input text: {}", ja_text);
    println!("Found {} sentences", ja_output.boundaries.len());

    // Example 3: Custom configuration
    println!("\n=== Example 3: Custom Configuration ===");
    let config = Config::builder()
        .language("en")
        .threads(4)
        .chunk_size(512) // 512KB chunks
        .abbreviations(false)
        .build()?;

    let custom_processor = SentenceProcessor::with_config(config)?;
    let technical_text =
        "The system uses TCP/IP protocol. Network speed is 1Gbps. Dr. Johnson approved it.";
    let custom_output = custom_processor.process(Input::from_text(technical_text))?;

    println!("Input text: {}", technical_text);
    println!("Processing stats:");
    println!("  - Duration: {:?}", custom_output.metadata.duration);
    println!("  - Strategy: {}", custom_output.metadata.strategy_used);
    println!(
        "  - Sentences: {}",
        custom_output.metadata.stats.sentence_count
    );

    // Example 4: Processing from file
    println!("\n=== Example 4: File Processing ===");
    // Create a temporary file for demonstration
    use std::io::Write;
    let temp_file = std::env::temp_dir().join("sakurs_example.txt");
    std::fs::File::create(&temp_file)?
        .write_all(b"This is from a file. It has multiple sentences. Each one ends properly.")?;

    let file_output = processor.process(Input::from_file(&temp_file))?;
    println!("Processed file: {:?}", temp_file);
    println!("Found {} sentences", file_output.boundaries.len());

    // Clean up
    std::fs::remove_file(temp_file)?;

    // Example 5: Performance comparison
    println!("\n=== Example 5: Performance Presets ===");
    let large_text = "This is a test. ".repeat(1000);

    let fast_config = Config::fast();
    let fast_processor = SentenceProcessor::with_config(fast_config)?;
    let start = std::time::Instant::now();
    let _ = fast_processor.process(Input::from_text(large_text.clone()))?;
    let fast_time = start.elapsed();

    let accurate_config = Config::accurate();
    let accurate_processor = SentenceProcessor::with_config(accurate_config)?;
    let start = std::time::Instant::now();
    let _ = accurate_processor.process(Input::from_text(large_text))?;
    let accurate_time = start.elapsed();

    println!("Processing 1000 sentences:");
    println!("  - Fast mode: {:?}", fast_time);
    println!("  - Accurate mode: {:?}", accurate_time);

    Ok(())
}
