"""Tests for the metrics module."""

import pytest

from benchmarks.cli.scripts.metrics import BenchmarkMetrics, MetricsMeasurer


class TestMetricsMeasurer:
    """Test cases for MetricsMeasurer class."""

    def test_measure_throughput(self):
        """Test throughput calculation."""
        measurer = MetricsMeasurer()
        
        # Test normal case
        throughput = measurer.measure_throughput(duration=10.0, data_size_mb=100.0)
        assert throughput == 10.0
        
        # Test with fractional values
        throughput = measurer.measure_throughput(duration=2.5, data_size_mb=7.5)
        assert throughput == 3.0
        
        # Test invalid duration
        with pytest.raises(ValueError):
            measurer.measure_throughput(duration=0.0, data_size_mb=100.0)
        
        with pytest.raises(ValueError):
            measurer.measure_throughput(duration=-1.0, data_size_mb=100.0)

    def test_calculate_precision_recall_f1(self):
        """Test precision, recall, and F1 calculation."""
        measurer = MetricsMeasurer()
        
        # Test perfect match
        predicted = [10, 20, 30, 40, 50]
        gold = [10, 20, 30, 40, 50]
        result = measurer.calculate_precision_recall_f1(predicted, gold)
        assert result["precision"] == 1.0
        assert result["recall"] == 1.0
        assert result["f1"] == 1.0
        
        # Test partial match
        predicted = [10, 20, 30, 60]  # 3 correct, 1 false positive
        gold = [10, 20, 30, 40, 50]  # 3 correct, 2 false negatives
        result = measurer.calculate_precision_recall_f1(predicted, gold)
        assert result["precision"] == 0.75  # 3/4
        assert result["recall"] == 0.6  # 3/5
        assert abs(result["f1"] - 0.6666666) < 0.0001
        
        # Test no match
        predicted = [60, 70, 80]
        gold = [10, 20, 30]
        result = measurer.calculate_precision_recall_f1(predicted, gold)
        assert result["precision"] == 0.0
        assert result["recall"] == 0.0
        assert result["f1"] == 0.0
        
        # Test empty predictions
        predicted = []
        gold = [10, 20, 30]
        result = measurer.calculate_precision_recall_f1(predicted, gold)
        assert result["precision"] == 0.0
        assert result["recall"] == 0.0
        assert result["f1"] == 0.0

    def test_calculate_pk_windowdiff(self):
        """Test Pk and WindowDiff calculation."""
        measurer = MetricsMeasurer()
        
        # Test perfect match
        reference = "0001000100010"
        hypothesis = "0001000100010"
        result = measurer.calculate_pk_windowdiff(reference, hypothesis)
        assert result["pk"] == 0.0
        assert result["windowdiff"] == 0.0
        
        # Test with errors
        reference = "00010001000100"
        hypothesis = "00100010001000"  # Shifted boundaries
        result = measurer.calculate_pk_windowdiff(reference, hypothesis, k=3)
        assert result["pk"] > 0.0
        assert result["windowdiff"] > 0.0
        
        # Test mismatched lengths
        reference = "0001000100"
        hypothesis = "000100"
        with pytest.raises(ValueError):
            measurer.calculate_pk_windowdiff(reference, hypothesis)
        
        # Test automatic k calculation
        reference = "000100010001"  # 4 segments
        hypothesis = "000100010001"
        result = measurer.calculate_pk_windowdiff(reference, hypothesis)
        # k should be len(reference) / (2 * num_segments) = 12 / 8 = 1.5 -> 1
        assert result["pk"] == 0.0

    def test_measure_with_timer(self):
        """Test timer measurement."""
        measurer = MetricsMeasurer()
        
        def test_func(x, y):
            """Simple test function."""
            import time
            time.sleep(0.01)  # Small delay
            return x + y
        
        duration, result = measurer.measure_with_timer(test_func, 2, 3)
        assert result == 5
        assert duration >= 0.01  # Should take at least 10ms
        assert duration < 0.1  # But not more than 100ms