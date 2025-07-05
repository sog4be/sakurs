# Benchmark Scripts

This directory contains utility scripts for running benchmarks and preparing data.

## Data Preparation

### prepare_data.py

Prepares all benchmark datasets by downloading and converting them to CLI-compatible formats.

```bash
# From benchmarks directory
uv run python cli/scripts/prepare_data.py
```

#### Generated Files Location

The script generates CLI format files in the following locations:

```
benchmarks/data/
├── ud_japanese_gsd/
│   └── cli_format/              # Japanese test data (generated)
│       ├── .gitkeep             # Preserves directory structure
│       ├── gsd_plain.txt        # Plain text (dynamically generated)
│       └── gsd_sentences.txt    # One sentence per line (dynamically generated)
├── ud_english_ewt/
│   └── cli_format/              # English test data (generated)
│       ├── .gitkeep             # Preserves directory structure
│       ├── ewt_plain.txt        # Plain text (dynamically generated)
│       └── ewt_sentences.txt    # One sentence per line (dynamically generated)
└── ...
```

**Note**: The `*.txt` files are NOT stored in git and must be generated locally by running the prepare script.

## Benchmark Scripts

- `evaluate_accuracy.py` - Evaluate segmentation accuracy
- `metrics.py` - Calculate accuracy metrics (Precision, Recall, F1, Pk, WindowDiff)
- `aggregate_results.py` - Aggregate results from multiple runs
- `parallel_runner.py` - Run benchmarks with multiple threads
- `results_formatter.py` - Format results for reports

## Usage Examples

### 1. Prepare Data
```bash
cd benchmarks
uv run python cli/scripts/prepare_data.py
```

### 2. Run Accuracy Evaluation
```bash
uv run python cli/scripts/evaluate_accuracy.py \
    --predicted output.txt \
    --reference data/ud_japanese_gsd/cli_format/gsd_sentences.txt \
    --language japanese
```

### 3. Run Experiments
```bash
cd benchmarks/cli
./run_experiments.sh
```