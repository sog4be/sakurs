"""Japanese sentence segmentation benchmarks comparing sakurs vs ja_sentence_segmenter."""

import functools

from ja_sentence_segmenter.common.pipeline import make_pipeline
from ja_sentence_segmenter.concatenate.simple_concatenator import concatenate_matching
from ja_sentence_segmenter.normalize.neologd_normalizer import normalize
from ja_sentence_segmenter.split.simple_splitter import split_newline, split_punctuation

import sakurs


class TestJapaneseBenchmarks:
    """Benchmark tests for Japanese sentence segmentation."""

    def _create_ja_segmenter(self):
        """Create ja_sentence_segmenter pipeline."""
        split_punc = functools.partial(split_punctuation, punctuations=r"。!?")
        concat_tail_no = functools.partial(
            concatenate_matching,
            former_matching_rule=r"^(?P<r>.+)(の)$",
            remove_former_matched=False,
        )
        return make_pipeline(normalize, split_newline, concat_tail_no, split_punc)

    def test_sakurs_japanese_400(self, benchmark, japanese_text_400):
        """Benchmark sakurs on 400-character Japanese text."""
        result = benchmark(sakurs.split, japanese_text_400, language="ja")
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_ja_segmenter_japanese_400(self, benchmark, japanese_text_400):
        """Benchmark ja_sentence_segmenter on 400-character Japanese text."""
        segmenter = self._create_ja_segmenter()
        result = benchmark(lambda text: list(segmenter(text)), japanese_text_400)
        assert isinstance(result, list)
        assert len(result) > 0

        # Store segmentation results in benchmark data
        benchmark.extra_info["segmentation"] = {
            "sentences": result,
            "count": len(result),
        }

        return result

    def test_sakurs_japanese_large(
        self, benchmark, japanese_text_400, large_text_multiplier
    ):
        """Benchmark sakurs on large Japanese text."""
        # Create large text by repeating the sample
        # Use fixed multiplier for Japanese: 200 repetitions
        multiplier = 200
        large_text = japanese_text_400 * multiplier

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            sakurs.split,
            args=(large_text,),
            kwargs={"language": "ja"},
            iterations=1,
            rounds=3,
        )

    def test_ja_segmenter_japanese_large(
        self, benchmark, japanese_text_400, large_text_multiplier
    ):
        """Benchmark ja_sentence_segmenter on large Japanese text."""
        # Create large text by repeating the sample
        # Use fixed multiplier for Japanese: 200 repetitions
        multiplier = 200
        large_text = japanese_text_400 * multiplier
        segmenter = self._create_ja_segmenter()

        # Set a reasonable timeout to prevent hanging
        benchmark.pedantic(
            lambda text: list(segmenter(text)),
            args=(large_text,),
            iterations=1,
            rounds=3,
        )
