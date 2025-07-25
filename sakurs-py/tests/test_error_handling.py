"""Test error handling and validation."""

import pytest

import sakurs


class TestParameterValidation:
    """Test parameter validation and error handling."""

    def test_iter_split_float_chunk_size(self):
        """Test that float chunk_size raises TypeError."""
        with pytest.raises(TypeError) as exc_info:
            list(sakurs.iter_split("Hello.", chunk_kb=1024.5))  # type: ignore

        assert "chunk_size" in str(exc_info.value)
        assert "float" in str(exc_info.value)
        assert "cannot be interpreted as an integer" in str(exc_info.value)

    def test_split_large_file_float_max_memory_mb(self):
        """Test that float max_memory_mb raises TypeError."""
        with pytest.raises(TypeError) as exc_info:
            list(sakurs.split_large_file("/tmp/test.txt", max_memory_mb=10.5))  # type: ignore

        assert "max_memory_mb" in str(exc_info.value)
        assert "float" in str(exc_info.value)
        assert "cannot be interpreted as an integer" in str(exc_info.value)

    def test_split_float_threads(self):
        """Test that float threads raises TypeError."""
        with pytest.raises(TypeError) as exc_info:
            sakurs.split("Hello.", threads=2.5)  # type: ignore

        assert "threads" in str(exc_info.value)
        assert "float" in str(exc_info.value)
        assert "cannot be interpreted as an integer" in str(exc_info.value)

    def test_split_float_chunk_size(self):
        """Test that float chunk_size raises TypeError."""
        with pytest.raises(TypeError) as exc_info:
            sakurs.split("Hello.", chunk_kb=1024.5)  # type: ignore

        assert "chunk_size" in str(exc_info.value)
        assert "float" in str(exc_info.value)
        assert "cannot be interpreted as an integer" in str(exc_info.value)

    def test_load_float_threads(self):
        """Test that float threads in load raises TypeError."""
        with pytest.raises(TypeError) as exc_info:
            sakurs.load("en", threads=4.0)  # type: ignore

        assert "threads" in str(exc_info.value)
        assert "float" in str(exc_info.value)
        assert "cannot be interpreted as an integer" in str(exc_info.value)

    def test_load_float_chunk_size(self):
        """Test that float chunk_size in load raises TypeError."""
        with pytest.raises(TypeError) as exc_info:
            sakurs.load("en", chunk_kb=2048.0)  # type: ignore

        assert "chunk_size" in str(exc_info.value)
        assert "float" in str(exc_info.value)
        assert "cannot be interpreted as an integer" in str(exc_info.value)

    def test_processor_config_float_parameters(self):
        """Test that ProcessorConfig with float parameters raises TypeError."""
        # ProcessorConfig uses different parameter names
        config = sakurs.ProcessorConfig()

        # Test setting float values to integer properties
        with pytest.raises(TypeError) as exc_info:
            config.num_threads = 2.5  # type: ignore

        # The error might be different for property setters
        assert (
            "int" in str(exc_info.value).lower()
            or "float" in str(exc_info.value).lower()
        )

    def test_valid_integer_parameters(self):
        """Test that integer parameters work correctly."""
        # These should work without errors
        sentences = list(sakurs.iter_split("Hello. World.", chunk_kb=1024))
        assert len(sentences) == 2

        sentences = sakurs.split("Hello. World.", threads=2, chunk_kb=1024)
        assert len(sentences) == 2

        processor = sakurs.load("en", threads=1, chunk_kb=512)
        assert processor is not None

    def test_zero_and_negative_integers(self):
        """Test edge cases with zero and negative integers."""
        # Zero threads raises ConfigurationError
        with pytest.raises(sakurs.ConfigurationError) as exc_info:
            sakurs.split("Hello. World.", threads=0)
        assert "threads must be greater than 0" in str(exc_info.value)

        # None for threads should work (auto-detect)
        sentences = sakurs.split("Hello. World.", threads=None)
        assert len(sentences) == 2

        # Negative values raise OverflowError (can't convert to unsigned)
        with pytest.raises(OverflowError) as exc_info2:
            sakurs.split("Hello. World.", threads=-1)
        assert "can't convert negative int to unsigned" in str(exc_info2.value)


class TestEncodingErrors:
    """Test encoding-related error handling."""

    def test_invalid_language_code(self):
        """Test invalid language code handling."""
        with pytest.raises(sakurs.SakursError) as exc_info:
            sakurs.split("Hello.", language="invalid_lang")

        assert "UnsupportedLanguage" in str(exc_info.value) or "invalid_lang" in str(
            exc_info.value
        )

    def test_invalid_execution_mode(self):
        """Test invalid execution mode handling."""
        with pytest.raises(sakurs.SakursError) as exc_info:
            sakurs.split("Hello.", execution_mode="invalid_mode")  # type: ignore

        assert "invalid_mode" in str(exc_info.value) or "execution_mode" in str(
            exc_info.value
        )


class TestInputValidation:
    """Test input validation and error handling."""

    def test_none_input(self):
        """Test None as input."""
        with pytest.raises(TypeError):
            sakurs.split(None)  # type: ignore

    def test_invalid_input_type(self):
        """Test invalid input types."""
        with pytest.raises(TypeError):
            sakurs.split(12345)  # type: ignore  # Integer instead of string

        with pytest.raises(TypeError):
            sakurs.split([1, 2, 3])  # type: ignore  # List instead of string

    def test_empty_string_input(self):
        """Test empty string input (should work)."""
        sentences = sakurs.split("")
        assert sentences == []

        sentences = list(sakurs.iter_split(""))
        assert sentences == []
