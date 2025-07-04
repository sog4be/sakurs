"""Tests for the results formatter module."""

import json

import pytest

from benchmarks.cli.scripts.results_formatter import ExperimentResult, ExperimentResults


class TestExperimentResults:
    """Test cases for ExperimentResults class."""

    def test_add_result(self):
        """Test adding results."""
        results = ExperimentResults()
        
        result1 = ExperimentResult(
            tool="sakurs",
            language="EN",
            threads=1,
            throughput_mbps=100.5,
        )
        results.add_result(result1)
        
        assert len(results.results) == 1
        assert results.results[0].tool == "sakurs"

    def test_to_json(self):
        """Test JSON export."""
        results = ExperimentResults()
        
        result1 = ExperimentResult(
            tool="sakurs",
            language="EN",
            threads=1,
            throughput_mbps=100.5,
            memory_peak_mib=50.2,
        )
        results.add_result(result1)
        
        json_output = results.to_json()
        assert json_output["count"] == 1
        assert len(json_output["experiments"]) == 1
        assert json_output["experiments"][0]["tool"] == "sakurs"
        assert json_output["experiments"][0]["throughput_mbps"] == 100.5

    def test_throughput_table_markdown(self):
        """Test throughput table generation in markdown."""
        results = ExperimentResults()
        
        # Add results for different threads
        for threads in [1, 2, 4, 8]:
            result = ExperimentResult(
                tool="Δ-Stack (Ours)",
                language="JA",
                threads=threads,
                throughput_mbps=100.0 * threads,
            )
            results.add_result(result)
        
        # Add baseline result
        baseline = ExperimentResult(
            tool="ja_sentence_segmenter",
            language="JA",
            threads=1,
            throughput_mbps=50.0,
        )
        results.add_result(baseline)
        
        table = results.to_markdown_table("throughput")
        
        # Check table structure
        lines = table.split("\n")
        assert "| Lang | Tool | 1 T | 2 T | 4 T | 8 T |" in lines[0]
        assert "| --- | --- | --- | --- | --- | --- |" in lines[1]
        
        # Check data presence
        assert "| JA | Δ-Stack (Ours) |" in table
        assert "100.0" in table  # 1 thread
        assert "800.0" in table  # 8 threads
        assert "| JA | ja_sentence_segmenter | 50.0 | — | — | — |" in table

    def test_memory_table_markdown(self):
        """Test memory table generation in markdown."""
        results = ExperimentResults()
        
        # Add results for 1 and 8 threads
        for threads in [1, 8]:
            result = ExperimentResult(
                tool="NLTK Punkt",
                language="EN",
                threads=threads,
                memory_peak_mib=30.0 + threads * 5,
            )
            results.add_result(result)
        
        table = results.to_markdown_table("memory")
        
        # Check table structure
        lines = table.split("\n")
        assert "| Lang | Tool | 1 T | 8 T |" in lines[0]
        assert "| --- | --- | --- | --- |" in lines[1]
        
        # Check data
        assert "| EN | NLTK Punkt |" in table
        assert "35.0" in table  # 1 thread
        assert "70.0" in table  # 8 threads

    def test_accuracy_table_markdown(self):
        """Test accuracy table generation in markdown."""
        results = ExperimentResults()
        
        result = ExperimentResult(
            tool="Δ-Stack (Ours)",
            language="EN",
            precision=0.952,
            recall=0.948,
            f1_score=0.950,
            pk_score=0.023,
            windowdiff_score=0.031,
        )
        results.add_result(result)
        
        table = results.to_markdown_table("accuracy")
        
        # Check table structure
        assert "| Lang | Tool | Precision | Recall | F1 | **Pk** | **WindowDiff** |" in table
        assert "| --- | --- | --- | --- | --- | --- | --- |" in table
        
        # Check data
        assert "| EN | Δ-Stack (Ours) |" in table
        assert "0.952" in table
        assert "0.948" in table
        assert "0.950" in table
        assert "0.023" in table
        assert "0.031" in table

    def test_latex_table_generation(self):
        """Test LaTeX table generation."""
        results = ExperimentResults()
        
        result = ExperimentResult(
            tool="sakurs",
            language="JA",
            threads=1,
            throughput_mbps=150.5,
        )
        results.add_result(result)
        
        latex_table = results.to_latex_table("throughput")
        
        # Check LaTeX structure
        assert r"\begin{tabular}" in latex_table
        assert r"\end{tabular}" in latex_table
        assert r"\toprule" in latex_table
        assert r"\midrule" in latex_table
        assert r"\bottomrule" in latex_table
        assert "150.5" in latex_table

    def test_invalid_metric(self):
        """Test invalid metric name."""
        results = ExperimentResults()
        
        with pytest.raises(ValueError):
            results.to_markdown_table("invalid_metric")
        
        with pytest.raises(ValueError):
            results.to_latex_table("invalid_metric")