"""Configuration and fixtures for benchmarks."""

from typing import Final

import pytest


@pytest.fixture
def english_text_400() -> str:
    """400-character English text with common patterns."""
    return """Mr. Baker, a coder from the U.S., drafted the following line: "Parser ready (v2.3 passes.) now." Can the server at 192.168.1.1 parse every case? Yes! Watch it stumble on sequences like e.g. ellipses... or does it!? Despite surprises, the module logs "Done (all tests ok.)" before midnight. Each token rides its boundary, yet pesky abbreviations lurk: Prof., Dr., St., etc., all set to trip splitters."""


@pytest.fixture
def japanese_text_400() -> str:
    """400-character Japanese text with common patterns."""
    return """文分割OSSの動作確認用サンプルとして、本稿では多様な構文や句読点を織り交ぜた四百文字ぴったりの文章を提示する。まず、冒頭で目的を簡潔に述べ、続けて条件を満たす特殊な鉤括弧表現を挿入する。それが「この節では、システムが正しく文を区切れるかを試すために『入れ子構造が含まれる文です。』と宣言し、内部にも句点を置いた。」という部分だ。さらに、助詞の省略や倒置を利用して自然な日本語を維持しつつ、語彙の重複を避ける。また、読点と中黒を適切に配し、視認性を向上させる。ここまでで約三百文字に満たないため、さらに字数を稼ぐ工夫として、典型的な敬語、引用、列挙の語法も盛り込もう。例えば、開発者は「期待どおりに区切られましたか？」と問い掛け、テスターは「はい、問題ありません！」と応じる対話を想定する。こうした会話体は解析器にとっても挑戦的であり、数字1と英字Aを挿入して補うことで正確性の検証に寄与するだろう！"""


# Constants for benchmark configuration
LARGE_TEXT_MULTIPLIER: Final[int] = (
    550  # Optimized for <30 second benchmark on slowest case (PySBD)
)


@pytest.fixture(scope="session")
def large_text_multiplier() -> int:
    """Multiplier to create large texts for performance testing."""
    # Adjusted to ensure all benchmarks complete within 30 seconds
    # The slowest case (PySBD on English text) takes ~28 seconds with this multiplier
    return LARGE_TEXT_MULTIPLIER
