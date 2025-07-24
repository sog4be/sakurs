"""English sentence segmentation benchmarks comparing sakurs vs PySBD."""

import pysbd

import sakurs


class TestEnglishBenchmarks:
    """Benchmark tests for English sentence segmentation."""

    def test_sakurs_english_400(self, benchmark, english_text_400):
        """Benchmark sakurs on 400-character English text."""
        result = benchmark(sakurs.split, english_text_400, language="en")
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_pysbd_english_400(self, benchmark, english_text_400):
        """Benchmark PySBD on 400-character English text."""
        seg = pysbd.Segmenter(language="en", clean=False)
        result = benchmark(seg.segment, english_text_400)
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_sakurs_english_large(
        self, benchmark, english_text_400, large_text_multiplier
    ):
        """Benchmark sakurs on large English text."""
        # Create large text by repeating the sample
        large_text = english_text_400 * large_text_multiplier

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            sakurs.split,
            args=(large_text,),
            kwargs={"language": "en"},
            iterations=1,
            rounds=3,
        )

    def test_pysbd_english_large(
        self, benchmark, english_text_400, large_text_multiplier
    ):
        """Benchmark PySBD on large English text."""
        # Create large text by repeating the sample
        large_text = english_text_400 * large_text_multiplier
        seg = pysbd.Segmenter(language="en", clean=False)

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(seg.segment, args=(large_text,), iterations=1, rounds=3)
