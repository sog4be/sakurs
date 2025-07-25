"""Test SentenceSplitter class functionality."""

import io
import tempfile
from pathlib import Path

import pytest

import sakurs


class TestProcessorBasics:
    """Test basic SentenceSplitter functionality."""

    def test_processor_creation(self):
        """Test creating a processor with default settings."""
        processor = sakurs.SentenceSplitter()
        assert processor.language == "en"
        assert processor.supports_parallel is True

    def test_processor_with_language(self):
        """Test creating a processor with specific language."""
        processor = sakurs.SentenceSplitter(language="ja")
        assert processor.language == "ja"

    def test_processor_split_text(self):
        """Test splitting text with processor."""
        processor = sakurs.SentenceSplitter(language="en")
        sentences = processor.split("Hello world. How are you?")
        assert len(sentences) == 2
        assert sentences[0] == "Hello world."
        assert sentences[1] == "How are you?"

    def test_processor_split_file(self):
        """Test splitting file content with processor."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("First sentence. Second sentence.")
            temp_path = f.name

        try:
            processor = sakurs.SentenceSplitter()
            sentences = processor.split(temp_path)
            assert len(sentences) == 2
            assert sentences[0] == "First sentence."
        finally:
            Path(temp_path).unlink()

    def test_processor_split_path_object(self):
        """Test splitting file using Path object."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("First sentence. Second sentence.")
            temp_path = Path(f.name)

        try:
            processor = sakurs.SentenceSplitter()
            sentences = processor.split(temp_path)
            assert len(sentences) == 2
        finally:
            temp_path.unlink()

    def test_processor_split_file_like(self):
        """Test splitting file-like object with processor."""
        text_io = io.StringIO("Hello world. How are you?")
        processor = sakurs.SentenceSplitter()
        sentences = processor.split(text_io)
        assert len(sentences) == 2

    def test_processor_with_different_encodings(self):
        """Test processor with different text encodings."""
        # UTF-8 text
        processor = sakurs.SentenceSplitter()
        text = "Hello world. こんにちは。"
        sentences = processor.split(text)
        assert len(sentences) >= 2

    def test_load_function(self):
        """Test load() factory function."""
        processor = sakurs.load("en")
        assert isinstance(processor, sakurs.SentenceSplitter)
        assert processor.language == "en"

    def test_load_with_params(self):
        """Test load() with additional parameters."""
        processor = sakurs.load("en", threads=2, chunk_size=1024)
        sentences = processor.split("Hello. World.")
        assert len(sentences) == 2


class TestProcessorAdvanced:
    """Test advanced SentenceSplitter functionality."""

    def test_processor_context_manager_normal(self):
        """Test SentenceSplitter as context manager - normal case."""
        with sakurs.SentenceSplitter(language="en") as proc:
            sentences = proc.split("Hello. World.")
            assert len(sentences) == 2

    def test_processor_context_manager_with_exception(self):
        """Test SentenceSplitter as context manager - exception case."""
        with pytest.raises(ValueError, match="Test exception"):  # noqa: SIM117
            with sakurs.SentenceSplitter():
                # Force an exception
                raise ValueError("Test exception")

    def test_processor_multiple_languages(self):
        """Test creating processors for different languages."""
        en_proc = sakurs.SentenceSplitter(language="en")
        ja_proc = sakurs.SentenceSplitter(language="ja")

        assert en_proc.language == "en"
        assert ja_proc.language == "ja"

    def test_processor_invalid_language(self):
        """Test processor with invalid language."""
        with pytest.raises(sakurs.InvalidLanguageError):
            sakurs.SentenceSplitter(language="invalid")

    def test_processor_execution_modes(self):
        """Test different execution modes."""
        text = "Sentence one. Sentence two. Sentence three."

        # Sequential mode
        proc_seq = sakurs.SentenceSplitter(execution_mode="sequential")
        sentences = proc_seq.split(text)
        assert len(sentences) == 3

        # Parallel mode
        proc_par = sakurs.SentenceSplitter(execution_mode="parallel", threads=2)
        sentences = proc_par.split(text)
        assert len(sentences) == 3

        # Adaptive mode (default)
        proc_adapt = sakurs.SentenceSplitter(execution_mode="adaptive")
        sentences = proc_adapt.split(text)
        assert len(sentences) == 3

    def test_processor_performance_params(self):
        """Test processor with various performance parameters."""
        processor = sakurs.SentenceSplitter(
            language="en", threads=4, chunk_size=1024, execution_mode="parallel"
        )

        text = " ".join([f"This is sentence number {i}." for i in range(10)])
        sentences = processor.split(text)
        assert len(sentences) == 10
