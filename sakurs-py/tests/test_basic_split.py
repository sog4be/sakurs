"""Tests for the basic split() function with return_details parameter."""

import pytest

import sakurs


class TestBasicSplit:
    """Test the split() function with various options."""

    def test_split_default_returns_strings(self):
        """Test that split() returns List[str] by default."""
        text = "Hello world. How are you? I'm fine!"
        result = sakurs.split(text)

        assert isinstance(result, list)
        assert all(isinstance(s, str) for s in result)
        assert len(result) == 3
        assert result[0] == "Hello world."
        assert result[1] == "How are you?"
        assert result[2] == "I'm fine!"

    def test_split_with_return_details_false(self):
        """Test that split() with return_details=False returns List[str]."""
        text = "First sentence. Second sentence."
        result = sakurs.split(text, return_details=False)

        assert isinstance(result, list)
        assert all(isinstance(s, str) for s in result)
        assert len(result) == 2

    def test_split_with_return_details_true(self):
        """Test that split() with return_details=True returns List[Sentence]."""
        text = "First sentence. Second sentence."
        result = sakurs.split(text, return_details=True)

        assert isinstance(result, list)
        assert all(isinstance(s, sakurs.Sentence) for s in result)
        assert len(result) == 2

        # Check first sentence
        assert result[0].text == "First sentence."
        assert result[0].start == 0
        assert result[0].end == 15
        assert result[0].confidence == 1.0
        assert isinstance(result[0].metadata, dict)

        # Check second sentence
        assert result[1].text == "Second sentence."
        assert result[1].start == 16
        assert result[1].end == 32

    def test_sentence_object_string_methods(self):
        """Test Sentence object string representations."""
        text = "Test sentence."
        result = sakurs.split(text, return_details=True)
        sentence = result[0]

        # Test __str__
        assert str(sentence) == "Test sentence."

        # Test __len__
        assert len(sentence) == len("Test sentence.")

        # Test __repr__
        repr_str = repr(sentence)
        assert "Sentence(" in repr_str
        assert "text='Test sentence.'" in repr_str
        assert "start=0" in repr_str

    def test_split_with_language_parameter(self):
        """Test split() with language parameter."""
        text = "Hello. World."

        # Test with English
        result_en = sakurs.split(text, language="en")
        assert len(result_en) == 2

        # Test with Japanese
        japanese_text = "こんにちは。元気ですか？"
        result_ja = sakurs.split(japanese_text, language="ja")
        assert len(result_ja) == 2

    def test_split_empty_text(self):
        """Test split() with empty text."""
        result = sakurs.split("", return_details=False)
        assert result == []

        result_detailed = sakurs.split("", return_details=True)
        assert result_detailed == []

    def test_split_no_sentence_endings(self):
        """Test split() with text that has no sentence endings."""
        text = "This is text without any sentence endings"

        result = sakurs.split(text, return_details=False)
        assert len(result) == 1
        assert result[0] == text

        result_detailed = sakurs.split(text, return_details=True)
        assert len(result_detailed) == 1
        assert result_detailed[0].text == text
        assert result_detailed[0].start == 0
        assert result_detailed[0].end == len(text)

    def test_split_with_abbreviations(self):
        """Test split() handles abbreviations correctly."""
        text = "Dr. Smith went to the U.S.A. yesterday. He had a meeting."
        result = sakurs.split(text)

        assert len(result) == 2
        assert result[0] == "Dr. Smith went to the U.S.A. yesterday."
        assert result[1] == "He had a meeting."

    def test_split_with_quotes(self):
        """Test split() handles quotes correctly."""
        text = 'He said "Hello there." Then he left.'
        result = sakurs.split(text, return_details=True)

        assert len(result) == 2
        assert result[0].text == 'He said "Hello there."'
        assert result[1].text == "Then he left."


class TestExceptions:
    """Test exception handling."""

    def test_invalid_language_error(self):
        """Test that invalid language raises InvalidLanguageError."""
        with pytest.raises(sakurs.InvalidLanguageError):
            sakurs.split("Hello.", language="unsupported_lang")

    def test_exception_hierarchy(self):
        """Test that exception hierarchy is correct."""
        # Base class
        assert issubclass(sakurs.SakursError, Exception)

        # Specific exceptions inherit from base
        assert issubclass(sakurs.InvalidLanguageError, sakurs.SakursError)
        assert issubclass(sakurs.ProcessingError, sakurs.SakursError)
        assert issubclass(sakurs.ConfigurationError, sakurs.SakursError)
        assert issubclass(sakurs.FileNotFoundError, sakurs.SakursError)


class TestProcessor:
    """Test Processor class with new functionality."""

    def test_processor_creation(self):
        """Test creating a processor."""
        processor = sakurs.load("en")
        assert processor.language == "en"
        assert processor.supports_parallel

    def test_processor_split(self):
        """Test processor.split() method."""
        processor = sakurs.load("en")
        text = "First. Second. Third."

        result = processor.split(text)
        assert isinstance(result, list)
        assert all(isinstance(s, str) for s in result)
        assert len(result) == 3

    def test_processor_with_config(self):
        """Test processor with custom configuration."""
        config = sakurs.ProcessorConfig(
            chunk_size=8192, overlap_size=256, num_threads=2
        )
        processor = sakurs.load("en", config)

        text = "Test sentence. Another one."
        result = processor.split(text)
        assert len(result) == 2
