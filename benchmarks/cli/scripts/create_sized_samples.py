#!/usr/bin/env python3
"""Create precisely sized data samples from Wikipedia data for scalability benchmarks.

This script generates fixed-size samples (1KiB, 10KiB, 100KiB, 1MiB, 10MiB) from the
Japanese Wikipedia corpus for use in scalability benchmarks. The samples are created
by truncating the source text at exact byte boundaries while preserving UTF-8 encoding.
"""

import logging
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)
logger = logging.getLogger(__name__)


def create_sized_sample(source_file: Path, target_size_bytes: int, output_file: Path) -> int:
    """Create a sample of exactly target_size_bytes from source file.

    Args:
        source_file: Path to source Wikipedia file
        target_size_bytes: Target size in bytes
        output_file: Path to output file

    Returns:
        Actual size of the created sample in bytes
    """
    logger.info(f"Creating {target_size_bytes:,} byte sample from {source_file.name}")

    # Read source data
    with open(source_file, encoding="utf-8") as f:
        content = f.read()

    # If source is smaller than target, duplicate content
    if len(content.encode("utf-8")) < target_size_bytes:
        multiplier = (target_size_bytes // len(content.encode("utf-8"))) + 1
        content = content * multiplier

    # Trim to exact size (preserve UTF-8 boundaries)
    current_size = 0
    result_chars = []

    for char in content:
        char_bytes = char.encode("utf-8")
        if current_size + len(char_bytes) <= target_size_bytes:
            result_chars.append(char)
            current_size += len(char_bytes)
        else:
            break

    result = "".join(result_chars)
    actual_size = len(result.encode("utf-8"))

    # Write output
    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, "w", encoding="utf-8") as f:
        f.write(result)

    logger.info(f"  Created: {actual_size:,} bytes (target: {target_size_bytes:,})")
    return actual_size


def main() -> int:
    """Create sized samples for scalability benchmarks.

    Returns:
        Exit code (0 for success, 1 for error)
    """
    # Define sizes with explicit type annotations
    sizes: list[tuple[int, str]] = [
        (1024, "1KiB"),  # 1 KiB
        (10 * 1024, "10KiB"),  # 10 KiB
        (100 * 1024, "100KiB"),  # 100 KiB
        (1024 * 1024, "1MiB"),  # 1 MiB
        (10 * 1024 * 1024, "10MiB"),  # 10 MiB
    ]

    # Base paths
    base_dir = Path(__file__).parent.parent.parent
    data_dir = base_dir / "data"
    sized_samples_dir = data_dir / "sized_samples"

    # Find Japanese Wikipedia sample
    cache_dir = data_dir / "cache"
    # Look for the 500MB Japanese Wikipedia file
    ja_wiki_file = cache_dir / "wikipedia_ja_500mb_20231101.txt"

    if not ja_wiki_file.exists():
        logger.error(f"Japanese Wikipedia sample not found at {ja_wiki_file}")
        logger.error("Please run 'uv run python benchmarks/cli/scripts/prepare_data.py' first")
        return 1

    logger.info(f"Using source: {ja_wiki_file}")

    # Ensure output directory exists
    sized_samples_dir.mkdir(parents=True, exist_ok=True)

    # Create sized samples
    try:
        created_count = 0
        for size_bytes, size_label in sizes:
            output_file = sized_samples_dir / f"wiki_ja_{size_label}.txt"
            actual_size = create_sized_sample(ja_wiki_file, size_bytes, output_file)
            if actual_size > 0:
                created_count += 1

        logger.info(f"\nSuccessfully created {created_count} samples in: {sized_samples_dir}")
        return 0
    except Exception as e:
        logger.error(f"Failed to create samples: {e}")
        return 1


if __name__ == "__main__":
    exit(main())
