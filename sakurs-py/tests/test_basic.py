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
    assert hasattr(processor, "sentences")  # Legacy support


def test_unsupported_language_error():
    """Test error handling for unsupported languages."""
    if sakurs is None:
        pytest.skip("sakurs module not built yet")

    with pytest.raises(RuntimeError, match="Unsupported language"):
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
