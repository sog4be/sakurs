"""English sentence segmentation benchmarks comparing sakurs vs PySBD."""

import pysbd
import pytest

import sakurs


@pytest.fixture()
def sakurs_processor_en():
    """Create and reuse sakurs English processor."""
    return sakurs.load("en")


@pytest.fixture()
def pysbd_segmenter():
    """Create and reuse PySBD segmenter."""
    return pysbd.Segmenter(language="en", clean=False)


class TestEnglishBenchmarks:
    """Benchmark tests for English sentence segmentation."""

    def _create_large_text(self, base_text: str, multiplier: int) -> str:
        """Create large text by repeating with space separation for English."""
        # Add space between repetitions to avoid word concatenation
        return " ".join([base_text] * multiplier)

    def test_sakurs_english_400(self, benchmark, english_text_400, sakurs_processor_en):
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

    def test_pysbd_english_400(self, benchmark, english_text_400, pysbd_segmenter):
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
        self, benchmark, english_text_400, large_text_multiplier, sakurs_processor_en
    ):
        """Benchmark sakurs on large English text."""
        # Create large text by repeating the sample with spaces
        large_text = self._create_large_text(english_text_400, large_text_multiplier)

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            sakurs_processor_en.split,
            args=(large_text,),
            iterations=1,
            rounds=3,
        )

    def test_pysbd_english_large(
        self, benchmark, english_text_400, large_text_multiplier, pysbd_segmenter
    ):
        """Benchmark PySBD on large English text."""
        # Create large text by repeating the sample with spaces
        large_text = self._create_large_text(english_text_400, large_text_multiplier)

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            pysbd_segmenter.segment, args=(large_text,), iterations=1, rounds=3
        )
