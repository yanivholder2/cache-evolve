#!/usr/bin/env python3
"""Hypothesis-writer wrapper — invokes the configurable LLM adapter and
verifies it wrote exactly N H<NN>.md files (ADR-0005).

Usage:
  hypothesis-writer.py N

Env:
  EVOLVE_ROOT     — path to evolve/ (default: parent of this script's dir)
  EVOLVE_LLM_CMD  — adapter command (default: evolve/scripts/agents/gemini.sh)
  CYCLE_NUM       — cycle number (default "0001"). Drives output-dir layout.

Output: prints discovered hypothesis-file paths on stdout, one per line.
Aborts loudly on adapter failure or wrong file count.

Swap target for #64: when the curated context-bundle assembler lands, the
agent's `task.md` (assembled below) will point at the bundle instead of raw
leaderboard / ledger paths.
"""
from __future__ import annotations

import os
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import NoReturn

AGENT_NAME = "hypothesis-writer"
DEFAULT_ADAPTER = "evolve/scripts/agents/gemini.sh"
HYPOTHESIS_FILE_RE = re.compile(r"H\d{2}\.md")


def die(msg: str) -> NoReturn:
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def parse_n(argv: list[str]) -> int:
    if len(argv) != 2:
        die("usage: hypothesis-writer.py N")
    try:
        n = int(argv[1])
    except ValueError:
        die(f"N must be a positive integer, got: {argv[1]!r}")
    if n < 1:
        die(f"N must be >= 1, got {n}")
    return n


def write_task_md(output_dir: Path, n: int, cycle_num: str, evolve_root: Path) -> None:
    body = (
        f"# Task: hypothesis-writer (cycle {cycle_num})\n\n"
        f"N: {n}\n"
        f"Cycle: {cycle_num}\n"
        f"Output dir: {output_dir}\n\n"
        f"## Read for context\n\n"
        f"- {evolve_root / 'leaderboard.md'}\n"
        f"- {evolve_root / 'ledger' / 'cycles.jsonl'}\n"
        f"- {evolve_root / 'subject' / 'extension-points.md'}\n\n"
        f"## Write\n\n"
        f"Exactly {n} files: H01.md..H{n:02d}.md under the output dir.\n"
        f"Each file's first non-empty non-header line is the canonical "
        f"Hypothesis text. Files must be distinct.\n"
    )
    (output_dir / "task.md").write_text(body, encoding="utf-8")


def main() -> None:
    n = parse_n(sys.argv)

    script_dir = Path(__file__).resolve().parent
    evolve_root = Path(os.environ.get("EVOLVE_ROOT") or script_dir.parent).resolve()
    repo_root = Path(os.environ.get("REPO_ROOT") or evolve_root.parent).resolve()
    cycle_num = os.environ.get("CYCLE_NUM") or "0001"

    output_dir = evolve_root / "cycles" / cycle_num / "hypotheses"
    output_dir.mkdir(parents=True, exist_ok=True)
    write_task_md(output_dir, n, cycle_num, evolve_root)

    default_adapter = repo_root / DEFAULT_ADAPTER
    llm_cmd = os.environ.get("EVOLVE_LLM_CMD") or str(default_adapter)
    argv = shlex.split(llm_cmd) + [AGENT_NAME, str(output_dir), "--writable", str(output_dir)]

    try:
        proc = subprocess.run(argv, capture_output=True, text=True)
    except FileNotFoundError as e:
        die(f"adapter not found: {e}")
    if proc.returncode != 0:
        if proc.stderr:
            sys.stderr.write(proc.stderr)
        die(f"adapter exited {proc.returncode} (cmd: {llm_cmd!r})")

    files = sorted(p for p in output_dir.iterdir() if HYPOTHESIS_FILE_RE.fullmatch(p.name))
    if len(files) != n:
        die(f"expected {n} H<NN>.md files in {output_dir}, got {len(files)}")

    for p in files:
        print(p)


if __name__ == "__main__":
    main()
