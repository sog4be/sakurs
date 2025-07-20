"""Tests for bytes input handling."""

import pytest

import sakurs


class TestBytesInput:
    """Test bytes input functionality."""

    def test_utf8_bytes_input(self):
        """Test with UTF-8 encoded bytes."""
        text = "This is UTF-8 bytes. It should work!"
        bytes_input = text.encode("utf-8")

        sentences = sakurs.split(bytes_input)
        assert len(sentences) == 2
        assert sentences[0] == "This is UTF-8 bytes."
        assert sentences[1] == "It should work!"

    def test_ascii_bytes_input(self):
        """Test with ASCII encoded bytes."""
        text = "Pure ASCII text. No special characters."
        bytes_input = text.encode("ascii")

        sentences = sakurs.split(bytes_input, encoding="ascii")
        assert len(sentences) == 2
        assert sentences[0] == "Pure ASCII text."
        assert sentences[1] == "No special characters."

    def test_latin1_bytes_input(self):
        """Test with Latin-1 encoded bytes."""
        text = "Latin-1 text with café. Très bien!"
        bytes_input = text.encode("latin-1")

        sentences = sakurs.split(bytes_input, encoding="latin-1")
        assert len(sentences) == 2
        assert "café" in sentences[0]
        assert "Très bien!" in sentences[1]

    def test_unicode_bytes(self):
        """Test bytes containing Unicode characters."""
        text = "Unicode test with émojis. And symbols: α, β, γ!"
        bytes_input = text.encode("utf-8")

        sentences = sakurs.split(bytes_input)
        assert len(sentences) == 2
        assert "émojis" in sentences[0]
        assert "α, β, γ!" in sentences[1]

    def test_japanese_bytes(self):
        """Test bytes containing Japanese text."""
        text = "これは日本語のバイト列です。正しく処理されるはずです。"
        bytes_input = text.encode("utf-8")

        sentences = sakurs.split(bytes_input, language="ja")
        assert len(sentences) == 2
        assert sentences[0] == "これは日本語のバイト列です。"
        assert sentences[1] == "正しく処理されるはずです。"

    def test_empty_bytes(self):
        """Test with empty bytes."""
        empty_bytes = b""
        sentences = sakurs.split(empty_bytes)
        assert sentences == []

    def test_invalid_utf8_bytes(self):
        """Test with invalid UTF-8 bytes."""
        # Create invalid UTF-8 sequence
        invalid_bytes = b"\xff\xfe Invalid UTF-8"

        with pytest.raises(sakurs.SakursError, match="Failed to decode bytes as UTF-8"):
            sakurs.split(invalid_bytes)

    def test_non_ascii_with_ascii_encoding(self):
        """Test non-ASCII bytes with ASCII encoding specified."""
        text = "Text with café"
        bytes_input = text.encode("utf-8")

        # This should fail because café contains non-ASCII
        with pytest.raises(sakurs.SakursError, match="Failed to decode bytes as ASCII"):
            sakurs.split(bytes_input, encoding="ascii")

    def test_bytes_with_return_details(self):
        """Test bytes input with return_details=True."""
        text = "First sentence. Second sentence."
        bytes_input = text.encode("utf-8")

        results = sakurs.split(bytes_input, return_details=True)
        assert len(results) == 2

        assert results[0].text == "First sentence."
        assert results[0].start == 0
        assert results[0].end == 15

        assert results[1].text == "Second sentence."
        assert results[1].start == 16
        assert results[1].end == 32

    def test_processor_with_bytes_input(self):
        """Test Processor class with bytes input."""
        processor = sakurs.Processor()

        text = "Testing processor. With bytes input!"
        bytes_input = text.encode("utf-8")

        sentences = processor.split(bytes_input)
        assert len(sentences) == 2
        assert sentences[0] == "Testing processor."
        assert sentences[1] == "With bytes input!"

    def test_large_bytes_input(self):
        """Test with large bytes input."""
        # Create large text
        text_parts = [f"Sentence number {i}." for i in range(100)]
        large_text = " ".join(text_parts)
        large_bytes = large_text.encode("utf-8")

        sentences = sakurs.split(large_bytes)
        assert len(sentences) == 100
        assert sentences[0] == "Sentence number 0."
        assert sentences[99] == "Sentence number 99."

    def test_unsupported_encoding(self):
        """Test with unsupported encoding."""
        text = "Some text"
        bytes_input = text.encode("utf-8")

        with pytest.raises(sakurs.SakursError, match="Unsupported encoding"):
            sakurs.split(bytes_input, encoding="shift-jis")

    def test_bytes_with_bom(self):
        """Test bytes with UTF-8 BOM."""
        # UTF-8 BOM followed by text
        bytes_with_bom = b"\xef\xbb\xbf" + b"Text with BOM. Should be handled."

        sentences = sakurs.split(bytes_with_bom)
        assert len(sentences) == 2
        # The BOM should be handled transparently
        assert sentences[0] == "Text with BOM."
        assert sentences[1] == "Should be handled."
