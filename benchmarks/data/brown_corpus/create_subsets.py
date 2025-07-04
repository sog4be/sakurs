#!/usr/bin/env python3
"""Create subset JSON files from the full Brown Corpus data."""

import json
from pathlib import Path


def create_subsets():
    """Create subset JSON files for benchmarking."""
    # Load full data
    cache_path = Path(__file__).parent / "cache" / "brown_corpus.json"
    with open(cache_path, encoding="utf-8") as f:
        full_data = json.load(f)

    # Create data directory
    data_dir = Path(__file__).parent / "data"
    data_dir.mkdir(exist_ok=True)

    # Convert to sentences format
    sentences = []
    last_pos = 0

    for boundary in full_data["boundaries"]:
        sent_text = full_data["text"][last_pos : boundary + 1]
        sentences.append({"text": sent_text})
        last_pos = boundary + 1

    # Add the last sentence (after last boundary)
    if last_pos < len(full_data["text"]):
        sentences.append({"text": full_data["text"][last_pos:]})

    # Create full sentences file
    all_data = {"sentences": sentences, "metadata": full_data["metadata"]}

    with open(data_dir / "sentences_all.json", "w", encoding="utf-8") as f:
        json.dump(all_data, f, ensure_ascii=False, indent=2)

    print(f"Created sentences_all.json with {len(sentences)} sentences")

    # Create subsets
    subset_sizes = [100, 1000, 5000, 10000]

    for size in subset_sizes:
        if size <= len(sentences):
            subset_data = {
                "sentences": sentences[:size],
                "metadata": {
                    "source": "Brown Corpus",
                    "subset_size": size,
                    "total_sentences": len(sentences),
                },
            }

            with open(data_dir / f"sentences_{size}.json", "w", encoding="utf-8") as f:
                json.dump(subset_data, f, ensure_ascii=False, indent=2)

            print(f"Created sentences_{size}.json")


if __name__ == "__main__":
    create_subsets()
