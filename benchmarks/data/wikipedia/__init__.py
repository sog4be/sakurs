"""Wikipedia corpus loader for performance benchmarking."""

from pathlib import Path
from .loader import WikipediaLoader

# Convenience functions
def create_loader(language: str, size_mb: int = 500) -> WikipediaLoader:
    """Create a Wikipedia loader for the specified language.
    
    Args:
        language: Language code (e.g., 'en', 'ja')
        size_mb: Target sample size in MB
        
    Returns:
        WikipediaLoader instance
    """
    return WikipediaLoader(language=language, size_mb=size_mb)

def is_available(language: str) -> bool:
    """Check if Wikipedia data is available for a language.
    
    Args:
        language: Language code
        
    Returns:
        True if data is cached
    """
    loader = WikipediaLoader(language=language)
    return loader.is_cached()

__all__ = ['WikipediaLoader', 'create_loader', 'is_available']