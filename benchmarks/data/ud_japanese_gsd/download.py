#!/usr/bin/env python3
"""Download and process UD Japanese GSD data for sakurs benchmarks."""

import json
import sys
import tempfile
from pathlib import Path

import click
import requests
from tqdm import tqdm

# Add parent directory to path to import schema
sys.path.insert(0, str(Path(__file__).parent.parent))
from schema import validate_corpus_data


def download_from_github() -> Path:
    """Download UD Japanese-GSD directly from GitHub.

    Returns:
        Path to downloaded directory
    """
    click.echo("📥 Downloading UD Japanese-GSD from GitHub...")

    # Create temporary directory
    temp_dir = Path(tempfile.mkdtemp())

    try:
        # Clone the repository
        import subprocess

        repo_url = "https://github.com/UniversalDependencies/UD_Japanese-GSD.git"
        gsd_dir = temp_dir / "UD_Japanese-GSD"

        click.echo(f"🔗 Cloning from {repo_url}...")
        result = subprocess.run(
            ["git", "clone", "--depth", "1", repo_url, str(gsd_dir)], capture_output=True, text=True
        )

        if result.returncode != 0:
            raise RuntimeError(f"Git clone failed: {result.stderr}")

        click.echo(f"✅ Downloaded UD Japanese-GSD to {gsd_dir}")
        return gsd_dir

    except Exception as e:
        # Clean up on error
        import shutil

        shutil.rmtree(temp_dir, ignore_errors=True)
        raise e


def download_ud_release(version: str = "2.16") -> Path:
    """Download UD release archive and extract Japanese GSD treebank.

    Args:
        version: UD version to download (default: "2.16")

    Returns:
        Path to extracted UD_Japanese-GSD directory

    Note:
        If download fails, you can also clone from GitHub:
        git clone https://github.com/UniversalDependencies/UD_Japanese-GSD.git
    """
    # UD 2.16 download URL
    if version == "2.16":
        url = "https://lindat.mff.cuni.cz/repository/xmlui/bitstream/handle/11234/1-5901/ud-treebanks-v2.16.tgz"
    else:
        raise ValueError(f"Unsupported UD version: {version}")

    click.echo(f"📥 Downloading UD {version} release...")

    # Create temporary directory for download
    temp_dir = Path(tempfile.mkdtemp())
    archive_path = temp_dir / f"ud-treebanks-v{version}.tgz"

    try:
        # Download with requests and progress bar with retry logic
        max_retries = 3
        for attempt in range(max_retries):
            try:
                click.echo(f"📥 Download attempt {attempt + 1}/{max_retries}")

                # Use session with proper settings for large file downloads
                session = requests.Session()
                session.headers.update({"User-Agent": "sakurs-benchmarks/1.0 (research-tool)"})

                # Much longer timeout for large files, allow redirects
                response = session.get(url, stream=True, timeout=600, allow_redirects=True)
                response.raise_for_status()

                total_size = int(response.headers.get("content-length", 0))

                with (
                    open(archive_path, "wb") as f,
                    tqdm(
                        desc="Downloading",
                        total=total_size,
                        unit="B",
                        unit_scale=True,
                        unit_divisor=1024,
                    ) as pbar,
                ):
                    for chunk in response.iter_content(
                        chunk_size=1024 * 1024
                    ):  # 1MB chunks for better performance
                        if chunk:
                            f.write(chunk)
                            pbar.update(len(chunk))

                # Verify download completed
                if archive_path.stat().st_size == total_size:
                    click.echo("✅ Download completed successfully")
                    break
                else:
                    click.echo("⚠️  Download incomplete, retrying...")
                    archive_path.unlink(missing_ok=True)

            except (OSError, requests.exceptions.RequestException) as e:
                click.echo(f"❌ Download attempt {attempt + 1} failed: {e}")
                archive_path.unlink(missing_ok=True)
                if attempt == max_retries - 1:
                    raise e
                click.echo("⏳ Waiting 30 seconds before retry...")
                import time

                time.sleep(30)

        click.echo(f"✅ Downloaded {archive_path.stat().st_size / 1024 / 1024:.1f} MB")

        # Extract archive
        click.echo("📦 Extracting archive...")
        import tarfile

        with tarfile.open(archive_path, "r:gz") as tar:
            # Extract only Japanese GSD treebank
            members = [m for m in tar.getmembers() if "UD_Japanese-GSD" in m.name]
            tar.extractall(temp_dir, members=members)

        # Find extracted Japanese GSD directory
        gsd_dir = temp_dir / "UD_Japanese-GSD"
        if not gsd_dir.exists():
            # Try with version suffix
            for item in temp_dir.iterdir():
                if item.is_dir() and "Japanese-GSD" in item.name:
                    gsd_dir = item
                    break

        if not gsd_dir.exists():
            raise FileNotFoundError("UD_Japanese-GSD directory not found in archive")

        click.echo(f"✅ Extracted UD Japanese GSD to {gsd_dir}")
        return gsd_dir

    except Exception as e:
        # Clean up on error
        import shutil

        shutil.rmtree(temp_dir, ignore_errors=True)
        raise e


def parse_conllu_file(file_path: Path) -> tuple[str, list[int]]:
    """Parse CoNLL-U file and extract sentence boundaries.

    Args:
        file_path: Path to .conllu file

    Returns:
        Tuple of (full_text, boundary_positions)
    """
    sentences = []
    current_sentence = []

    click.echo(f"📖 Parsing {file_path.name}...")

    with open(file_path, encoding="utf-8") as f:
        lines = f.readlines()

    for line in tqdm(lines, desc="Processing lines"):
        line = line.strip()

        if line == "":  # Empty line = sentence boundary
            if current_sentence:
                # For Japanese, we don't add spaces between words
                sentences.append("".join(current_sentence))
                current_sentence = []
        elif not line.startswith("#"):  # Skip comment lines
            fields = line.split("\t")
            if len(fields) >= 2:
                # Skip multi-word tokens (e.g., "1-2" ranges)
                if "-" not in fields[0] and "." not in fields[0]:
                    word_form = fields[1]  # FORM field
                    if word_form != "_":  # Skip empty forms
                        current_sentence.append(word_form)

    # Add final sentence if exists
    if current_sentence:
        sentences.append("".join(current_sentence))

    # Build full text and calculate boundary positions
    full_text = ""
    boundaries = []

    for i, sentence in enumerate(sentences):
        # For Japanese, add a space between sentences (not between words)
        if i > 0:
            full_text += " "
        full_text += sentence
        # Boundary at the end of sentence
        boundaries.append(len(full_text))

    # Remove final boundary (end of text)
    if boundaries:
        boundaries.pop()

    click.echo(f"✅ Parsed {len(sentences)} sentences, {len(full_text)} characters")
    return full_text.strip(), boundaries


def process_ud_japanese_gsd(gsd_dir: Path) -> tuple[str, list[int]]:
    """Process UD Japanese GSD files and combine train/dev/test splits.

    Args:
        gsd_dir: Path to UD_Japanese-GSD directory

    Returns:
        Tuple of (combined_text, combined_boundaries)
    """
    splits = ["train", "dev", "test"]
    all_texts = []
    all_boundaries = []
    current_offset = 0

    for split in splits:
        conllu_file = gsd_dir / f"ja_gsd-ud-{split}.conllu"
        if not conllu_file.exists():
            click.echo(f"⚠️  {split} file not found: {conllu_file}")
            continue

        text, boundaries = parse_conllu_file(conllu_file)

        # Adjust boundary positions for combined text
        adjusted_boundaries = [b + current_offset for b in boundaries]

        # Add split separator if not first
        if all_texts:
            all_texts.append(" ")
            current_offset += 1

        all_texts.append(text)
        all_boundaries.extend(adjusted_boundaries)
        current_offset += len(text)

        click.echo(f"✅ Processed {split}: {len(boundaries)} sentences")

    combined_text = "".join(all_texts)
    click.echo(f"✅ Combined {len(all_boundaries)} sentences, {len(combined_text)} characters")

    return combined_text, all_boundaries


def save_corpus_data(text: str, boundaries: list[int], output_path: Path) -> None:
    """Save corpus data in sakurs-compatible JSON format."""
    # Calculate metadata - for Japanese, we count characters as "words"
    char_count = len(text)
    # Count actual words by splitting on spaces (between sentences)
    word_count = sum(len(sentence) for sentence in text.split())

    metadata = {
        "source": "UD Japanese GSD r2.16",
        "version": "2.16",
        "sentences": len(boundaries),
        "characters": char_count,
        "words": word_count,
        "genres": ["wiki", "news", "blog"],
        "splits": ["train", "dev", "test"],
        "format": "CoNLL-U",
        "license": "CC BY-SA-NC 4.0",
        "note": "Full text available for accurate benchmarking",
    }

    # Create data structure
    corpus_data = {
        "name": "ud_japanese_gsd_full",
        "text": text,
        "boundaries": boundaries,
        "metadata": metadata,
    }

    # Validate before saving
    try:
        validate_corpus_data(corpus_data)
        click.echo("✅ Data validation passed")
    except ValueError as e:
        click.echo(f"❌ Data validation failed: {e}", err=True)
        sys.exit(1)

    # Save to JSON
    click.echo(f"💾 Saving to {output_path}...")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(corpus_data, f, ensure_ascii=False, indent=2)

    # Print summary
    click.echo("\n✨ Processing complete!")
    click.echo(f"   Source: {metadata['source']}")
    click.echo(f"   Sentences: {metadata['sentences']:,}")
    click.echo(f"   Characters: {metadata['characters']:,}")
    click.echo(f"   Words: {metadata['words']:,}")
    click.echo(f"   Genres: {', '.join(metadata['genres'])}")
    click.echo(f"   Output: {output_path}")


@click.command()
@click.option(
    "--output",
    "-o",
    type=click.Path(path_type=Path),
    default="cache/ud_japanese_gsd.json",
    help="Output file path",
)
@click.option(
    "--version",
    "-v",
    default="2.16",
    help="UD version to download",
)
@click.option(
    "--force",
    "-f",
    is_flag=True,
    help="Force re-download and re-process even if cache exists",
)
def main(output: Path, version: str, force: bool) -> None:
    """Download and process UD Japanese GSD for sakurs benchmarks."""
    # Ensure output directory exists
    output.parent.mkdir(parents=True, exist_ok=True)

    # Check if already exists
    if output.exists() and not force:
        click.echo(f"✓ Cached data already exists at {output}")
        click.echo("  Use --force to re-process")
        return

    try:
        # Try to download from UD release archive first
        try:
            gsd_dir = download_ud_release(version)
        except Exception as e:
            click.echo(f"⚠️  UD archive download failed: {e}")
            click.echo("📥 Trying GitHub download as fallback...")
            gsd_dir = download_from_github()

        # Process CoNLL-U files
        text, boundaries = process_ud_japanese_gsd(gsd_dir)

        # Save processed data
        save_corpus_data(text, boundaries, output)

        # Clean up temporary files
        import shutil

        shutil.rmtree(gsd_dir.parent, ignore_errors=True)

    except Exception as e:
        click.echo(f"❌ Error: {e}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    main()

