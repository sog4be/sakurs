"""Version management for Wikipedia datasets."""

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)


class WikipediaVersionManager:
    """Manage versions and metadata for Wikipedia datasets."""

    def __init__(self, cache_dir: Path):
        """Initialize version manager.

        Args:
            cache_dir: Directory for caching dataset metadata
        """
        self.cache_dir = cache_dir
        self.metadata_file = cache_dir / "wikipedia_metadata.json"
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    def save_metadata(
        self,
        language: str,
        size_mb: int,
        date: str,
        article_count: int,
        actual_size_mb: float,
        download_timestamp: datetime,
        additional_info: dict[str, Any] | None = None,
    ):
        """Save dataset metadata.

        Args:
            language: Language code
            size_mb: Target size in MB
            date: Wikipedia dump date
            article_count: Number of articles in the sample
            actual_size_mb: Actual size of the sample in MB
            download_timestamp: When the dataset was downloaded
            additional_info: Any additional metadata
        """
        metadata = self.load_all_metadata()

        key = f"{language}_{size_mb}mb_{date}"

        metadata[key] = {
            "language": language,
            "target_size_mb": size_mb,
            "actual_size_mb": actual_size_mb,
            "wikipedia_dump_date": date,
            "article_count": article_count,
            "download_timestamp": download_timestamp.isoformat(),
            "download_date": download_timestamp.strftime("%Y-%m-%d"),
            "dataset_name": f"wikimedia/wikipedia/{date}.{language}",
            **(additional_info or {}),
        }

        with open(self.metadata_file, "w", encoding="utf-8") as f:
            json.dump(metadata, f, indent=2, ensure_ascii=False)

        logger.info(f"Saved metadata for {key}")

    def load_metadata(self, language: str, size_mb: int, date: str) -> dict[str, Any] | None:
        """Load metadata for a specific dataset.

        Args:
            language: Language code
            size_mb: Target size in MB
            date: Wikipedia dump date

        Returns:
            Metadata dictionary or None if not found
        """
        metadata = self.load_all_metadata()
        key = f"{language}_{size_mb}mb_{date}"
        return metadata.get(key)

    def load_all_metadata(self) -> dict[str, Any]:
        """Load all metadata.

        Returns:
            Dictionary of all dataset metadata
        """
        if not self.metadata_file.exists():
            return {}

        try:
            with open(self.metadata_file, encoding="utf-8") as f:
                return json.load(f)
        except (json.JSONDecodeError, IOError) as e:
            logger.warning(f"Failed to load metadata: {e}")
            return {}

    def get_latest_version(self, language: str, size_mb: int) -> str | None:
        """Get the latest available version for a language and size.

        Args:
            language: Language code
            size_mb: Target size in MB

        Returns:
            Latest dump date or None if no versions found
        """
        metadata = self.load_all_metadata()

        matching_keys = [key for key in metadata if key.startswith(f"{language}_{size_mb}mb_")]

        if not matching_keys:
            return None

        # Extract dates and sort
        dates = [key.split("_")[-1] for key in matching_keys]
        return sorted(dates)[-1]

    def list_available_versions(self) -> list[dict[str, Any]]:
        """List all available dataset versions.

        Returns:
            List of dataset version info
        """
        metadata = self.load_all_metadata()

        versions = []
        for key, info in metadata.items():
            versions.append(
                {
                    "key": key,
                    "language": info["language"],
                    "size_mb": info["target_size_mb"],
                    "date": info["wikipedia_dump_date"],
                    "download_date": info.get("download_date", "Unknown"),
                    "articles": info.get("article_count", 0),
                }
            )

        return sorted(versions, key=lambda x: (x["language"], x["date"]))
