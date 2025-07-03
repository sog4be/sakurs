"""Type stubs for sakurs Python bindings."""

from typing import overload

__version__: str

class Boundary:
    """Represents a sentence boundary in text."""

    offset: int
    is_sentence_end: bool
    confidence: float

    def __init__(
        self,
        offset: int,
        is_sentence_end: bool,
        confidence: float | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

class ProcessorConfig:
    """Configuration for text processing."""

    chunk_size: int
    overlap_size: int
    max_threads: int | None

    def __init__(
        self,
        chunk_size: int = 8192,
        overlap_size: int = 256,
        max_threads: int | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

class ProcessingMetrics:
    """Metrics from text processing."""

    boundaries_found: int
    chunk_count: int
    thread_count: int
    total_time_us: int
    chunking_time_us: int
    parallel_time_us: int
    merge_time_us: int

    def __repr__(self) -> str: ...
    @property
    def chars_per_second(self) -> float: ...

class ProcessingResult:
    """Result of text processing."""

    boundaries: list[Boundary]
    metrics: ProcessingMetrics

    def sentences(self) -> list[str]: ...
    def __repr__(self) -> str: ...

class Processor:
    """Main processor for sentence boundary detection."""

    def __init__(
        self,
        language: str = "en",
        config: ProcessorConfig | None = None,
    ) -> None: ...
    def process(
        self,
        text: str,
        threads: int | None = None,
    ) -> ProcessingResult: ...
    def sentences(
        self,
        text: str,
        threads: int | None = None,
    ) -> list[str]: ...
    @property
    def language(self) -> str: ...
    @property
    def supports_parallel(self) -> bool: ...
    @property
    def config(self) -> ProcessorConfig: ...
    def __repr__(self) -> str: ...

@overload
def sent_tokenize(
    text: str,
    *,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
) -> list[str]: ...
@overload
def sent_tokenize(
    text: str,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
) -> list[str]: ...
def load(
    language: str,
    config: ProcessorConfig | None = None,
) -> Processor: ...
def segment(
    text: str,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
) -> list[str]: ...
def supported_languages() -> list[str]: ...

# Exception types
class SakursError(Exception): ...
