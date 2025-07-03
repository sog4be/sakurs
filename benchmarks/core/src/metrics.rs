//! Accuracy metrics for sentence boundary detection evaluation

use std::collections::HashSet;

/// Accuracy metrics for sentence boundary detection
#[derive(Debug, Clone, PartialEq)]
pub struct AccuracyMetrics {
    /// True positive count
    pub true_positives: usize,
    /// False positive count
    pub false_positives: usize,
    /// False negative count
    pub false_negatives: usize,
    /// Precision: TP / (TP + FP)
    pub precision: f64,
    /// Recall: TP / (TP + FN)
    pub recall: f64,
    /// F1 score: 2 * (precision * recall) / (precision + recall)
    pub f1_score: f64,
    /// Pk score (if calculated)
    pub pk_score: Option<f64>,
    /// WindowDiff score (if calculated)
    pub window_diff: Option<f64>,
}

impl AccuracyMetrics {
    /// Create metrics from raw counts
    pub fn from_counts(
        true_positives: usize,
        false_positives: usize,
        false_negatives: usize,
    ) -> Self {
        let precision = if true_positives + false_positives > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if true_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        Self {
            true_positives,
            false_positives,
            false_negatives,
            precision,
            recall,
            f1_score,
            pk_score: None,
            window_diff: None,
        }
    }

    /// Add Pk score
    pub fn with_pk_score(mut self, pk: f64) -> Self {
        self.pk_score = Some(pk);
        self
    }

    /// Add WindowDiff score
    pub fn with_window_diff(mut self, wd: f64) -> Self {
        self.window_diff = Some(wd);
        self
    }
}

/// Calculate accuracy metrics by comparing predicted and actual boundary positions
pub fn calculate_accuracy_metrics(predicted: &[usize], actual: &[usize]) -> AccuracyMetrics {
    let predicted_set: HashSet<usize> = predicted.iter().cloned().collect();
    let actual_set: HashSet<usize> = actual.iter().cloned().collect();

    let true_positives = predicted_set.intersection(&actual_set).count();
    let false_positives = predicted_set.difference(&actual_set).count();
    let false_negatives = actual_set.difference(&predicted_set).count();

    AccuracyMetrics::from_counts(true_positives, false_positives, false_negatives)
}

/// Calculate Pk score for text segmentation evaluation
///
/// Pk measures the probability that a randomly chosen pair of sentences
/// separated by k sentences are incorrectly classified as being in the
/// same segment or different segments.
pub fn calculate_pk_score(
    predicted: &[usize],
    actual: &[usize],
    text_length: usize,
    k: Option<usize>,
) -> f64 {
    // Convert boundary positions to segment assignments
    let pred_segments = boundaries_to_segments(predicted, text_length);
    let actual_segments = boundaries_to_segments(actual, text_length);

    // Default k to half of average segment length
    let k = k.unwrap_or_else(|| {
        let avg_segment_length = text_length / (actual.len() + 1);
        avg_segment_length / 2
    });

    if text_length <= k {
        return 0.0;
    }

    let mut errors = 0;
    let comparisons = text_length - k;

    for i in 0..comparisons {
        let j = i + k;
        let pred_same = pred_segments[i] == pred_segments[j];
        let actual_same = actual_segments[i] == actual_segments[j];

        if pred_same != actual_same {
            errors += 1;
        }
    }

    errors as f64 / comparisons as f64
}

/// Calculate WindowDiff score for text segmentation evaluation
///
/// WindowDiff is similar to Pk but counts the difference in the number
/// of boundaries within each window, making it more sensitive to
/// near-miss errors.
pub fn calculate_window_diff(
    predicted: &[usize],
    actual: &[usize],
    text_length: usize,
    k: Option<usize>,
) -> f64 {
    // Default k to half of average segment length
    let k = k.unwrap_or_else(|| {
        let avg_segment_length = text_length / (actual.len() + 1);
        avg_segment_length / 2
    });

    if text_length <= k {
        return 0.0;
    }

    let mut errors = 0;
    let comparisons = text_length - k;

    for i in 0..comparisons {
        let window_end = i + k;

        // Count boundaries in window for predicted
        let pred_count = predicted
            .iter()
            .filter(|&&pos| pos > i && pos <= window_end)
            .count();

        // Count boundaries in window for actual
        let actual_count = actual
            .iter()
            .filter(|&&pos| pos > i && pos <= window_end)
            .count();

        if pred_count != actual_count {
            errors += 1;
        }
    }

    errors as f64 / comparisons as f64
}

/// Convert boundary positions to segment assignments for each position
fn boundaries_to_segments(boundaries: &[usize], text_length: usize) -> Vec<usize> {
    let mut segments = vec![0; text_length];
    let mut current_segment = 0;
    let mut boundary_idx = 0;

    for (i, segment) in segments.iter_mut().enumerate() {
        if boundary_idx < boundaries.len() && i >= boundaries[boundary_idx] {
            current_segment += 1;
            boundary_idx += 1;
        }
        *segment = current_segment;
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_accuracy() {
        let predicted = vec![10, 20, 30];
        let actual = vec![10, 20, 30];

        let metrics = calculate_accuracy_metrics(&predicted, &actual);

        assert_eq!(metrics.true_positives, 3);
        assert_eq!(metrics.false_positives, 0);
        assert_eq!(metrics.false_negatives, 0);
        assert_eq!(metrics.precision, 1.0);
        assert_eq!(metrics.recall, 1.0);
        assert_eq!(metrics.f1_score, 1.0);
    }

    #[test]
    fn test_no_predictions() {
        let predicted = vec![];
        let actual = vec![10, 20, 30];

        let metrics = calculate_accuracy_metrics(&predicted, &actual);

        assert_eq!(metrics.true_positives, 0);
        assert_eq!(metrics.false_positives, 0);
        assert_eq!(metrics.false_negatives, 3);
        assert_eq!(metrics.precision, 0.0);
        assert_eq!(metrics.recall, 0.0);
        assert_eq!(metrics.f1_score, 0.0);
    }

    #[test]
    fn test_partial_match() {
        let predicted = vec![10, 25, 35];
        let actual = vec![10, 20, 30];

        let metrics = calculate_accuracy_metrics(&predicted, &actual);

        assert_eq!(metrics.true_positives, 1);
        assert_eq!(metrics.false_positives, 2);
        assert_eq!(metrics.false_negatives, 2);
        assert!(metrics.precision > 0.0 && metrics.precision < 1.0);
        assert!(metrics.recall > 0.0 && metrics.recall < 1.0);
        assert!(metrics.f1_score > 0.0 && metrics.f1_score < 1.0);
    }

    #[test]
    fn test_pk_score_perfect() {
        let predicted = vec![10, 20, 30];
        let actual = vec![10, 20, 30];

        let pk = calculate_pk_score(&predicted, &actual, 40, Some(5));
        assert_eq!(pk, 0.0);
    }

    #[test]
    fn test_window_diff_perfect() {
        let predicted = vec![10, 20, 30];
        let actual = vec![10, 20, 30];

        let wd = calculate_window_diff(&predicted, &actual, 40, Some(5));
        assert_eq!(wd, 0.0);
    }
}
