"""Type stubs for sakurs Python bindings."""

from typing import Any, Literal, overload

__version__: str

# Output classes
class Sentence:
    """Represents a single sentence with detailed metadata."""

    text: str
    start: int
    end: int
    confidence: float
    metadata: dict[str, Any]

    def __init__(
        self,
        text: str,
        start: int,
        end: int,
        confidence: float = 1.0,
        metadata: dict[str, Any] | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __len__(self) -> int: ...

class ProcessingMetadata:
    """Processing statistics and metadata for sentence detection."""

    total_sentences: int
    processing_time_ms: float
    threads_used: int
    chunk_size_used: int
    execution_mode_used: str

    def __init__(
        self,
        total_sentences: int,
        processing_time_ms: float,
        threads_used: int,
        chunk_size_used: int,
        execution_mode_used: str,
    ) -> None: ...
    def __repr__(self) -> str: ...

# Configuration classes
class ProcessorConfig:
    """Configuration for text processing."""

    chunk_size: int
    overlap_size: int
    num_threads: int | None
    max_chunk_size: int

    def __init__(
        self,
        chunk_size: int = 4096,
        overlap_size: int = 128,
        num_threads: int | None = None,
        max_chunk_size: int = 1048576,
    ) -> None: ...
    def __repr__(self) -> str: ...

# Processor class
class Processor:
    """Main processor for sentence boundary detection."""

    def __init__(
        self,
        language: str = "en",
        config: ProcessorConfig | None = None,
    ) -> None: ...
    def split(
        self,
        text: str,
        threads: int | None = None,
    ) -> list[str]: ...
    def sentences(
        self,
        text: str,
        threads: int | None = None,
    ) -> list[str]: ...  # Deprecated: use split() instead
    @property
    def language(self) -> str: ...
    @property
    def supports_parallel(self) -> bool: ...
    def __repr__(self) -> str: ...

# Module-level functions
@overload
def split(
    text: str,
    *,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
    return_details: Literal[False] = False,
) -> list[str]: ...
@overload
def split(
    text: str,
    *,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
    return_details: Literal[True],
) -> list[Sentence]: ...
def load(
    language: str,
    config: ProcessorConfig | None = None,
) -> Processor: ...
def supported_languages() -> list[str]: ...

# Exception hierarchy
class SakursError(Exception):
    """Base exception for all sakurs errors."""

    ...

class InvalidLanguageError(SakursError):
    """Raised when language code is not recognized."""

    ...

class ProcessingError(SakursError):
    """Raised when text processing fails."""

    ...

class ConfigurationError(SakursError):
    """Raised when configuration is invalid."""

    ...

class FileNotFoundError(SakursError):
    """Raised when input file is not found."""

    ...
