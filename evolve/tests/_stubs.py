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
    canned files into output_dir.

    Knobs:
      hypothesis_lines  explicit per-file text (one entry → H01.md, etc.)
      n_lines           shorthand: write N stub files "stub hypothesis K"
      exit_code         raise CalledProcessError(exit_code) before writing
      capture_file      log each invocation + task.md into this file
    """

    def __init__(
        self,
        *,
        hypothesis_lines: list[str] | None = None,
        n_lines: int | None = None,
        exit_code: int = 0,
        capture_file: Path | None = None,
    ) -> None:
        self.hypothesis_lines = hypothesis_lines
        self.n_lines = n_lines
        self.exit_code = exit_code
        self.capture_file = capture_file

    def run(
        self,
        agent_name: str,
        output_dir: Path,
        *,
        read_only: bool = False,
    ) -> None:
        if self.capture_file is not None:
            self._capture(agent_name, output_dir, read_only)
        if self.exit_code != 0:
            raise subprocess.CalledProcessError(self.exit_code, "stub-adapter")

        if agent_name == "hypothesis-writer":
            self._write_hypotheses(output_dir)
        elif agent_name == "variant-implementer":
            self._write_variant(output_dir)
        else:
            raise RuntimeError(f"stub-adapter: unknown agent {agent_name!r}")

    def _write_hypotheses(self, output_dir: Path) -> None:
        if self.hypothesis_lines is not None:
            lines = [line for line in self.hypothesis_lines if line]
        elif self.n_lines is not None:
            lines = [f"stub hypothesis {i}" for i in range(1, self.n_lines + 1)]
        else:
            return  # write nothing — wrapper should fail with wrong count

        for i, line in enumerate(lines, start=1):
            (output_dir / f"H{i:02d}.md").write_text(
                f"# Hypothesis H{i:02d}\n\n{line}\n",
                encoding="utf-8",
            )

    def _write_variant(self, output_dir: Path) -> None:
        for name in ("segment.rs", "manifest.toml"):
            shutil.copy(GOOD_IMPL / name, output_dir / name)

    def _capture(self, agent_name, output_dir, read_only):
        task_md = ""
        task_path = output_dir / "task.md"
        if task_path.is_file():
            task_md = task_path.read_text(encoding="utf-8")
        with open(self.capture_file, "a", encoding="utf-8") as f:
            f.write("----- invocation -----\n")
            f.write(f"agent: {agent_name}\n")
            f.write(f"output-dir: {output_dir}\n")
            f.write(f"read-only: {read_only}\n")
            f.write("----- task.md -----\n")
            f.write(task_md if task_md.endswith("\n") else task_md + "\n")
