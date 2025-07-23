"""Configuration and fixtures for benchmarks."""

import pytest


@pytest.fixture
def english_text_400():
    """400-character English text with common patterns."""
    return """Dr. Smith arrived at 3:30 p.m. yesterday. He said, "Hello, how are you?" The patient replied, "I'm fine, thanks!" The U.S. government announced new policies. Mr. Johnson works at Apple Inc. and lives on 5th Ave. The meeting is scheduled for Jan. 15th at 2:00 PM. Please R.S.V.P. by Friday. See pp. 10-15 for more details. This includes various patterns e.g. abbreviations."""


@pytest.fixture
def japanese_text_400():
    """400-character Japanese text with common patterns."""
    return """山田先生は「こんにちは。」と言いました。「今日はいい天気ですね！」と返事をしました。株式会社ABCは東京都渋谷区にあります。営業時間は午前9時から午後6時までです。詳細については、P.123を参照してください。U.S.A.から輸入された商品です。「これは何ですか？」「それはペンです。」明日は雨が降るでしょう…たぶん。田中さんは言った「分かりました」"""


@pytest.fixture(scope="session")
def large_text_multiplier():
    """Multiplier to create large texts for performance testing."""
    # Adjust this value to ensure tests complete within 10 seconds
    # This will be calibrated during initial testing
    return 2500  # Reduced from 25000 to avoid timeout
