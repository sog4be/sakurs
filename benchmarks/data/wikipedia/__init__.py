"""Wikipedia corpus loader using Hugging Face datasets."""

from .loader import WikipediaLoader


def create_loader(language: str, size_mb: int = 500, date: str = None):
    """Create a Wikipedia loader for the specified language.

    Args:
        language: Language code (e.g., 'en', 'ja')
        size_mb: Target sample size in MB
        date: Wikipedia dump date (default: 20231101)

    Returns:
        WikipediaLoader instance
    """
    return WikipediaLoader(language=language, size_mb=size_mb, date=date)


def is_available(language: str, size_mb: int = 500) -> bool:
    """Check if Wikipedia data is available for the language.

    Args:
        language: Language code
        size_mb: Sample size in MB

    Returns:
        True if sample exists
    """
    loader = WikipediaLoader(language=language, size_mb=size_mb)
    return loader.is_cached()


__all__ = ["WikipediaLoader", "create_loader", "is_available"]
