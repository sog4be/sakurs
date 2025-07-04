"""Unified results formatting for experiment outputs."""

import json
from dataclasses import asdict, dataclass
from typing import Dict, List, Optional


@dataclass
class ExperimentResult:
    """Container for a single experiment result."""

    tool: str
    language: str
    threads: Optional[int] = None
    throughput_mbps: Optional[float] = None
    memory_peak_mib: Optional[float] = None
    precision: Optional[float] = None
    recall: Optional[float] = None
    f1_score: Optional[float] = None
    pk_score: Optional[float] = None
    windowdiff_score: Optional[float] = None
    dataset: Optional[str] = None
    dataset_size_mb: Optional[float] = None


class ExperimentResults:
    """Collection of experiment results with formatting capabilities."""

    def __init__(self):
        """Initialize empty results collection."""
        self.results: List[ExperimentResult] = []

    def add_result(self, result: ExperimentResult):
        """Add a result to the collection."""
        self.results.append(result)

    def to_json(self) -> dict:
        """Convert results to JSON-serializable dictionary."""
        return {
            "experiments": [asdict(result) for result in self.results],
            "count": len(self.results),
        }

    def to_markdown_table(self, metric: str) -> str:
        """Generate markdown table for a specific metric.

        Args:
            metric: One of 'throughput', 'memory', 'accuracy'

        Returns:
            Markdown-formatted table string
        """
        if metric == "throughput":
            return self._throughput_table_markdown()
        elif metric == "memory":
            return self._memory_table_markdown()
        elif metric == "accuracy":
            return self._accuracy_table_markdown()
        else:
            raise ValueError(f"Unknown metric: {metric}")

    def to_latex_table(self, metric: str) -> str:
        """Generate LaTeX table for a specific metric.

        Args:
            metric: One of 'throughput', 'memory', 'accuracy'

        Returns:
            LaTeX-formatted table string
        """
        if metric == "throughput":
            return self._throughput_table_latex()
        elif metric == "memory":
            return self._memory_table_latex()
        elif metric == "accuracy":
            return self._accuracy_table_latex()
        else:
            raise ValueError(f"Unknown metric: {metric}")

    def _throughput_table_markdown(self) -> str:
        """Generate throughput table in markdown format."""
        # Group results by language and tool
        grouped = self._group_by_language_tool()

        lines = ["| Lang | Tool | 1 T | 2 T | 4 T | 8 T |"]
        lines.append("| --- | --- | --- | --- | --- | --- |")

        for lang in ["JA", "EN"]:
            if lang in grouped:
                for tool, results in grouped[lang].items():
                    thread_results = {r.threads: r.throughput_mbps for r in results}
                    line = f"| {lang} | {tool} | "
                    for threads in [1, 2, 4, 8]:
                        value = thread_results.get(threads)
                        if value is not None:
                            line += f"{value:.1f} | "
                        else:
                            line += "— | "
                    lines.append(line.rstrip())

        return "\n".join(lines)

    def _memory_table_markdown(self) -> str:
        """Generate memory usage table in markdown format."""
        grouped = self._group_by_language_tool()

        lines = ["| Lang | Tool | 1 T | 8 T |"]
        lines.append("| --- | --- | --- | --- |")

        for lang in ["JA", "EN"]:
            if lang in grouped:
                for tool, results in grouped[lang].items():
                    thread_results = {r.threads: r.memory_peak_mib for r in results}
                    line = f"| {lang} | {tool} | "
                    for threads in [1, 8]:
                        value = thread_results.get(threads)
                        if value is not None:
                            line += f"{value:.1f} | "
                        else:
                            line += "— | "
                    lines.append(line.rstrip())

        return "\n".join(lines)

    def _accuracy_table_markdown(self) -> str:
        """Generate accuracy table in markdown format."""
        grouped = self._group_by_language_tool()

        lines = [
            "| Lang | Tool | Precision | Recall | F1 | **Pk** | **WindowDiff** |"
        ]
        lines.append("| --- | --- | --- | --- | --- | --- | --- |")

        for lang in ["JA", "EN"]:
            if lang in grouped:
                for tool, results in grouped[lang].items():
                    # Use first result for accuracy (not thread-dependent)
                    r = results[0]
                    line = f"| {lang} | {tool} | "
                    for value in [
                        r.precision,
                        r.recall,
                        r.f1_score,
                        r.pk_score,
                        r.windowdiff_score,
                    ]:
                        if value is not None:
                            line += f"{value:.3f} | "
                        else:
                            line += "— | "
                    lines.append(line.rstrip())

        return "\n".join(lines)

    def _throughput_table_latex(self) -> str:
        """Generate throughput table in LaTeX format."""
        grouped = self._group_by_language_tool()

        lines = [
            r"\begin{tabular}{llrrrr}",
            r"\toprule",
            r"Lang & Tool & 1 T & 2 T & 4 T & 8 T \\",
            r"\midrule",
        ]

        for lang in ["JA", "EN"]:
            if lang in grouped:
                for tool, results in grouped[lang].items():
                    thread_results = {r.threads: r.throughput_mbps for r in results}
                    line = f"{lang} & {tool} & "
                    for threads in [1, 2, 4, 8]:
                        value = thread_results.get(threads)
                        if value is not None:
                            line += f"{value:.1f} & "
                        else:
                            line += "--- & "
                    lines.append(line.rstrip(" & ") + r" \\")

        lines.extend([r"\bottomrule", r"\end{tabular}"])
        return "\n".join(lines)

    def _memory_table_latex(self) -> str:
        """Generate memory usage table in LaTeX format."""
        grouped = self._group_by_language_tool()

        lines = [
            r"\begin{tabular}{llrr}",
            r"\toprule",
            r"Lang & Tool & 1 T & 8 T \\",
            r"\midrule",
        ]

        for lang in ["JA", "EN"]:
            if lang in grouped:
                for tool, results in grouped[lang].items():
                    thread_results = {r.threads: r.memory_peak_mib for r in results}
                    line = f"{lang} & {tool} & "
                    for threads in [1, 8]:
                        value = thread_results.get(threads)
                        if value is not None:
                            line += f"{value:.1f} & "
                        else:
                            line += "--- & "
                    lines.append(line.rstrip(" & ") + r" \\")

        lines.extend([r"\bottomrule", r"\end{tabular}"])
        return "\n".join(lines)

    def _accuracy_table_latex(self) -> str:
        """Generate accuracy table in LaTeX format."""
        grouped = self._group_by_language_tool()

        lines = [
            r"\begin{tabular}{llrrrrr}",
            r"\toprule",
            r"Lang & Tool & Precision & Recall & F1 & \textbf{Pk} & \textbf{WindowDiff} \\",
            r"\midrule",
        ]

        for lang in ["JA", "EN"]:
            if lang in grouped:
                for tool, results in grouped[lang].items():
                    r = results[0]
                    line = f"{lang} & {tool} & "
                    for value in [
                        r.precision,
                        r.recall,
                        r.f1_score,
                        r.pk_score,
                        r.windowdiff_score,
                    ]:
                        if value is not None:
                            line += f"{value:.3f} & "
                        else:
                            line += "--- & "
                    lines.append(line.rstrip(" & ") + r" \\")

        lines.extend([r"\bottomrule", r"\end{tabular}"])
        return "\n".join(lines)

    def _group_by_language_tool(self) -> Dict[str, Dict[str, List[ExperimentResult]]]:
        """Group results by language and tool."""
        grouped = {}
        for result in self.results:
            lang = result.language
            tool = result.tool
            if lang not in grouped:
                grouped[lang] = {}
            if tool not in grouped[lang]:
                grouped[lang][tool] = []
            grouped[lang][tool].append(result)
        return grouped