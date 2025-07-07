"""Type stubs for sakurs Python bindings."""

from typing import overload

__version__: str

class ProcessorConfig:
    """Configuration for text processing."""

    chunk_size: int
    overlap_size: int
    num_threads: int | None

    def __init__(
        self,
        chunk_size: int = 8192,
        overlap_size: int = 256,
        num_threads: int | None = None,
    ) -> None: ...
    def __repr__(self) -> str: ...

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

@overload
def split(
    text: str,
    *,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
) -> list[str]: ...
@overload
def split(
    text: str,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
) -> list[str]: ...
def sent_tokenize(
    text: str,
    language: str = "en",
    config: ProcessorConfig | None = None,
    threads: int | None = None,
) -> list[str]: ...  # Deprecated: use split() instead
def load(
    language: str,
    config: ProcessorConfig | None = None,
) -> Processor: ...
def supported_languages() -> list[str]: ...

# Exception types
class SakursError(Exception): ...
