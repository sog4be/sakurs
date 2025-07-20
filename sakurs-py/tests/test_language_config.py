"""Test LanguageConfig and related configuration classes."""

import tempfile
from pathlib import Path

import pytest

import sakurs


def test_create_minimal_language_config():
    """Test creating a minimal LanguageConfig."""
    metadata = sakurs.MetadataConfig("test", "Test Language")
    terminators = sakurs.TerminatorConfig([".", "!", "?"])
    ellipsis = sakurs.EllipsisConfig()
    enclosures = sakurs.EnclosureConfig([])
    suppression = sakurs.SuppressionConfig()
    abbreviations = sakurs.AbbreviationConfig()

    config = sakurs.LanguageConfig(
        metadata=metadata,
        terminators=terminators,
        ellipsis=ellipsis,
        enclosures=enclosures,
        suppression=suppression,
        abbreviations=abbreviations,
        sentence_starters=None,
    )

    assert config.metadata.code == "test"
    assert config.metadata.name == "Test Language"
    assert config.terminators.chars == [".", "!", "?"]
    assert config.sentence_starters is None


def test_create_full_language_config():
    """Test creating a full LanguageConfig with all features."""
    # Metadata
    metadata = sakurs.MetadataConfig("custom", "Custom Language")

    # Terminators with patterns
    terminator_pattern = sakurs.TerminatorPattern("!?", "surprised_question")
    terminators = sakurs.TerminatorConfig(
        chars=[".", "!", "?"], patterns=[terminator_pattern]
    )

    # Ellipsis with rules
    context_rule = sakurs.ContextRule("followed_by_capital", True)
    exception_pattern = sakurs.ExceptionPattern(r"\b(um|uh)\.\.\.", False)
    ellipsis = sakurs.EllipsisConfig(
        treat_as_boundary=True,
        patterns=["...", "…"],
        context_rules=[context_rule],
        exceptions=[exception_pattern],
    )

    # Enclosures
    pair1 = sakurs.EnclosurePair("(", ")")
    pair2 = sakurs.EnclosurePair('"', '"', symmetric=True)
    enclosures = sakurs.EnclosureConfig([pair1, pair2])

    # Suppression
    fast_pattern = sakurs.FastPattern("'", before="alpha", after="alpha")
    regex_pattern = sakurs.RegexPattern(r"\d+\.\d+", "decimal numbers")
    suppression = sakurs.SuppressionConfig(
        fast_patterns=[fast_pattern], regex_patterns=[regex_pattern]
    )

    # Abbreviations
    abbreviations = sakurs.AbbreviationConfig(
        titles=["Dr", "Mr", "Mrs"], common=["etc", "vs", "e.g"]
    )

    # Sentence starters
    sentence_starters = sakurs.SentenceStarterConfig(
        require_following_space=True,
        min_word_length=2,
        pronouns=["He", "She", "They"],
        articles=["The", "A", "An"],
    )

    config = sakurs.LanguageConfig(
        metadata=metadata,
        terminators=terminators,
        ellipsis=ellipsis,
        enclosures=enclosures,
        suppression=suppression,
        abbreviations=abbreviations,
        sentence_starters=sentence_starters,
    )

    # Verify all components
    assert config.metadata.code == "custom"
    assert len(config.terminators.patterns) == 1
    assert len(config.ellipsis.context_rules) == 1
    assert len(config.enclosures.pairs) == 2
    assert len(config.suppression.fast_patterns) == 1
    assert config.abbreviations["titles"] == ["Dr", "Mr", "Mrs"]
    assert config.sentence_starters is not None
    assert config.sentence_starters.require_following_space is True


def test_load_language_config_from_toml():
    """Test loading a LanguageConfig from a TOML file."""
    toml_content = """
[metadata]
code = "test"
name = "Test Language"

[terminators]
chars = [".", "!", "?"]
patterns = [
    { pattern = "!?", name = "surprised" }
]

[ellipsis]
treat_as_boundary = true
patterns = ["...", "…"]

[enclosures]
pairs = [
    { open = "(", close = ")" },
    { open = "'", close = "'", symmetric = true }
]

[suppression]
fast_patterns = [
    { char = "'", before = "alpha", after = "alpha" }
]

[abbreviations]
titles = ["Dr", "Mr"]
common = ["etc", "vs"]
"""

    with tempfile.NamedTemporaryFile(mode="w", suffix=".toml", delete=False) as f:
        f.write(toml_content)
        temp_path = Path(f.name)

    try:
        config = sakurs.LanguageConfig.from_toml(temp_path)

        assert config.metadata.code == "test"
        assert config.metadata.name == "Test Language"
        assert config.terminators.chars == [".", "!", "?"]
        assert len(config.terminators.patterns) == 1
        assert config.enclosures.pairs[0].open == "("
        assert config.enclosures.pairs[1].symmetric is True
        assert config.abbreviations["titles"] == ["Dr", "Mr"]
    finally:
        temp_path.unlink()


def test_save_language_config_to_toml():
    """Test saving a LanguageConfig to a TOML file."""
    # Create a config
    metadata = sakurs.MetadataConfig("save_test", "Save Test Language")
    terminators = sakurs.TerminatorConfig([".", "!"])
    ellipsis = sakurs.EllipsisConfig(patterns=["..."])
    enclosures = sakurs.EnclosureConfig([sakurs.EnclosurePair("{", "}")])
    suppression = sakurs.SuppressionConfig()
    abbreviations = sakurs.AbbreviationConfig(titles=["Prof"])

    config = sakurs.LanguageConfig(
        metadata=metadata,
        terminators=terminators,
        ellipsis=ellipsis,
        enclosures=enclosures,
        suppression=suppression,
        abbreviations=abbreviations,
        sentence_starters=None,
    )

    with tempfile.NamedTemporaryFile(suffix=".toml", delete=False) as f:
        temp_path = Path(f.name)

    try:
        # Save and reload
        config.to_toml(temp_path)
        loaded_config = sakurs.LanguageConfig.from_toml(temp_path)

        assert loaded_config.metadata.code == "save_test"
        assert loaded_config.terminators.chars == [".", "!"]
        assert loaded_config.ellipsis.patterns == ["..."]
        assert loaded_config.enclosures.pairs[0].open == "{"
        assert loaded_config.abbreviations["titles"] == ["Prof"]
    finally:
        temp_path.unlink()


def test_split_with_custom_language_config():
    """Test using split() with a custom LanguageConfig."""
    # Create a custom config that only uses "!" as terminator
    metadata = sakurs.MetadataConfig("exclaim", "Exclamation Only")
    terminators = sakurs.TerminatorConfig(["!"])  # Only exclamation marks
    ellipsis = sakurs.EllipsisConfig()
    enclosures = sakurs.EnclosureConfig([])
    suppression = sakurs.SuppressionConfig()
    abbreviations = sakurs.AbbreviationConfig()

    config = sakurs.LanguageConfig(
        metadata=metadata,
        terminators=terminators,
        ellipsis=ellipsis,
        enclosures=enclosures,
        suppression=suppression,
        abbreviations=abbreviations,
        sentence_starters=None,
    )

    # Test text with periods and exclamation marks
    text = "Hello world. This won't split! But this will! And this. Not this!"

    sentences = sakurs.split(text, language_config=config)

    # Should only split on exclamation marks
    assert len(sentences) == 3
    assert sentences[0] == "Hello world. This won't split!"
    assert sentences[1] == "But this will!"
    assert sentences[2] == "And this. Not this!"


def test_processor_with_custom_language_config():
    """Test creating a Processor with custom LanguageConfig."""
    # Create a simple custom config
    metadata = sakurs.MetadataConfig("custom", "Custom Language")
    terminators = sakurs.TerminatorConfig([".", "?"])
    ellipsis = sakurs.EllipsisConfig()
    enclosures = sakurs.EnclosureConfig(
        [
            sakurs.EnclosurePair("(", ")"),
        ]
    )
    suppression = sakurs.SuppressionConfig()
    abbreviations = sakurs.AbbreviationConfig(titles=["Dr", "Prof"])

    config = sakurs.LanguageConfig(
        metadata=metadata,
        terminators=terminators,
        ellipsis=ellipsis,
        enclosures=enclosures,
        suppression=suppression,
        abbreviations=abbreviations,
        sentence_starters=None,
    )

    # Create processor with custom config
    processor = sakurs.Processor(language_config=config)

    # Test with abbreviations
    text = "Dr. Smith asked a question? Prof. Jones answered."
    sentences = processor.split(text)

    assert len(sentences) == 2
    assert "Dr. Smith" in sentences[0]
    assert "Prof. Jones" in sentences[1]


def test_abbreviation_config_dict_interface():
    """Test AbbreviationConfig dictionary-like interface."""
    abbreviations = sakurs.AbbreviationConfig(
        titles=["Dr", "Mr", "Mrs"], locations=["St", "Ave", "Blvd"]
    )

    # Test __getitem__
    assert abbreviations["titles"] == ["Dr", "Mr", "Mrs"]
    assert abbreviations["locations"] == ["St", "Ave", "Blvd"]

    # Test __setitem__
    abbreviations["academic"] = ["Ph.D", "M.D"]
    assert abbreviations["academic"] == ["Ph.D", "M.D"]

    # Test KeyError for non-existent key
    with pytest.raises(KeyError):
        _ = abbreviations["nonexistent"]


def test_config_repr_methods():
    """Test __repr__ methods for all config classes."""
    metadata = sakurs.MetadataConfig("en", "English")
    assert "code='en'" in repr(metadata)
    assert "name='English'" in repr(metadata)

    terminator = sakurs.TerminatorPattern("!?", "surprised")
    assert "pattern='!?'" in repr(terminator)

    terminators = sakurs.TerminatorConfig([".", "!"])
    assert 'chars=[".", "!"]' in repr(terminators)

    pair = sakurs.EnclosurePair("(", ")", symmetric=False)
    assert "open='('" in repr(pair)
    assert "close=')'" in repr(pair)

    config = sakurs.LanguageConfig(
        metadata=metadata,
        terminators=terminators,
        ellipsis=sakurs.EllipsisConfig(),
        enclosures=sakurs.EnclosureConfig([]),
        suppression=sakurs.SuppressionConfig(),
        abbreviations=sakurs.AbbreviationConfig(),
        sentence_starters=None,
    )
    assert "code='en'" in repr(config)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
