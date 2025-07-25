"""Tests for file path input handling."""

import tempfile
from pathlib import Path

import sakurs


class TestFileInput:
    """Test file path input functionality."""

    def test_string_path_input(self):
        """Test with string file path."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("This is a test. It has two sentences.")
            temp_path = f.name

        try:
            # Test with string path
            sentences = sakurs.split(temp_path)
            assert len(sentences) == 2
            assert sentences[0] == "This is a test."
            assert sentences[1] == "It has two sentences."
        finally:
            Path(temp_path).unlink()

    def test_pathlib_path_input(self):
        """Test with pathlib.Path object."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("Hello world! How are you?")
            temp_path = Path(f.name)

        try:
            # Test with Path object
            sentences = sakurs.split(temp_path)
            assert len(sentences) == 2
            assert sentences[0] == "Hello world!"
            assert sentences[1] == "How are you?"
        finally:
            temp_path.unlink()

    def test_file_with_unicode(self):
        """Test file with Unicode content."""
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", suffix=".txt", delete=False
        ) as f:
            f.write("これは日本語です。テストしています。")
            temp_path = f.name

        try:
            sentences = sakurs.split(temp_path, language="ja")
            assert len(sentences) == 2
            assert sentences[0] == "これは日本語です。"
            assert sentences[1] == "テストしています。"
        finally:
            Path(temp_path).unlink()

    def test_nonexistent_file(self):
        """Test with non-existent file path."""
        # When a string path doesn't exist, it's treated as text content
        # This is expected behavior for backward compatibility
        # The text "/path/that/does/not/exist.txt" contains a period, so it's split
        sentences = sakurs.split("/path/that/does/not/exist.txt")
        assert len(sentences) == 2
        assert sentences[0] == "/path/that/does/not/exist."
        assert sentences[1] == "txt"

    def test_file_with_different_encodings(self):
        """Test files with different encodings."""
        # UTF-8 (default)
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", suffix=".txt", delete=False
        ) as f:
            f.write("UTF-8 text. With émojis! 🎉")
            utf8_path = f.name

        try:
            sentences = sakurs.split(utf8_path)
            assert len(sentences) == 3
            assert "émojis!" in sentences[1]
        finally:
            Path(utf8_path).unlink()

        # Latin-1 - Currently skipped due to core Input limitation
        # TODO: sakurs_core Input currently only accepts UTF-8 text
        # This test is commented out until core support is added
        # with tempfile.NamedTemporaryFile(mode="wb", suffix=".txt", delete=False) as f:
        #     # Write Latin-1 encoded text
        #     text = "Latin-1 text with café. Très bien!"
        #     f.write(text.encode("latin-1"))
        #     latin1_path = f.name
        #
        # try:
        #     sentences = sakurs.split(latin1_path, encoding="latin-1")
        #     assert len(sentences) == 2
        #     assert "café" in sentences[0]
        #     assert "Très bien!" in sentences[1]
        # finally:
        #     Path(latin1_path).unlink()

    def test_empty_file(self):
        """Test with empty file."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            # Write nothing
            temp_path = f.name

        try:
            sentences = sakurs.split(temp_path)
            assert sentences == []
        finally:
            Path(temp_path).unlink()

    def test_large_file(self):
        """Test with larger file to trigger parallel processing."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            # Write many sentences
            sentences = [f"This is sentence number {i}." for i in range(1000)]
            f.write(" ".join(sentences))
            temp_path = f.name

        try:
            sentences = sakurs.split(temp_path, parallel=True, chunk_kb=1024)
            assert len(sentences) == 1000
            assert sentences[0] == "This is sentence number 0."
            assert sentences[999] == "This is sentence number 999."
        finally:
            Path(temp_path).unlink()

    def test_file_path_with_return_details(self):
        """Test file input with return_details=True."""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("First sentence. Second sentence.")
            temp_path = f.name

        try:
            results = sakurs.split(temp_path, return_details=True)
            assert len(results) == 2

            # Check first sentence
            assert results[0].text == "First sentence."
            assert results[0].start == 0
            assert results[0].end == 15

            # Check second sentence
            assert results[1].text == "Second sentence."
            assert (
                results[1].start == 15
            )  # Note: This includes the space before "Second"
            assert (
                results[1].end == 32
            )  # End of text (currently includes trailing position)
        finally:
            Path(temp_path).unlink()

    def test_processor_with_file_input(self):
        """Test Processor class with file input."""
        processor = sakurs.SentenceSplitter()

        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
            f.write("Testing processor. With file input!")
            temp_path = f.name

        try:
            sentences = processor.split(temp_path)
            assert len(sentences) == 2
            assert sentences[0] == "Testing processor."
            assert sentences[1] == "With file input!"
        finally:
            Path(temp_path).unlink()
