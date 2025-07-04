#!/usr/bin/env python3
"""Statistical analysis utilities for benchmark results.

Provides confidence intervals, significance testing, and effect sizes
for academic publication requirements.
"""

import json
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Union
import logging
from dataclasses import dataclass
from scipy import stats
import warnings

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


@dataclass
class ConfidenceInterval:
    """Confidence interval with point estimate."""
    estimate: float
    lower: float
    upper: float
    confidence_level: float = 0.95
    
    def __str__(self) -> str:
        return f"{self.estimate:.4f} [{self.lower:.4f}, {self.upper:.4f}]"
    
    def to_dict(self) -> Dict[str, float]:
        return {
            "estimate": self.estimate,
            "ci_lower": self.lower,
            "ci_upper": self.upper,
            "confidence_level": self.confidence_level
        }


def bootstrap_confidence_interval(
    data: Union[List[float], np.ndarray],
    statistic: callable = np.mean,
    n_bootstrap: int = 10000,
    confidence_level: float = 0.95,
    random_state: Optional[int] = 42
) -> ConfidenceInterval:
    """Calculate bootstrap confidence interval for a statistic.
    
    Args:
        data: Sample data
        statistic: Function to calculate statistic (default: mean)
        n_bootstrap: Number of bootstrap samples
        confidence_level: Confidence level (default: 0.95)
        random_state: Random seed for reproducibility
    
    Returns:
        ConfidenceInterval object
    """
    if len(data) == 0:
        return ConfidenceInterval(0.0, 0.0, 0.0, confidence_level)
    
    data = np.asarray(data)
    rng = np.random.RandomState(random_state)
    
    # Calculate bootstrap samples
    bootstrap_stats = []
    for _ in range(n_bootstrap):
        sample = rng.choice(data, size=len(data), replace=True)
        bootstrap_stats.append(statistic(sample))
    
    # Calculate percentiles
    alpha = 1 - confidence_level
    lower_percentile = (alpha / 2) * 100
    upper_percentile = (1 - alpha / 2) * 100
    
    lower = np.percentile(bootstrap_stats, lower_percentile)
    upper = np.percentile(bootstrap_stats, upper_percentile)
    estimate = statistic(data)
    
    return ConfidenceInterval(estimate, lower, upper, confidence_level)


def proportion_confidence_interval(
    successes: int,
    trials: int,
    confidence_level: float = 0.95
) -> ConfidenceInterval:
    """Calculate confidence interval for a proportion using Clopper-Pearson method.
    
    This is the exact method recommended for accuracy metrics in NLP.
    
    Args:
        successes: Number of successes
        trials: Total number of trials
        confidence_level: Confidence level (default: 0.95)
    
    Returns:
        ConfidenceInterval object
    """
    if trials == 0:
        return ConfidenceInterval(0.0, 0.0, 0.0, confidence_level)
    
    proportion = successes / trials
    alpha = 1 - confidence_level
    
    # Clopper-Pearson interval
    if successes == 0:
        lower = 0.0
        upper = 1 - (alpha / 2) ** (1 / trials)
    elif successes == trials:
        lower = (alpha / 2) ** (1 / trials)
        upper = 1.0
    else:
        lower = stats.beta.ppf(alpha / 2, successes, trials - successes + 1)
        upper = stats.beta.ppf(1 - alpha / 2, successes + 1, trials - successes)
    
    return ConfidenceInterval(proportion, lower, upper, confidence_level)


def add_confidence_intervals_to_metrics(
    metrics: Dict[str, float],
    n_samples: Optional[int] = None,
    timing_data: Optional[List[float]] = None
) -> Dict[str, Union[float, Dict[str, float]]]:
    """Add confidence intervals to evaluation metrics.
    
    Args:
        metrics: Dictionary with metrics (precision, recall, f1, pk, window_diff)
        n_samples: Number of samples for proportion-based metrics
        timing_data: Raw timing data for performance metrics
    
    Returns:
        Enhanced metrics dictionary with confidence intervals
    """
    enhanced_metrics = {}
    
    # For accuracy metrics (proportions)
    proportion_metrics = ['precision', 'recall', 'f1']
    for metric_name in proportion_metrics:
        if metric_name in metrics:
            value = metrics[metric_name]
            if n_samples:
                # Use exact method for proportions
                successes = int(value * n_samples)
                ci = proportion_confidence_interval(successes, n_samples)
            else:
                # Fallback: assume large sample
                se = np.sqrt(value * (1 - value) / 1000)  # Assume n=1000
                ci = ConfidenceInterval(
                    value,
                    max(0, value - 1.96 * se),
                    min(1, value + 1.96 * se)
                )
            enhanced_metrics[metric_name] = ci.to_dict()
    
    # For error metrics (Pk, WindowDiff)
    error_metrics = ['pk', 'window_diff']
    for metric_name in error_metrics:
        if metric_name in metrics:
            value = metrics[metric_name]
            # These are already proportions (error rates)
            if n_samples:
                errors = int(value * n_samples)
                ci = proportion_confidence_interval(errors, n_samples)
            else:
                se = np.sqrt(value * (1 - value) / 1000)
                ci = ConfidenceInterval(
                    value,
                    max(0, value - 1.96 * se),
                    min(1, value + 1.96 * se)
                )
            enhanced_metrics[metric_name] = ci.to_dict()
    
    # For timing metrics
    if timing_data and len(timing_data) > 0:
        # Mean time with bootstrap CI
        time_ci = bootstrap_confidence_interval(timing_data, np.mean)
        enhanced_metrics['mean_time'] = time_ci.to_dict()
        
        # Throughput if file size is known
        if 'file_size_mb' in metrics and metrics['file_size_mb'] > 0:
            throughput_data = [metrics['file_size_mb'] / t for t in timing_data]
            throughput_ci = bootstrap_confidence_interval(throughput_data, np.mean)
            enhanced_metrics['throughput_mb_s'] = throughput_ci.to_dict()
    
    # Keep other metrics as-is
    for key, value in metrics.items():
        if key not in enhanced_metrics and not key.endswith('_ci'):
            enhanced_metrics[key] = value
    
    return enhanced_metrics


def paired_t_test(
    sample1: List[float],
    sample2: List[float],
    alternative: str = 'two-sided'
) -> Tuple[float, float]:
    """Perform paired t-test for performance comparison.
    
    Args:
        sample1: First sample (e.g., system A times)
        sample2: Second sample (e.g., system B times)
        alternative: 'two-sided', 'less', or 'greater'
    
    Returns:
        Tuple of (statistic, p-value)
    """
    if len(sample1) != len(sample2):
        raise ValueError("Samples must have the same length for paired test")
    
    if len(sample1) < 2:
        return 0.0, 1.0
    
    # Use scipy's paired t-test
    statistic, p_value = stats.ttest_rel(sample1, sample2, alternative=alternative)
    
    return float(statistic), float(p_value)


def wilcoxon_signed_rank_test(
    sample1: List[float],
    sample2: List[float],
    alternative: str = 'two-sided'
) -> Tuple[float, float]:
    """Perform Wilcoxon signed-rank test (non-parametric alternative to paired t-test).
    
    Args:
        sample1: First sample
        sample2: Second sample
        alternative: 'two-sided', 'less', or 'greater'
    
    Returns:
        Tuple of (statistic, p-value)
    """
    if len(sample1) != len(sample2):
        raise ValueError("Samples must have the same length for paired test")
    
    if len(sample1) < 2:
        return 0.0, 1.0
    
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        statistic, p_value = stats.wilcoxon(sample1, sample2, alternative=alternative)
    
    return float(statistic), float(p_value)


def calculate_effect_size(
    sample1: List[float],
    sample2: List[float]
) -> Dict[str, float]:
    """Calculate effect size measures (Cohen's d and Hedge's g).
    
    Args:
        sample1: First sample
        sample2: Second sample
    
    Returns:
        Dictionary with effect sizes
    """
    s1 = np.asarray(sample1)
    s2 = np.asarray(sample2)
    
    n1, n2 = len(s1), len(s2)
    
    if n1 == 0 or n2 == 0:
        return {"cohens_d": 0.0, "hedges_g": 0.0}
    
    # Means
    m1, m2 = np.mean(s1), np.mean(s2)
    
    # Pooled standard deviation
    v1, v2 = np.var(s1, ddof=1), np.var(s2, ddof=1)
    pooled_std = np.sqrt(((n1 - 1) * v1 + (n2 - 1) * v2) / (n1 + n2 - 2))
    
    # Cohen's d
    cohens_d = (m1 - m2) / pooled_std if pooled_std > 0 else 0.0
    
    # Hedge's g (corrected for small sample bias)
    # Correction factor
    cf = 1 - 3 / (4 * (n1 + n2) - 9)
    hedges_g = cohens_d * cf
    
    return {
        "cohens_d": float(cohens_d),
        "hedges_g": float(hedges_g)
    }


def format_statistical_result(
    estimate: float,
    ci: Optional[ConfidenceInterval] = None,
    p_value: Optional[float] = None,
    effect_size: Optional[float] = None
) -> str:
    """Format statistical results for academic reporting.
    
    Args:
        estimate: Point estimate
        ci: Confidence interval
        p_value: P-value from significance test
        effect_size: Effect size (e.g., Cohen's d)
    
    Returns:
        Formatted string following APA style
    """
    result = f"{estimate:.3f}"
    
    if ci:
        result += f", 95% CI [{ci.lower:.3f}, {ci.upper:.3f}]"
    
    if p_value is not None:
        if p_value < 0.001:
            result += ", p < .001"
        else:
            result += f", p = {p_value:.3f}"
    
    if effect_size is not None:
        result += f", d = {effect_size:.2f}"
    
    return result


