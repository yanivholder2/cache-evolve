"""In-process LLM adapter stub for tests."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from src._io import append_utf8, read_utf8, write_utf8

GOOD_IMPL = Path(__file__).parent / "fixtures" / "good" / "impl"


class StubLLMAdapter:
    """Deterministic in-process adapter that writes canned files."""

    def identity(self) -> str:
        return "StubLLMAdapter"

    def __init__(
        self,
        *,
        hypothesis_lines: list[str] | None = None,
        n_lines: int | None = None,
        parent_variants: list[str] | None = None,
        changes: list[str] | None = None,
        exit_code: int = 0,
        capture_file: Path | None = None,
    ) -> None:
        self.hypothesis_lines = hypothesis_lines
        self.n_lines = n_lines
        self.parent_variants = parent_variants
        self.changes = changes
        self.exit_code = exit_code
        self.capture_file = capture_file

    def run(
        self,
        agent_name: str,
        workspace: Path,
        *,
        readable_roots: list[Path],
        allow_edits: bool,
    ) -> None:
        if self.capture_file is not None:
            self._capture(agent_name, workspace, readable_roots, allow_edits)
        if self.exit_code != 0:
            raise subprocess.CalledProcessError(self.exit_code, "stub-adapter")

        if agent_name == "hypotheses-writer":
            self._write_hypotheses(workspace)
        elif agent_name == "variant-implementer":
            self._write_variant(workspace)
        elif agent_name == "variant-insight-writer":
            write_utf8(workspace / "insight.md", "# Variant Insight\n\nstub body\n")
        elif agent_name == "cycle-insight-writer":
            write_utf8(workspace / "cycle-insight.md", "# Cycle Insight\n\nstub body\n")
        elif agent_name == "cross-cycle-insight-writer":
            write_utf8(workspace / "cross-cycle-insights.md", "# Cross-Cycle Insights\n\nstub body\n")
        else:
            raise RuntimeError(f"stub-adapter: unknown agent {agent_name!r}")

    def _write_hypotheses(self, workspace: Path) -> None:
        if self.hypothesis_lines is not None:
            lines = [line for line in self.hypothesis_lines if line]
        elif self.n_lines is not None:
            lines = [f"stub hypothesis {i}" for i in range(1, self.n_lines + 1)]
        else:
            # Empty output exercises the wrapper's wrong-count failure path.
            return

        for i, line in enumerate(lines, start=1):
            parent_variant = self.parent_variants[i - 1] if self.parent_variants and i - 1 < len(self.parent_variants) else "baseline"
            change = self.changes[i - 1] if self.changes and i - 1 < len(self.changes) else "stub changes"
            write_utf8(
                workspace / f"H{i:02d}.md",
                f"# Hypothesis H{i:02d}\n\n{line}\n\n## Parent\n\n{parent_variant}\n\n## Changes\n\n{change}\n",
            )

    def _write_variant(self, workspace: Path) -> None:
        for name in ("segment.rs", "manifest.toml"):
            shutil.copy(GOOD_IMPL / name, workspace / name)

    def _capture(self, agent_name, workspace, readable_roots, allow_edits):
        task_md = ""
        task_path = workspace / "task.md"
        if task_path.is_file():
            task_md = read_utf8(task_path)
        if not task_md.endswith("\n"):
            task_md += "\n"
        append_utf8(
            self.capture_file,
            "----- invocation -----\n"
            f"agent: {agent_name}\n"
            f"workspace: {workspace}\n"
            f"readable-roots: {readable_roots}\n"
            f"allow-edits: {allow_edits}\n"
            "----- task.md -----\n"
            f"{task_md}",
        )
