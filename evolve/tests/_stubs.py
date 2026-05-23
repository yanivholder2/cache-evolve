"""In-process LLM adapter stub for tests.

Replaces the old `_mocks/llm-stub.py` subprocess adapter. Tests inject a
`StubLLMAdapter` instance into `run_cycle`/`write_hypotheses` directly.
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

GOOD_IMPL = Path(__file__).parent / "fixtures" / "good" / "impl"


class StubLLMAdapter:
    """Deterministic in-process adapter. Dispatches by agent_name; writes
    canned files into workspace.

    Knobs:
      hypothesis_lines  explicit per-file claim lines (one entry → H01.md, etc.)
      n_lines           shorthand: write N stub files "stub hypothesis K"
      parent_variants   per-hypothesis parent-variant identifier (defaults to "baseline" for each)
      changes           per-hypothesis '## Changes' prose (defaults to "stub changes")
      exit_code         raise CalledProcessError(exit_code) before writing
      capture_file      log each invocation + task.md into this file
    """

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
        read_only: bool,
    ) -> None:
        if self.capture_file is not None:
            self._capture(agent_name, workspace, readable_roots, read_only)
        if self.exit_code != 0:
            raise subprocess.CalledProcessError(self.exit_code, "stub-adapter")

        if agent_name == "hypotheses-writer":
            self._write_hypotheses(workspace)
        elif agent_name == "variant-implementer":
            self._write_variant(workspace)
        else:
            raise RuntimeError(f"stub-adapter: unknown agent {agent_name!r}")

    def _write_hypotheses(self, workspace: Path) -> None:
        if self.hypothesis_lines is not None:
            lines = [line for line in self.hypothesis_lines if line]
        elif self.n_lines is not None:
            lines = [f"stub hypothesis {i}" for i in range(1, self.n_lines + 1)]
        else:
            return  # write nothing — wrapper should fail with wrong count

        for i, line in enumerate(lines, start=1):
            parent_variant = self.parent_variants[i - 1] if self.parent_variants and i - 1 < len(self.parent_variants) else "baseline"
            change = self.changes[i - 1] if self.changes and i - 1 < len(self.changes) else "stub changes"
            (workspace / f"H{i:02d}.md").write_text(
                f"# Hypothesis H{i:02d}\n\n{line}\n\n## Parent\n\n{parent_variant}\n\n## Changes\n\n{change}\n",
                encoding="utf-8",
            )

    def _write_variant(self, workspace: Path) -> None:
        for name in ("segment.rs", "manifest.toml"):
            shutil.copy(GOOD_IMPL / name, workspace / name)

    def _capture(self, agent_name, workspace, readable_roots, read_only):
        task_md = ""
        task_path = workspace / "task.md"
        if task_path.is_file():
            task_md = task_path.read_text(encoding="utf-8")
        with open(self.capture_file, "a", encoding="utf-8") as f:
            f.write("----- invocation -----\n")
            f.write(f"agent: {agent_name}\n")
            f.write(f"workspace: {workspace}\n")
            f.write(f"readable-roots: {readable_roots}\n")
            f.write(f"read-only: {read_only}\n")
            f.write("----- task.md -----\n")
            f.write(task_md if task_md.endswith("\n") else task_md + "\n")
