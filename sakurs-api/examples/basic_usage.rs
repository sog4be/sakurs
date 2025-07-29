//! Basic usage example for the new 3-crate architecture

use sakurs_api::{process_text, Config, ConfigBuilder, SentenceProcessor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Method 1: Simplest usage with convenience function
    println!("=== Method 1: Convenience Function ===");
    let output = process_text("Hello world. This is a test. How are you?")?;

    println!("Found {} sentences:", output.boundaries.len());
    for (i, boundary) in output.boundaries.iter().enumerate() {
        println!(
            "  Sentence {}: ends at byte {}",
            i + 1,
            boundary.byte_offset
        );
    }
    println!("Processing took {}ms\n", output.metadata.processing_time_ms);

    // Method 2: Using configuration presets
    println!("=== Method 2: Configuration Presets ===");
    let _config = Config::fast();
    let processor = ConfigBuilder::default().fast().build_processor()?;

    let text = "This is a longer text. It has multiple sentences. Some are short. \
                Some are much longer and contain more complex structures!";
    let output = processor.process(text)?;

    println!("Fast mode found {} sentences", output.len());

    // Method 3: Custom configuration
    println!("\n=== Method 3: Custom Configuration ===");
    let processor = ConfigBuilder::default()
        .language("en")?
        .threads(Some(2))
        .chunk_size(1024)
        .build_processor()?;

    let output = processor
        .process("Dr. Smith went to the store. He bought some milk. Then he went home.")?;

    println!("Custom config found {} sentences", output.len());

    // Method 4: Japanese text processing
    println!("\n=== Method 4: Japanese Text ===");
    let processor = SentenceProcessor::with_language("ja")?;
    let japanese_text = "これは日本語のテキストです。複数の文が含まれています。どうですか？";

    let boundaries = processor.process(japanese_text)?;
    println!("Japanese processor found {} sentences", boundaries.len());

    Ok(())
}
