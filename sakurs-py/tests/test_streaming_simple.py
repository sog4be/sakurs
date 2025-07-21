"""Simple test to verify streaming functionality."""

import io

import sakurs


def test_basic_streaming():
    """Test basic streaming functionality."""
    print("Testing basic streaming...")

    # Test 1: Stream from text
    text = "Hello world. This is a test. Another sentence?"
    sentences = list(sakurs.stream_split(text))
    print(f"Text streaming: {sentences}")
    assert sentences == ["Hello world.", "This is a test.", "Another sentence?"]

    # Test 2: Stream from BytesIO
    bytes_io = io.BytesIO(b"One. Two. Three.")
    sentences = list(sakurs.stream_split(bytes_io))
    print(f"BytesIO streaming: {sentences}")
    assert sentences == ["One.", "Two.", "Three."]

    # Test 3: Stream with Japanese
    text = "これは文です。もう一つの文。"
    sentences = list(sakurs.stream_split(text, language="ja"))
    print(f"Japanese streaming: {sentences}")
    assert len(sentences) == 2

    # Test 4: Processor iter_split
    processor = sakurs.load("en")
    sentences = list(processor.iter_split("First. Second. Third."))
    print(f"Processor streaming: {sentences}")
    assert sentences == ["First.", "Second.", "Third."]

    print("\nAll tests passed! ✅")


if __name__ == "__main__":
    test_basic_streaming()
