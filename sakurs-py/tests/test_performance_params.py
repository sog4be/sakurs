"""Test performance parameters functionality."""

import pytest

import sakurs


class TestPerformanceParameters:
    """Test that performance parameters work correctly."""

    def test_split_with_sequential_mode(self):
        """Test split with sequential execution mode."""
        text = "First sentence. Second sentence. Third sentence."
        sentences = sakurs.split(text, execution_mode="sequential")
        assert len(sentences) == 3
        assert sentences[0] == "First sentence."

    def test_split_with_parallel_mode(self):
        """Test split with parallel execution mode."""
        text = "First sentence. Second sentence. Third sentence."
        sentences = sakurs.split(text, execution_mode="parallel", threads=2)
        assert len(sentences) == 3
        assert sentences[0] == "First sentence."

    def test_split_with_adaptive_mode(self):
        """Test split with adaptive execution mode (default)."""
        text = "First sentence. Second sentence. Third sentence."
        sentences = sakurs.split(text, execution_mode="adaptive")
        assert len(sentences) == 3
        assert sentences[0] == "First sentence."

    def test_split_with_custom_chunk_size(self):
        """Test split with custom chunk size."""
        text = "First sentence. Second sentence. Third sentence."
        # Small chunk size to potentially trigger parallel processing
        sentences = sakurs.split(text, chunk_kb=1)
        assert len(sentences) == 3

    def test_split_with_parallel_flag(self):
        """Test split with parallel flag to force parallel processing."""
        text = "Short text. Another sentence."
        # Force parallel even for small text
        sentences = sakurs.split(text, parallel=True, threads=2)
        assert len(sentences) == 2

    def test_load_with_performance_params(self):
        """Test load() function with performance parameters."""
        processor = sakurs.load("en", threads=4, chunk_kb=1)
        assert processor.language == "en"
        sentences = processor.split("Hello world. How are you?")
        assert len(sentences) == 2

    def test_processor_with_performance_params(self):
        """Test Processor initialization with performance parameters."""
        processor = sakurs.SentenceSplitter(
            language="en", threads=2, chunk_kb=1, execution_mode="parallel"
        )
        assert processor.language == "en"
        sentences = processor.split("Hello world. How are you?")
        assert len(sentences) == 2

    def test_processor_context_manager(self):
        """Test Processor as context manager."""
        with sakurs.SentenceSplitter(language="en") as processor:
            sentences = processor.split("Hello world. How are you?")
            assert len(sentences) == 2

    def test_processor_with_return_details(self):
        """Test Processor.split with return_details=True."""
        processor = sakurs.SentenceSplitter(language="en")
        sentences = processor.split("Hello world. How are you?", return_details=True)
        assert len(sentences) == 2
        assert hasattr(sentences[0], "text")
        assert hasattr(sentences[0], "start")
        assert hasattr(sentences[0], "end")
        assert sentences[0].text == "Hello world."

    def test_invalid_execution_mode(self):
        """Test that invalid execution mode raises error."""
        with pytest.raises(sakurs.ConfigurationError):
            sakurs.split("Test.", execution_mode="invalid")  # type: ignore

    def test_processor_repr(self):
        """Test Processor string representation."""
        processor = sakurs.SentenceSplitter(language="en", threads=4, chunk_kb=1)
        repr_str = repr(processor)
        assert "SentenceSplitter" in repr_str
        assert "language='en'" in repr_str
        assert "threads" in repr_str
        assert "chunk_kb" in repr_str


class TestProcessorStreamingConfig:
    """Test Processor with streaming configuration."""

    def test_processor_streaming_mode(self):
        """Test Processor with streaming mode enabled."""
        processor = sakurs.SentenceSplitter(
            language="en",
            streaming=True,
            stream_chunk_mb=1,  # 1MB
        )
        sentences = processor.split("Hello world. How are you?")
        assert len(sentences) == 2

    def test_processor_default_streaming_disabled(self):
        """Test that streaming is disabled by default."""
        processor = sakurs.SentenceSplitter(language="en")
        # Should use default chunk size, not streaming chunk size
        sentences = processor.split("Hello world. How are you?")
        assert len(sentences) == 2
