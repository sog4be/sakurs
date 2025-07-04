#!/usr/bin/env python3
"""Evaluate segmentation accuracy against ground truth."""

import sys
import json
import logging
from pathlib import Path
from typing import List, Dict, Tuple
import click
import numpy as np
from sklearn.metrics import precision_recall_fscore_support

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


def read_sentences(file_path: Path) -> List[str]:
    """Read sentences from file (one per line)."""
    with open(file_path, 'r', encoding='utf-8') as f:
        return [line.strip() for line in f if line.strip()]


def align_sentences(predicted: List[str], reference: List[str]) -> Tuple[List[int], List[int]]:
    """Align predicted and reference sentences to find boundaries.
    
    Returns:
        Tuple of (predicted_boundaries, reference_boundaries) as character positions
    """
    # Build text and track boundaries
    pred_text = ""
    pred_boundaries = []
    for sent in predicted:
        pred_text += sent
        pred_boundaries.append(len(pred_text))
        pred_text += " "  # Add space between sentences
        
    ref_text = ""
    ref_boundaries = []
    for sent in reference:
        ref_text += sent
        ref_boundaries.append(len(ref_text))
        ref_text += " "
        
    return pred_boundaries[:-1], ref_boundaries[:-1]  # Exclude last boundary


def calculate_metrics(predicted: List[str], reference: List[str]) -> Dict[str, float]:
    """Calculate segmentation metrics.
    
    Args:
        predicted: List of predicted sentences
        reference: List of reference sentences
        
    Returns:
        Dictionary with precision, recall, f1, pk, and window_diff
    """
    # Basic boundary-based metrics
    pred_boundaries, ref_boundaries = align_sentences(predicted, reference)
    
    # Convert to sets for comparison
    pred_set = set(pred_boundaries)
    ref_set = set(ref_boundaries)
    
    # Calculate precision, recall, F1
    true_positive = len(pred_set & ref_set)
    false_positive = len(pred_set - ref_set)
    false_negative = len(ref_set - pred_set)
    
    precision = true_positive / (true_positive + false_positive) if (true_positive + false_positive) > 0 else 0
    recall = true_positive / (true_positive + false_negative) if (true_positive + false_negative) > 0 else 0
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0
    
    # Calculate Pk (probability of error)
    # Simplified version - would need full implementation for paper
    pk = calculate_pk(pred_boundaries, ref_boundaries)
    
    # Calculate WindowDiff
    window_diff = calculate_window_diff(pred_boundaries, ref_boundaries)
    
    return {
        "precision": precision,
        "recall": recall,
        "f1": f1,
        "pk": pk,
        "window_diff": window_diff,
        "predicted_sentences": len(predicted),
        "reference_sentences": len(reference)
    }


def calculate_pk(pred_boundaries: List[int], ref_boundaries: List[int], k: int = None) -> float:
    """Calculate Pk metric (Beeferman et al., 1999).
    
    Simplified implementation - production version would need proper windowing.
    """
    # Placeholder - return error rate for now
    total = len(ref_boundaries)
    errors = len(set(pred_boundaries) ^ set(ref_boundaries))
    return errors / total if total > 0 else 0


def calculate_window_diff(pred_boundaries: List[int], ref_boundaries: List[int], k: int = None) -> float:
    """Calculate WindowDiff metric (Pevzner & Hearst, 2002).
    
    Simplified implementation - production version would need proper windowing.
    """
    # Placeholder - return normalized error count
    total = len(ref_boundaries)
    errors = len(set(pred_boundaries) ^ set(ref_boundaries))
    return errors / total if total > 0 else 0


@click.command()
@click.option('--predicted', '-p', required=True, type=click.Path(exists=True), 
              help='Path to predicted sentences file')
@click.option('--reference', '-r', required=True, type=click.Path(exists=True),
              help='Path to reference sentences file')
@click.option('--output', '-o', type=click.Path(), help='Output JSON file for results')
@click.option('--format', 'output_format', type=click.Choice(['json', 'text']), 
              default='text', help='Output format')
def main(predicted, reference, output, output_format):
    """Evaluate segmentation accuracy."""
    # Read files
    pred_sentences = read_sentences(Path(predicted))
    ref_sentences = read_sentences(Path(reference))
    
    logger.info(f"Predicted sentences: {len(pred_sentences)}")
    logger.info(f"Reference sentences: {len(ref_sentences)}")
    
    # Calculate metrics
    metrics = calculate_metrics(pred_sentences, ref_sentences)
    
    # Output results
    if output_format == 'json' or output:
        result = {
            "predicted_file": str(predicted),
            "reference_file": str(reference),
            "metrics": metrics
        }
        
        if output:
            with open(output, 'w') as f:
                json.dump(result, f, indent=2)
            logger.info(f"Results saved to {output}")
        else:
            print(json.dumps(result, indent=2))
    else:
        # Text format
        print("\nSegmentation Accuracy Results")
        print("=" * 40)
        print(f"Predicted sentences: {metrics['predicted_sentences']}")
        print(f"Reference sentences: {metrics['reference_sentences']}")
        print(f"Precision: {metrics['precision']:.4f}")
        print(f"Recall: {metrics['recall']:.4f}")
        print(f"F1 Score: {metrics['f1']:.4f}")
        print(f"Pk: {metrics['pk']:.4f}")
        print(f"WindowDiff: {metrics['window_diff']:.4f}")


if __name__ == '__main__':
    main()