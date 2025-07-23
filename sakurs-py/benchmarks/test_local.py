#!/usr/bin/env python3
"""Quick local test to verify benchmarks work correctly."""

import subprocess
import sys


def test_imports():
    """Test that all required libraries can be imported."""
    print("Testing imports...")
    try:
        import sakurs

        print("✓ sakurs imported successfully")
    except ImportError as e:
        print(f"✗ Failed to import sakurs: {e}")
        return False

    try:
        import pysbd

        print("✓ pysbd imported successfully")
    except ImportError as e:
        print(f"✗ Failed to import pysbd: {e}")
        return False

    try:
        import ja_sentence_segmenter

        print("✓ ja_sentence_segmenter imported successfully")
    except ImportError as e:
        print(f"✗ Failed to import ja_sentence_segmenter: {e}")
        return False

    try:
        import pytest_benchmark

        print("✓ pytest_benchmark imported successfully")
    except ImportError as e:
        print(f"✗ Failed to import pytest_benchmark: {e}")
        return False

    return True


def test_basic_functionality():
    """Test basic functionality of each library."""
    print("\nTesting basic functionality...")

    # Test sakurs
    try:
        import sakurs

        result = sakurs.split("Hello world. How are you?", language="en")
        assert len(result) == 2
        print("✓ sakurs English: OK")

        result = sakurs.split("こんにちは。元気ですか？", language="ja")
        assert len(result) == 2
        print("✓ sakurs Japanese: OK")
    except Exception as e:
        print(f"✗ sakurs failed: {e}")
        return False

    # Test PySBD
    try:
        import pysbd

        seg = pysbd.Segmenter(language="en", clean=False)
        result = seg.segment("Hello world. How are you?")
        assert len(result) == 2
        print("✓ PySBD: OK")
    except Exception as e:
        print(f"✗ PySBD failed: {e}")
        return False

    # Test ja_sentence_segmenter
    try:
        import functools

        from ja_sentence_segmenter.common.pipeline import make_pipeline
        from ja_sentence_segmenter.concatenate.simple_concatenator import (
            concatenate_matching,
        )
        from ja_sentence_segmenter.normalize.neologd_normalizer import normalize
        from ja_sentence_segmenter.split.simple_splitter import (
            split_newline,
            split_punctuation,
        )

        split_punc = functools.partial(split_punctuation, punctuations=r"。!?")
        concat_tail_no = functools.partial(
            concatenate_matching,
            former_matching_rule=r"^(?P<r>.+)(の)$",
            remove_former_matched=False,
        )
        segmenter = make_pipeline(normalize, split_newline, concat_tail_no, split_punc)
        result = list(segmenter("こんにちは。元気ですか？"))
        assert len(result) >= 1
        print("✓ ja_sentence_segmenter: OK")
    except Exception as e:
        print(f"✗ ja_sentence_segmenter failed: {e}")
        return False

    return True


def run_benchmarks_subset():
    """Run a subset of benchmarks to verify they work."""
    print("\nRunning benchmark subset...")

    cmd = [
        sys.executable,
        "-m",
        "pytest",
        "benchmarks/test_benchmark_english.py::TestEnglishBenchmarks::test_sakurs_english_400",
        "benchmarks/test_benchmark_english.py::TestEnglishBenchmarks::test_pysbd_english_400",
        "benchmarks/test_benchmark_japanese.py::TestJapaneseBenchmarks::test_sakurs_japanese_400",
        "benchmarks/test_benchmark_japanese.py::TestJapaneseBenchmarks::test_ja_segmenter_japanese_400",
        "--benchmark-only",
        "--benchmark-warmup=off",
        "-v",
    ]

    result = subprocess.run(cmd, check=False, capture_output=True, text=True)

    if result.returncode == 0:
        print("✓ Benchmarks ran successfully!")
        print("\nOutput:")
        print(result.stdout)
        return True
    else:
        print("✗ Benchmarks failed!")
        print("\nError:")
        print(result.stderr)
        return False


def main():
    """Run all local tests."""
    print("Local Benchmark Test Suite")
    print("=" * 50)

    success = True

    # Test imports
    if not test_imports():
        print("\n⚠️  Import test failed. Make sure to install dependencies:")
        print("    uv pip install -e '.[benchmark]'")
        success = False

    # Test basic functionality
    if success and not test_basic_functionality():
        print("\n⚠️  Basic functionality test failed.")
        success = False

    # Run benchmarks
    if success and not run_benchmarks_subset():
        print("\n⚠️  Benchmark test failed.")
        success = False

    if success:
        print("\n✅ All tests passed! Benchmarks are ready to use.")
    else:
        print("\n❌ Some tests failed. Please fix the issues before proceeding.")
        sys.exit(1)


if __name__ == "__main__":
    main()
