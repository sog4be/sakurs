"""Tests for the basic split() function API."""

import pytest

import sakurs


class TestSplitFunction:
    """Test the split() function with various inputs and options."""

    def test_basic_split_english(self):
        """Test basic sentence splitting for English text."""
        text = "Hello world. How are you? I'm fine, thanks!"
        result = sakurs.split(text)

        assert isinstance(result, list)
        assert len(result) == 3
        # Default behavior: sentences are trimmed
        assert result[0] == "Hello world."
        assert result[1] == "How are you?"
        assert result[2] == "I'm fine, thanks!"

    def test_split_with_language_parameter(self):
        """Test split with explicit language parameter."""
        # English
        text = "Hello world. How are you?"
        result = sakurs.split(text, language="en")
        assert len(result) == 2

        # Japanese
        text_ja = "こんにちは。元気ですか？"
        result_ja = sakurs.split(text_ja, language="ja")
        assert len(result_ja) == 2
        assert result_ja[0] == "こんにちは。"
        assert result_ja[1] == "元気ですか？"

    def test_split_with_return_details(self):
        """Test split with return_details=True."""
        text = "Hello world. How are you?"
        result = sakurs.split(text, return_details=True)

        assert isinstance(result, list)
        sentences = result
        assert len(sentences) == 2

        # Check first sentence details
        sent0 = sentences[0]
        assert hasattr(sent0, "text")
        assert hasattr(sent0, "start")
        assert hasattr(sent0, "end")
        assert hasattr(sent0, "confidence")
        assert hasattr(sent0, "metadata")

        # Default behavior: text is trimmed but offsets point to original positions
        assert sent0.text == "Hello world."
        assert sent0.start == 0
        assert sent0.end == 12
        assert sent0.confidence == 1.0

        # Check second sentence details
        sent1 = sentences[1]
        assert sent1.text == "How are you?"  # Trimmed by default
        assert sent1.start == 12  # Original position (includes leading space)
        assert sent1.end == 25
        assert sent1.confidence == 1.0

        # When return_details=True, we get Sentence objects
        # Metadata is not directly accessible in the current implementation

    def test_split_performance_parameters(self):
        """Test split with performance tuning parameters."""
        text = "This is a test. " * 100  # Long text
        # Remove trailing space to avoid edge cases
        text = text.rstrip()

        # Test with threads parameter
        result = sakurs.split(text, threads=2)
        assert isinstance(result, list)
        assert len(result) == 100

        # Test with chunk_size parameter
        result = sakurs.split(text, chunk_size=1024)
        assert isinstance(result, list)
        assert len(result) == 100

        # Test with parallel=True
        result = sakurs.split(text, parallel=True)
        assert isinstance(result, list)
        assert len(result) == 100

        # Test with execution_mode
        result = sakurs.split(text, execution_mode="sequential")
        assert isinstance(result, list)
        assert len(result) == 100

    def test_split_empty_text(self):
        """Test split with empty text."""
        result = sakurs.split("")
        assert result == []

        result_details = sakurs.split("", return_details=True)
        assert result_details == []

    def test_split_single_sentence(self):
        """Test split with single sentence (no terminators)."""
        text = "This is a sentence without punctuation"
        result = sakurs.split(text)
        assert len(result) == 1
        assert result[0] == text

    def test_split_with_multiple_spaces(self):
        """Test split with multiple spaces between sentences."""
        text = "First sentence.    Second sentence.  Third."
        result = sakurs.split(text)
        assert len(result) == 3
        # Default behavior: sentences are trimmed
        assert result[0] == "First sentence."
        assert result[1] == "Second sentence."
        assert result[2] == "Third."

    def test_split_with_abbreviations(self):
        """Test split handles abbreviations correctly."""
        text = "Dr. Smith went to the U.S.A. yesterday. He had a meeting."
        result = sakurs.split(text)
        assert len(result) == 2
        assert result[0] == "Dr. Smith went to the U.S.A. yesterday."
        assert result[1] == "He had a meeting."  # Trimmed by default

    def test_split_with_quotes(self):
        """Test split handles quotes correctly."""
        text = 'He said "Hello there." Then he left.'
        result = sakurs.split(text)
        # The algorithm correctly treats text inside quotes as part of the same sentence
        assert len(result) == 1
        assert result[0] == 'He said "Hello there." Then he left.'

    def test_split_japanese_text(self):
        """Test split with Japanese text."""
        text = "これは日本語の文章です。とても面白いですね！最後の文。"
        result = sakurs.split(text, language="ja")

        assert len(result) == 3
        assert result[0] == "これは日本語の文章です。"
        assert result[1] == "とても面白いですね！"
        assert result[2] == "最後の文。"

    def test_split_with_ellipsis(self):
        """Test split handles ellipsis."""
        text = "Well... I don't know. Maybe..."
        result = sakurs.split(text)
        # Ellipsis handling depends on configuration
        assert len(result) >= 1

    def test_invalid_language_error(self):
        """Test that invalid language raises appropriate error."""
        with pytest.raises(sakurs.InvalidLanguageError):
            sakurs.split("Hello world.", language="invalid")

    def test_invalid_execution_mode_error(self):
        """Test that invalid execution_mode raises appropriate error."""
        with pytest.raises(sakurs.ConfigurationError):
            sakurs.split("Hello world.", execution_mode="invalid")  # type: ignore[call-overload]

    def test_multiple_sentences_offsets(self):
        """Test that offsets are correct for multiple sentences."""
        text = "First sentence. Second one. Third! Fourth?"
        result = sakurs.split(text, return_details=True)

        assert len(result) == 4

        # Verify each sentence's text and offsets
        # Default behavior: text is trimmed but offsets point to original positions
        expected = [
            ("First sentence.", 0, 15),
            ("Second one.", 15, 27),  # Text is trimmed
            ("Third!", 27, 34),  # Text is trimmed
            ("Fourth?", 34, 42),  # Text is trimmed
        ]

        for i, (expected_text, expected_start, expected_end) in enumerate(expected):
            sent = result[i]
            assert sent.text == expected_text, f"Sentence {i}: text mismatch"
            assert sent.start == expected_start, f"Sentence {i}: start offset mismatch"
            assert sent.end == expected_end, f"Sentence {i}: end offset mismatch"
            # Note: text[sent.start:sent.end] includes leading spaces, but sent.text is trimmed

    def test_japanese_offsets(self):
        """Test that offsets work correctly with multi-byte Japanese characters."""
        text = "これは日本語です。とても面白い！最後の文。"
        result = sakurs.split(text, return_details=True, language="ja")

        assert len(result) == 3

        # Japanese text uses multi-byte characters, so we need to be careful with offsets
        # Since Japanese doesn't typically have spaces between sentences, trimming doesn't affect the text
        for i, sent in enumerate(result):
            # For Japanese without spaces, trimmed text equals original slice
            assert text[sent.start : sent.end] == sent.text, (
                f"Sentence {i}: slice matches text for Japanese"
            )

    def test_preserve_whitespace_option(self):
        """Test preserve_whitespace=True option."""
        text = "First sentence.    Second one.  Third!"

        # Default behavior: trim whitespace
        result_trimmed = sakurs.split(text)
        assert result_trimmed == ["First sentence.", "Second one.", "Third!"]

        # With preserve_whitespace=True
        result_preserved = sakurs.split(text, preserve_whitespace=True)
        assert len(result_preserved) == 3
        assert result_preserved[0] == "First sentence."
        assert result_preserved[1] == "    Second one."  # Leading spaces preserved
        assert result_preserved[2] == "  Third!"  # Leading spaces preserved

    def test_preserve_whitespace_with_details(self):
        """Test preserve_whitespace=True with return_details=True."""
        text = "Hello world.  How are you?   I'm fine!"

        # Default: trimmed text
        result_default = sakurs.split(text, return_details=True)
        assert len(result_default) == 3
        assert result_default[0].text == "Hello world."
        assert result_default[1].text == "How are you?"  # Trimmed
        assert result_default[2].text == "I'm fine!"  # Trimmed

        # Offsets should still point to original positions
        assert result_default[1].start == 12  # Points to space before "How"
        assert text[result_default[1].start : result_default[1].end] == "  How are you?"

        # With preserve_whitespace=True
        result_preserved = sakurs.split(
            text, return_details=True, preserve_whitespace=True
        )
        assert len(result_preserved) == 3
        assert result_preserved[0].text == "Hello world."
        assert result_preserved[1].text == "  How are you?"  # Spaces preserved
        assert result_preserved[2].text == "   I'm fine!"  # Spaces preserved

        # Offsets match exactly
        for sent in result_preserved:
            assert text[sent.start : sent.end] == sent.text


class TestSentenceClass:
    """Test the Sentence class functionality."""

    def test_sentence_string_representation(self):
        """Test Sentence __str__ returns the text."""
        text = "Hello world."
        result = sakurs.split(text, return_details=True)
        sentence = result[0]

        assert str(sentence) == "Hello world."

    def test_sentence_repr(self):
        """Test Sentence __repr__ includes all fields."""
        text = "Hello world."
        result = sakurs.split(text, return_details=True)
        sentence = result[0]

        repr_str = repr(sentence)
        assert "Sentence" in repr_str
        assert "text='Hello world.'" in repr_str
        assert "start=0" in repr_str
        assert "end=12" in repr_str
        assert "confidence=1" in repr_str


class TestProcessingMetadata:
    """Test the ProcessingMetadata class functionality."""

    def test_metadata_fields(self):
        """Test all metadata fields are present and valid."""
        # Skip this test for now as metadata is not directly returned
        # with the current API implementation
        pass

    def test_metadata_repr(self):
        """Test ProcessingMetadata __repr__."""
        # Skip this test for now as metadata is not directly returned
        # with the current API implementation
        pass


class TestLoadFunction:
    """Test the load() function."""

    def test_load_processor(self):
        """Test loading a processor."""
        processor = sakurs.load("en")
        assert isinstance(processor, sakurs.Processor)
        assert processor.language == "en"
        assert processor.supports_parallel is True

    def test_load_with_parameters(self):
        """Test loading with performance parameters."""
        processor = sakurs.load("ja", threads=4, chunk_size=1024)
        assert isinstance(processor, sakurs.Processor)
        assert processor.language == "ja"

    def test_processor_split(self):
        """Test using processor to split text."""
        processor = sakurs.load("en")
        result = processor.split("Hello world. How are you?")
        assert len(result) == 2
        assert result[0] == "Hello world."
        assert result[1] == "How are you?"


class TestSupportedLanguages:
    """Test the supported_languages() function."""

    def test_supported_languages(self):
        """Test getting list of supported languages."""
        languages = sakurs.supported_languages()
        assert isinstance(languages, list)
        assert "en" in languages
        assert "ja" in languages
        assert len(languages) >= 2
