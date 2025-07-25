"""Tests for file-like object input handling."""

import io
import tempfile

import pytest

import sakurs


class TestFileLikeInput:
    """Test file-like object input functionality."""

    def test_stringio_input(self):
        """Test with StringIO text stream."""
        text_stream = io.StringIO("This is from StringIO. It works!")

        sentences = sakurs.split(text_stream)
        assert len(sentences) == 2
        assert sentences[0] == "This is from StringIO."
        assert sentences[1] == "It works!"

    def test_bytesio_input(self):
        """Test with BytesIO binary stream."""
        text = "This is from BytesIO. It also works!"
        binary_stream = io.BytesIO(text.encode("utf-8"))

        sentences = sakurs.split(binary_stream)
        assert len(sentences) == 2
        assert sentences[0] == "This is from BytesIO."
        assert sentences[1] == "It also works!"

    def test_file_object_text_mode(self):
        """Test with actual file object in text mode."""
        with tempfile.NamedTemporaryFile(mode="w+", suffix=".txt", delete=True) as f:
            f.write("File object test. In text mode.")
            f.seek(0)  # Reset to beginning

            sentences = sakurs.split(f)
            assert len(sentences) == 2
            assert sentences[0] == "File object test."
            assert sentences[1] == "In text mode."

    def test_file_object_binary_mode(self):
        """Test with actual file object in binary mode."""
        with tempfile.NamedTemporaryFile(mode="w+b", suffix=".txt", delete=True) as f:
            text = "File object test. In binary mode."
            f.write(text.encode("utf-8"))
            f.seek(0)  # Reset to beginning

            sentences = sakurs.split(f)
            assert len(sentences) == 2
            assert sentences[0] == "File object test."
            assert sentences[1] == "In binary mode."

    def test_stringio_with_unicode(self):
        """Test StringIO with Unicode content."""
        text_stream = io.StringIO("日本語のテスト。動作確認です。")

        sentences = sakurs.split(text_stream, language="ja")
        assert len(sentences) == 2
        assert sentences[0] == "日本語のテスト。"
        assert sentences[1] == "動作確認です。"

    def test_bytesio_with_different_encoding(self):
        """Test BytesIO with non-UTF-8 encoding."""
        text = "Latin-1 encoding test. With café!"
        binary_stream = io.BytesIO(text.encode("latin-1"))

        sentences = sakurs.split(binary_stream, encoding="latin-1")
        assert len(sentences) == 2
        assert "café" in sentences[1]

    def test_custom_file_like_object(self):
        """Test with custom file-like object that implements read()."""

        class CustomReader:
            def __init__(self, text):
                self.text = text
                self.position = 0

            def read(self, size=-1):
                if size == -1:
                    result = self.text[self.position :]
                    self.position = len(self.text)
                else:
                    result = self.text[self.position : self.position + size]
                    self.position += len(result)
                return result

        reader = CustomReader("Custom reader test. It implements read()!")
        sentences = sakurs.split(reader)
        assert len(sentences) == 2
        assert sentences[0] == "Custom reader test."
        assert sentences[1] == "It implements read()!"

    def test_empty_stream(self):
        """Test with empty stream."""
        empty_stream = io.StringIO("")
        sentences = sakurs.split(empty_stream)
        assert sentences == []

    def test_filelike_with_return_details(self):
        """Test file-like input with return_details=True."""
        text_stream = io.StringIO("First. Second.")

        results = sakurs.split(text_stream, return_details=True)
        assert len(results) == 2

        assert results[0].text == "First."
        assert results[0].start == 0
        assert results[0].end == 6

        assert results[1].text == "Second."
        assert results[1].start == 6  # Same offset issue - space not included
        assert results[1].end == 14

    def test_processor_with_filelike_input(self):
        """Test Processor class with file-like input."""
        processor = sakurs.SentenceSplitter()

        text_stream = io.StringIO("Testing processor. With StringIO!")
        sentences = processor.split(text_stream)
        assert len(sentences) == 2
        assert sentences[0] == "Testing processor."
        assert sentences[1] == "With StringIO!"

    def test_invalid_file_like_object(self):
        """Test with object that doesn't have read() method."""

        class NotAFileObject:
            def __init__(self):
                self.data = "This won't work"

        invalid_obj = NotAFileObject()

        with pytest.raises(TypeError) as exc_info:
            sakurs.split(invalid_obj)  # type: ignore[call-overload]
        assert "Expected str, bytes, Path, or file-like object" in str(exc_info.value)

    def test_read_returns_bytes_in_text_mode(self):
        """Test handling when read() returns bytes (binary mode file)."""

        class BinaryReader:
            def read(self):
                return b"Binary data. From custom reader."

        reader = BinaryReader()
        sentences = sakurs.split(reader)  # type: ignore[call-overload]
        assert len(sentences) == 2
        assert sentences[0] == "Binary data."
        assert sentences[1] == "From custom reader."

    def test_large_stream(self):
        """Test with large stream to verify memory efficiency."""
        # Create a large text in memory
        large_text = " ".join(f"Sentence number {i}." for i in range(1000))
        text_stream = io.StringIO(large_text)

        sentences = sakurs.split(text_stream)
        assert len(sentences) == 1000
        assert sentences[0] == "Sentence number 0."
        assert sentences[999] == "Sentence number 999."
