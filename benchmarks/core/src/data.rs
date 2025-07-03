//! Test data generation and management for benchmarks

use std::fmt;

/// A test data sample with text and ground truth boundaries
#[derive(Debug, Clone)]
pub struct TestData {
    /// Name/identifier for this test case
    pub name: String,
    /// The text content
    pub text: String,
    /// Ground truth sentence boundary positions (character indices)
    pub boundaries: Vec<usize>,
    /// Optional metadata
    pub metadata: Option<String>,
}

impl TestData {
    /// Create a new test data instance
    pub fn new(name: impl Into<String>, text: impl Into<String>, boundaries: Vec<usize>) -> Self {
        Self {
            name: name.into(),
            text: text.into(),
            boundaries,
            metadata: None,
        }
    }

    /// Add metadata to the test data
    pub fn with_metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    /// Get the number of sentences (boundaries + 1)
    pub fn sentence_count(&self) -> usize {
        self.boundaries.len() + 1
    }

    /// Get text length in characters
    pub fn text_length(&self) -> usize {
        self.text.len()
    }

    /// Validate that boundaries are within text bounds and sorted
    pub fn validate(&self) -> Result<(), String> {
        if self.boundaries.is_empty() {
            return Ok(());
        }

        // Check if sorted
        for i in 1..self.boundaries.len() {
            if self.boundaries[i] <= self.boundaries[i - 1] {
                return Err(format!(
                    "Boundaries not sorted: {} comes after {}",
                    self.boundaries[i],
                    self.boundaries[i - 1]
                ));
            }
        }

        // Check bounds
        let max_boundary = *self.boundaries.last().unwrap();
        if max_boundary >= self.text.len() {
            return Err(format!(
                "Boundary {} exceeds text length {}",
                max_boundary,
                self.text.len()
            ));
        }

        Ok(())
    }
}

impl fmt::Display for TestData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TestData '{}': {} chars, {} sentences",
            self.name,
            self.text.len(),
            self.sentence_count()
        )
    }
}

/// Generate simple test data samples
pub mod generators {
    use super::TestData;

    /// Generate a simple text with period-separated sentences
    pub fn simple_sentences(sentence_count: usize) -> TestData {
        let sentence = "This is a test sentence";
        let mut text = String::new();
        let mut boundaries = Vec::new();

        for i in 0..sentence_count {
            if i > 0 {
                text.push(' ');
            }
            text.push_str(sentence);
            text.push('.');

            if i < sentence_count - 1 {
                boundaries.push(text.len());
            }
        }

        TestData::new(
            format!("simple_{}_sentences", sentence_count),
            text,
            boundaries,
        )
    }

    /// Generate text with abbreviations
    pub fn with_abbreviations() -> TestData {
        let text = "Dr. Smith works at the U.S. Geological Survey. He studies earthquakes. \
                   Prof. Johnson collaborates with him. They published in Nature.";

        // Boundaries after actual sentence endings (period + space), not abbreviations
        let boundaries = vec![46, 70, 107];

        TestData::new("abbreviations", text, boundaries)
    }

    /// Generate text with quotations
    pub fn with_quotations() -> TestData {
        let text = r#"She said, "Hello there!" He replied, "How are you?" They chatted. "Nice weather," she noted."#;

        // Boundaries after quotation sentences
        // Note: sakurs detects at '!' and '?' within quotes, and after period
        let boundaries = vec![23, 50, 65];

        TestData::new("quotations", text, boundaries)
    }

    /// Generate text with numbers and decimals
    pub fn with_numbers() -> TestData {
        let text = "The temperature was 98.6 degrees. The price increased by 3.5%. \
                   We need 2.5 kg of flour. Mix it well.";

        // Boundaries after period + space
        let boundaries = vec![33, 62, 87];

        TestData::new("numbers", text, boundaries)
    }

    /// Generate complex mixed text
    pub fn complex_mixed() -> TestData {
        let text = r#"Dr. Watson said, "Elementary!" The U.S. market grew 2.5% in Q1. "Impressive," noted the C.E.O. of Tech Corp."#;

        // Boundaries after exclamation in quote and after period
        // Note: sakurs incorrectly splits C.E.O.
        let boundaries = vec![29, 63];

        TestData::new("complex_mixed", text, boundaries)
    }

    /// Generate a large text sample for performance testing
    pub fn large_text(approx_size: usize) -> TestData {
        let base_sentences = [
            "This is a normal sentence.",
            "Dr. Smith studied the results carefully.",
            "The temperature reached 32.5 degrees today.",
            r#"She exclaimed, "What a beautiful day!""#,
            "The U.S. economy showed signs of growth.",
            "Prof. Johnson published three papers.",
            "We need 2.5 kg of ingredients.",
            r#"He asked, "Can you help me?""#,
        ];

        let mut text = String::new();
        let mut boundaries = Vec::new();
        let mut current_size = 0;

        while current_size < approx_size {
            for (i, sentence) in base_sentences.iter().enumerate() {
                if current_size >= approx_size {
                    break;
                }

                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(sentence);
                current_size = text.len();

                // Add boundary if not the last sentence
                if current_size < approx_size || i < base_sentences.len() - 1 {
                    boundaries.push(text.len());
                }
            }
        }

        // Remove the last boundary if it exists
        if !boundaries.is_empty() {
            boundaries.pop();
        }

        TestData::new(
            format!("large_text_{}k", approx_size / 1000),
            text,
            boundaries,
        )
    }
}

/// Load predefined Brown Corpus samples
/// Note: This is a placeholder. Real implementation would load actual Brown Corpus data
pub mod brown_corpus {
    use super::TestData;

    /// Get a small Brown Corpus sample for testing
    pub fn small_sample() -> TestData {
        // This is a simplified example. Real implementation would load from files
        let text = "The Fulton County Grand Jury said Friday an investigation of \
                   Atlanta's recent primary election produced no evidence that \
                   any irregularities took place. The jury further said in \
                   term-end presentments that the City Executive Committee, \
                   which had over-all charge of the election, deserves the \
                   praise and thanks of the City of Atlanta for the manner \
                   in which the election was conducted.";

        // Boundaries are detected after period + space
        let boundaries = vec![151];

        TestData::new("brown_corpus_sample", text, boundaries)
            .with_metadata("Brown Corpus news category sample")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_generator() {
        let data = generators::simple_sentences(3);
        assert_eq!(data.sentence_count(), 3);
        assert_eq!(data.boundaries.len(), 2);
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_validation() {
        let valid = TestData::new("valid", "Hello. World.", vec![6]);
        assert!(valid.validate().is_ok());

        let invalid_order = TestData::new("invalid", "Hello. World.", vec![10, 5]);
        assert!(invalid_order.validate().is_err());

        let invalid_bounds = TestData::new("invalid", "Hello.", vec![10]);
        assert!(invalid_bounds.validate().is_err());
    }

    #[test]
    fn test_generators() {
        let tests = vec![
            generators::simple_sentences(5),
            generators::with_abbreviations(),
            generators::with_quotations(),
            generators::with_numbers(),
            generators::complex_mixed(),
            generators::large_text(1000),
        ];

        for test in tests {
            assert!(test.validate().is_ok(), "Invalid test data: {}", test.name);
        }
    }
}
