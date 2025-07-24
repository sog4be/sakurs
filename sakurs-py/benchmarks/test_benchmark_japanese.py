"""Japanese sentence segmentation benchmarks comparing sakurs vs ja_sentence_segmenter."""

import functools

import pytest
from ja_sentence_segmenter.common.pipeline import make_pipeline
from ja_sentence_segmenter.concatenate.simple_concatenator import concatenate_matching
from ja_sentence_segmenter.normalize.neologd_normalizer import normalize
from ja_sentence_segmenter.split.simple_splitter import split_newline, split_punctuation

import sakurs


@pytest.fixture()
def sakurs_processor_ja():
    """Create and reuse sakurs Japanese processor."""
    return sakurs.load("ja")


@pytest.fixture()
def ja_segmenter():
    """Create and reuse ja_sentence_segmenter pipeline."""
    split_punc = functools.partial(split_punctuation, punctuations=r"。!?")
    concat_tail_no = functools.partial(
        concatenate_matching,
        former_matching_rule=r"^(?P<r>.+)(の)$",
        remove_former_matched=False,
    )
    return make_pipeline(normalize, split_newline, concat_tail_no, split_punc)


class TestJapaneseBenchmarks:
    """Benchmark tests for Japanese sentence segmentation."""

    def _create_large_text(self, base_text: str, multiplier: int) -> str:
        """Create large text by repeating without separator for Japanese."""
        # Japanese doesn't need spaces between repetitions
        return base_text * multiplier

    def test_sakurs_japanese_400(
        self, benchmark, japanese_text_400, sakurs_processor_ja
    ):
        """Benchmark sakurs on 400-character Japanese text."""
        result = benchmark(sakurs_processor_ja.split, japanese_text_400)
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_ja_segmenter_japanese_400(
        self, benchmark, japanese_text_400, ja_segmenter
    ):
        """Benchmark ja_sentence_segmenter on 400-character Japanese text."""
        result = benchmark(lambda text: list(ja_segmenter(text)), japanese_text_400)
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_sakurs_japanese_large(
        self, benchmark, japanese_text_400, large_text_multiplier, sakurs_processor_ja
    ):
        """Benchmark sakurs on large Japanese text."""
        # Create large text by repeating the sample
        # Use fixed multiplier for Japanese: 200 repetitions
        multiplier = 200
        large_text = self._create_large_text(japanese_text_400, multiplier)

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            sakurs_processor_ja.split,
            args=(large_text,),
            iterations=1,
            rounds=3,
        )

    def test_ja_segmenter_japanese_large(
        self, benchmark, japanese_text_400, large_text_multiplier, ja_segmenter
    ):
        """Benchmark ja_sentence_segmenter on large Japanese text."""
        # Create large text by repeating the sample
        # Use fixed multiplier for Japanese: 200 repetitions
        multiplier = 200
        large_text = self._create_large_text(japanese_text_400, multiplier)

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            lambda text: list(ja_segmenter(text)),
            args=(large_text,),
            iterations=1,
            rounds=3,
        )
