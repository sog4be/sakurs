"""Basic tests for sakurs Python bindings."""

import pytest

# Import sakurs for testing once it's built
# We'll handle ImportError in tests to provide better error messages
try:
    import sakurs
except ImportError:
    sakurs = None  # type: ignore[assignment]

# Constants
MIN_SUPPORTED_LANGUAGES = 2


def test_import():
    """Test that sakurs can be imported."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    assert hasattr(sakurs, "__version__")
    assert hasattr(sakurs, "split")
    assert hasattr(sakurs, "load")
    assert hasattr(sakurs, "Processor")


def test_supported_languages():
    """Test supported languages function."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    languages = sakurs.supported_languages()
    assert isinstance(languages, list)
    assert len(languages) >= MIN_SUPPORTED_LANGUAGES
    assert "en" in languages or "english" in languages
    assert "ja" in languages or "japanese" in languages


@pytest.mark.parametrize("language", ["en", "english", "ja", "japanese"])
def test_processor_creation(language):
    """Test creating processors for supported languages."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    processor = sakurs.load(language)
    assert processor is not None
    assert hasattr(processor, "split")


def test_unsupported_language_error():
    """Test error handling for unsupported languages."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    with pytest.raises(sakurs.InvalidLanguageError):
        sakurs.load("unsupported_language")


def test_basic_sentence_tokenization():
    """Test basic sentence tokenization."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    text = "Hello world. How are you? I am fine!"
    sentences = sakurs.split(text)
    assert isinstance(sentences, list)
    assert len(sentences) > 0
    assert all(isinstance(s, str) for s in sentences)


@pytest.mark.parametrize(
    ("text", "min_sentences"),
    [
        ("One sentence.", 1),
        ("First. Second.", 2),
        ("First? Second! Third.", 3),
    ],
)
def test_sentence_count(text, min_sentences):
    """Test that sentence count is reasonable."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    sentences = sakurs.split(text)
    assert len(sentences) >= min_sentences


def test_japanese_basic():
    """Test basic Japanese sentence tokenization."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    # Test with common Japanese punctuation
    text = "こんにちは。元気ですか？はい、元気です！"
    sentences = sakurs.split(text, language="ja")

    assert len(sentences) == 3
    assert sentences[0] == "こんにちは。"
    assert sentences[1] == "元気ですか？"
    assert sentences[2] == "はい、元気です！"


def test_japanese_with_brackets():
    """Test Japanese sentence tokenization with brackets."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    # Test with Japanese quotation marks (鉤括弧)
    text = "彼は「おはよう！」と言った。「本当ですか？」と私は聞きました。"
    sentences = sakurs.split(text, language="ja")
    assert len(sentences) == 2
    assert sentences[0] == "彼は「おはよう！」と言った。"
    assert sentences[1] == "「本当ですか？」と私は聞きました。"


def test_japanese_mixed_punctuation():
    """Test Japanese with various punctuation marks."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    # Test with mixed punctuation including exclamation marks
    text = "すごい！これは素晴らしいです。「本当に？」と彼女は尋ねた。はい、本当です！"
    sentences = sakurs.split(text, language="ja")
    assert len(sentences) == 4
    assert sentences[0] == "すごい！"
    assert sentences[1] == "これは素晴らしいです。"
    assert sentences[2] == "「本当に？」と彼女は尋ねた。"
    assert sentences[3] == "はい、本当です！"


def test_japanese_nested_quotes():
    """Test Japanese with nested quotation marks."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    # Test with nested quotes and various end punctuation
    text = "田中さんは「彼女が『すごい！』と言った」と話しました。面白いですね？"
    sentences = sakurs.split(text, language="ja")
    assert len(sentences) == 2
    # First sentence contains nested quotes
    assert "『すごい！』" in sentences[0]
    assert sentences[1] == "面白いですね？"
