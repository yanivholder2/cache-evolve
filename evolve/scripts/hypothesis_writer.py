"""Hypothesis-writer driver — invokes an LLMAdapter and verifies it wrote
exactly N H<NN>.md files.

Swap target for #64: when the curated context-bundle assembler lands, the
agent's `task.md` (assembled below) will point at the bundle instead of raw
leaderboard / ledger paths.
"""

from __future__ import annotations

import logging
import re
from pathlib import Path

from adapters import LLMAdapter

AGENT_NAME = "hypothesis-writer"
HYPOTHESIS_FILE_RE = re.compile(r"H\d{2}\.md")

log = logging.getLogger("hypothesis-writer")


def _write_task_md(
    output_dir: Path, n: int, cycle_num: str, evolve_root: Path, outputs_root: Path
) -> None:
    body = (
        f"# Task: hypothesis-writer (cycle {cycle_num})\n\n"
        f"N: {n}\n"
        f"Cycle: {cycle_num}\n"
        f"Output dir: {output_dir}\n\n"
        f"## Read for context\n\n"
        f"- {outputs_root / 'leaderboard.md'}\n"
        f"- {outputs_root / 'ledger' / 'cycles.jsonl'}\n"
        f"- {evolve_root / 'subject' / 'extension-points.md'}\n\n"
        f"## Write\n\n"
        f"Exactly {n} files: H01.md..H{n:02d}.md under the output dir.\n"
        f"Each file's first non-empty non-header line is the canonical "
        f"Hypothesis text. Files must be distinct.\n"
    )
    (output_dir / "task.md").write_text(body, encoding="utf-8")


def write_hypotheses(
    n: int,
    *,
    evolve_root: Path,
    outputs_root: Path,
    cycle_num: str,
    adapter: LLMAdapter,
) -> list[Path]:
    """Drive the adapter to produce N H<NN>.md files; return their paths.

    Raises ValueError on bad N or wrong file count, and propagates any
    exception raised by the adapter.
    """

    if n < 1:
        raise ValueError(f"N must be >= 1, got {n}")

    output_dir = outputs_root / "cycles" / cycle_num / "hypotheses"
    output_dir.mkdir(parents=True, exist_ok=True)
    log.info("N=%d cycle=%s output_dir=%s", n, cycle_num, output_dir)

    _write_task_md(output_dir, n, cycle_num, evolve_root, outputs_root)
    log.debug("wrote %s", output_dir / "task.md")

    adapter.run(agent_name=AGENT_NAME, output_dir=output_dir)

    files = sorted(p for p in output_dir.iterdir() if HYPOTHESIS_FILE_RE.fullmatch(p.name))
    if len(files) != n:
        raise ValueError(
            f"adapter wrote {len(files)} H<NN>.md files in {output_dir}, expected {n}"
        )
    log.info("wrote %d H<NN>.md files in %s", len(files), output_dir)
    return files
