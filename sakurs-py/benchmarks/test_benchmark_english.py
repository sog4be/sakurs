"""English sentence segmentation benchmarks comparing sakurs vs PySBD."""

from typing import Final

import pysbd
import pytest
from pytest_benchmark.fixture import BenchmarkFixture

import sakurs


@pytest.fixture
def sakurs_processor_en() -> sakurs.SentenceSplitter:
    """Create and reuse sakurs English processor."""
    return sakurs.load("en")


@pytest.fixture
def pysbd_segmenter() -> pysbd.Segmenter:
    """Create and reuse PySBD segmenter."""
    return pysbd.Segmenter(language="en", clean=False)


# Benchmark configuration constants
LARGE_TEXT_ITERATIONS: Final[int] = 1
LARGE_TEXT_ROUNDS: Final[int] = 3


class TestEnglishBenchmarks:
    """Benchmark tests for English sentence segmentation."""

    def _create_large_text(self, base_text: str, multiplier: int) -> str:
        """Create large text by repeating with space separation for English."""
        # Add space between repetitions to avoid word concatenation
        return " ".join([base_text] * multiplier)

    def test_sakurs_english_400(
        self,
        benchmark: BenchmarkFixture,
        english_text_400: str,
        sakurs_processor_en: sakurs.SentenceSplitter,
    ) -> list[str]:
        """Benchmark sakurs on 400-character English text."""
        result = benchmark(sakurs_processor_en.split, english_text_400)
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_pysbd_english_400(
        self,
        benchmark: BenchmarkFixture,
        english_text_400: str,
        pysbd_segmenter: pysbd.Segmenter,
    ) -> list[str]:
        """Benchmark PySBD on 400-character English text."""
        result = benchmark(pysbd_segmenter.segment, english_text_400)
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_sakurs_english_large(
        self,
        benchmark: BenchmarkFixture,
        english_text_400: str,
        large_text_multiplier: int,
        sakurs_processor_en: sakurs.SentenceSplitter,
    ) -> None:
        """Benchmark sakurs on large English text."""
        # Create large text by repeating the sample with spaces
        large_text = self._create_large_text(english_text_400, large_text_multiplier)

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            sakurs_processor_en.split,
            args=(large_text,),
            iterations=LARGE_TEXT_ITERATIONS,
            rounds=LARGE_TEXT_ROUNDS,
        )

    def test_pysbd_english_large(
        self,
        benchmark: BenchmarkFixture,
        english_text_400: str,
        large_text_multiplier: int,
        pysbd_segmenter: pysbd.Segmenter,
    ) -> None:
        """Benchmark PySBD on large English text."""
        # Create large text by repeating the sample with spaces
        large_text = self._create_large_text(english_text_400, large_text_multiplier)

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            pysbd_segmenter.segment,
            args=(large_text,),
            iterations=LARGE_TEXT_ITERATIONS,
            rounds=LARGE_TEXT_ROUNDS,
        )
