"""Type stubs for sakurs Python bindings."""

from pathlib import Path
from typing import Any, BinaryIO, Literal, Protocol, TextIO, overload

class FileProtocol(Protocol):
    """Protocol for file-like objects with read() method."""
    def read(self, size: int = -1) -> str | bytes: ...

__version__: str

# Exception types
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

# Output types
class Sentence:
    """Sentence with metadata."""

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

class ProcessingMetadata:
    """Processing statistics and information."""

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

# Configuration
class ProcessorConfig:
    """Configuration for text processing."""

    chunk_size: int
    overlap_size: int
    num_threads: int | None
    parallel_threshold: int

    def __init__(
        self,
        chunk_size: int = 262144,  # 256KB
        overlap_size: int = 256,
        num_threads: int | None = None,
        parallel_threshold: int = 1048576,  # 1MB
    ) -> None: ...
    def __repr__(self) -> str: ...

# Language Configuration Classes
class MetadataConfig:
    """Metadata configuration for a language."""

    code: str
    name: str

    def __init__(self, code: str, name: str) -> None: ...
    def __repr__(self) -> str: ...

class TerminatorPattern:
    """Terminator pattern configuration."""

    pattern: str
    name: str

    def __init__(self, pattern: str, name: str) -> None: ...
    def __repr__(self) -> str: ...

class TerminatorConfig:
    """Terminator configuration."""

    chars: list[str]
    patterns: list[TerminatorPattern]

    def __init__(
        self, chars: list[str], patterns: list[TerminatorPattern] | None = None
    ) -> None: ...
    def __repr__(self) -> str: ...

class ContextRule:
    """Context rule for ellipsis handling."""

    condition: str
    boundary: bool

    def __init__(self, condition: str, boundary: bool) -> None: ...
    def __repr__(self) -> str: ...

class ExceptionPattern:
    """Exception pattern for ellipsis handling."""

    regex: str
    boundary: bool

    def __init__(self, regex: str, boundary: bool) -> None: ...
    def __repr__(self) -> str: ...

class EllipsisConfig:
    """Ellipsis configuration."""

    treat_as_boundary: bool
    patterns: list[str]
    context_rules: list[ContextRule]
    exceptions: list[ExceptionPattern]

    def __init__(
        self,
        treat_as_boundary: bool = True,
        patterns: list[str] | None = None,
        context_rules: list[ContextRule] | None = None,
        exceptions: list[ExceptionPattern] | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

class EnclosurePair:
    """Enclosure pair configuration."""

    open: str
    close: str
    symmetric: bool

    def __init__(self, open: str, close: str, symmetric: bool = False) -> None: ...
    def __repr__(self) -> str: ...

class EnclosureConfig:
    """Enclosure configuration."""

    pairs: list[EnclosurePair]

    def __init__(self, pairs: list[EnclosurePair]) -> None: ...
    def __repr__(self) -> str: ...

class FastPattern:
    """Fast pattern for suppression."""

    char: str
    line_start: bool
    before: str | None
    after: str | None

    def __init__(
        self,
        char: str,
        line_start: bool = False,
        before: str | None = None,
        after: str | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

class RegexPattern:
    """Regex pattern for suppression."""

    pattern: str
    description: str | None

    def __init__(self, pattern: str, description: str | None = None) -> None: ...
    def __repr__(self) -> str: ...

class SuppressionConfig:
    """Suppression configuration."""

    fast_patterns: list[FastPattern]
    regex_patterns: list[RegexPattern]

    def __init__(
        self,
        fast_patterns: list[FastPattern] | None = None,
        regex_patterns: list[RegexPattern] | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

class AbbreviationConfig:
    """Abbreviation configuration."""

    categories: dict[str, list[str]]

    def __init__(self, **kwargs: list[str]) -> None: ...
    def __repr__(self) -> str: ...
    def __getitem__(self, key: str) -> list[str]: ...
    def __setitem__(self, key: str, value: list[str]) -> None: ...

class SentenceStarterConfig:
    """Sentence starter configuration."""

    categories: dict[str, list[str]]
    require_following_space: bool
    min_word_length: int

    def __init__(
        self,
        require_following_space: bool = True,
        min_word_length: int = 1,
        **kwargs: list[str],
    ) -> None: ...
    def __repr__(self) -> str: ...

class SentenceIterator:
    """Iterator for streaming sentence processing."""

    def __iter__(self) -> SentenceIterator: ...
    def __next__(self) -> str: ...

class LanguageConfig:
    """Complete language configuration."""

    metadata: MetadataConfig
    terminators: TerminatorConfig
    ellipsis: EllipsisConfig
    enclosures: EnclosureConfig
    suppression: SuppressionConfig
    abbreviations: AbbreviationConfig
    sentence_starters: SentenceStarterConfig | None

    def __init__(
        self,
        metadata: MetadataConfig,
        terminators: TerminatorConfig,
        ellipsis: EllipsisConfig,
        enclosures: EnclosureConfig,
        suppression: SuppressionConfig,
        abbreviations: AbbreviationConfig,
        sentence_starters: SentenceStarterConfig | None = None,
    ) -> None: ...
    @classmethod
    def from_toml(cls, path: Path | str) -> LanguageConfig: ...
    def to_toml(self, path: Path | str) -> None: ...
    def __repr__(self) -> str: ...

class Processor:
    """Main processor for sentence boundary detection."""

    def __init__(
        self,
        *,
        language: str | None = None,
        language_config: LanguageConfig | None = None,
        threads: int | None = None,
        chunk_size: int | None = None,
        execution_mode: Literal["sequential", "parallel", "adaptive"] = "adaptive",
        streaming: bool = False,
        stream_chunk_size: int = 10485760,  # 10MB
    ) -> None: ...
    @overload
    def split(
        self,
        input: str | bytes | Path | TextIO | BinaryIO | FileProtocol,
        *,
        return_details: Literal[False] = False,
        encoding: str = "utf-8",
    ) -> list[str]: ...
    @overload
    def split(
        self,
        input: str | bytes | Path | TextIO | BinaryIO | FileProtocol,
        *,
        return_details: Literal[True],
        encoding: str = "utf-8",
    ) -> list[Sentence]: ...
    def iter_split(
        self,
        input: str | bytes | Path | TextIO | BinaryIO | FileProtocol,
        *,
        encoding: str = "utf-8",
        preserve_whitespace: bool = False,
    ) -> SentenceIterator: ...
    @property
    def language(self) -> str: ...
    @property
    def supports_parallel(self) -> bool: ...
    def __enter__(self) -> Processor: ...
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: object | None,
    ) -> bool: ...
    def __repr__(self) -> str: ...

# Main API functions
@overload
def split(
    input: str | bytes | Path | TextIO | BinaryIO | FileProtocol,
    *,
    language: str | None = None,
    language_config: LanguageConfig | None = None,
    threads: int | None = None,
    chunk_size: int | None = None,
    parallel: bool = False,
    execution_mode: Literal["sequential", "parallel", "adaptive"] = "adaptive",
    return_details: Literal[False] = False,
    preserve_whitespace: bool = False,
    encoding: str = "utf-8",
) -> list[str]: ...
@overload
def split(
    input: str | bytes | Path | TextIO | BinaryIO | FileProtocol,
    *,
    language: str | None = None,
    language_config: LanguageConfig | None = None,
    threads: int | None = None,
    chunk_size: int | None = None,
    parallel: bool = False,
    execution_mode: Literal["sequential", "parallel", "adaptive"] = "adaptive",
    return_details: Literal[True],
    preserve_whitespace: bool = False,
    encoding: str = "utf-8",
) -> list[Sentence]: ...
def load(
    language: str,
    *,
    threads: int | None = None,
    chunk_size: int | None = None,
    execution_mode: Literal["sequential", "parallel", "adaptive"] = "adaptive",
) -> Processor:
    """Load a processor for a specific language."""
    ...

def stream_split(
    input: str | bytes | Path | TextIO | BinaryIO | FileProtocol,
    *,
    language: str | None = None,
    language_config: LanguageConfig | None = None,
    chunk_size_mb: int = 10,
    overlap_size: int = 1024,
    preserve_whitespace: bool = False,
    encoding: str = "utf-8",
) -> SentenceIterator:
    """Stream sentences from large files without loading entire content."""
    ...

def supported_languages() -> list[str]:
    """Get list of supported languages."""
    ...
