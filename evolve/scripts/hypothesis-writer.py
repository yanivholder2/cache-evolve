#!/usr/bin/env python3
"""Hypothesis writer — assembles a prompt and asks the LLM for N Hypotheses.

Usage:
  hypothesis-writer.py N

Emits exactly N non-empty lines on stdout, one Hypothesis per line. Fails
loudly on any deviation (wrong line count, empty output, LLM non-zero exit).

Env:
  EVOLVE_ROOT       — path to evolve/ (default: parent of this script's dir)
  EVOLVE_LLM_CMD    — full LLM CLI invocation per ADR-0005
                      (default: "gemini -p"). Prompt appended as final argv.
  CYCLE_NUM         — used for the raw-log filename. Default "0000".

Inputs (token-frugal, ADR-0002): the wrapper concatenates the subagent
markdown (`.claude/agents/hypothesis-writer.md`) with a `## Context` block
containing the whole `evolve/leaderboard.md`, the whole `evolve/ledger/
cycles.jsonl`, and the whole `evolve/subject/extension-points.md`. The LLM
reads files only via this assembled prompt — never directly. Section sizes
are logged so prompt blow-up is observable; the unbounded read is acceptable
while the ledger is small and is the single hand-off point for #64 to swap
in a curated context bundle.
"""
from __future__ import annotations

import logging
import os
import shlex
import subprocess
import sys
from pathlib import Path
from typing import NoReturn

DEFAULT_LLM_CMD = "gemini -p"

log = logging.getLogger("hypothesis-writer")


def die(msg: str) -> NoReturn:
    log.error(msg)
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def setup_logging() -> None:
    log.setLevel(logging.DEBUG)
    if not log.handlers:
        sh = logging.StreamHandler(sys.stderr)
        sh.setFormatter(logging.Formatter("%(asctime)s %(levelname)s %(message)s", "%H:%M:%S"))
        log.addHandler(sh)


def read_or_empty(path: Path) -> str:
    if path.is_file():
        return path.read_text(encoding="utf-8")
    return ""


def assemble_prompt(evolve_root: Path, repo_root: Path, n: int) -> str:
    subagent_md = repo_root / ".claude" / "agents" / "hypothesis-writer.md"
    leaderboard = read_or_empty(evolve_root / "leaderboard.md")
    ledger = read_or_empty(evolve_root / "ledger" / "cycles.jsonl")
    extension_points = read_or_empty(evolve_root / "subject" / "extension-points.md")
    subagent_body = read_or_empty(subagent_md)
    if not subagent_body:
        die(f"subagent prompt not found at {subagent_md}")

    log.info(
        "prompt sections: subagent=%dB leaderboard=%dB ledger=%dB extension_points=%dB",
        len(subagent_body),
        len(leaderboard),
        len(ledger),
        len(extension_points),
    )

    parts = [
        subagent_body.rstrip(),
        "",
        "## Context",
        "",
        "### Leaderboard",
        "",
        leaderboard.rstrip() or "(empty — cold start)",
        "",
        "### Ledger",
        "",
        ledger.rstrip() or "(empty — cold start)",
        "",
        "### Extension points",
        "",
        extension_points.rstrip() or "(empty)",
        "",
        "## Task",
        "",
        f"Emit exactly {n} Hypotheses, one per line. No numbering, no prose, no blank lines.",
        "",
    ]
    prompt = "\n".join(parts)
    log.info("prompt total: %dB / %d lines", len(prompt), prompt.count("\n") + 1)
    return prompt


def invoke_llm(llm_cmd: str, prompt: str) -> str:
    argv = shlex.split(llm_cmd) + [prompt]
    log.debug("llm argv head: %s", argv[:-1])
    try:
        proc = subprocess.run(argv, capture_output=True, text=True)
    except FileNotFoundError as e:
        die(f"LLM command not found: {e}")
    if proc.returncode != 0:
        log.error("llm stderr: %s", proc.stderr.rstrip())
        die(f"LLM exited {proc.returncode} (cmd: {llm_cmd!r})")
    return proc.stdout


def parse_strict(raw: str, n: int) -> list[str]:
    """Strict: exactly N non-empty lines. No cleanup, no tolerance."""
    lines = [line for line in raw.splitlines() if line.strip()]
    if len(lines) != n:
        die(f"expected {n} hypothesis lines from LLM, got {len(lines)}")
    return lines


def write_raw_log(evolve_root: Path, cycle_num: str, raw: str) -> None:
    logs_dir = evolve_root / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    log_path = logs_dir / f"cycle-{cycle_num}-hypothesis-writer.log"
    log_path.write_text(raw, encoding="utf-8")
    log.info("raw log: %s (%dB)", log_path, len(raw))


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


def main() -> None:
    setup_logging()
    n = parse_n(sys.argv)

    script_dir = Path(__file__).resolve().parent
    evolve_root = Path(os.environ.get("EVOLVE_ROOT") or script_dir.parent).resolve()
    repo_root = Path(os.environ.get("REPO_ROOT") or evolve_root.parent).resolve()
    llm_cmd = os.environ.get("EVOLVE_LLM_CMD") or DEFAULT_LLM_CMD
    cycle_num = os.environ.get("CYCLE_NUM") or "0000"

    log.info("evolve_root=%s repo_root=%s cycle_num=%s", evolve_root, repo_root, cycle_num)
    log.info("EVOLVE_LLM_CMD=%s", llm_cmd)

    prompt = assemble_prompt(evolve_root, repo_root, n)
    raw = invoke_llm(llm_cmd, prompt)
    write_raw_log(evolve_root, cycle_num, raw)
    lines = parse_strict(raw, n)

    for line in lines:
        print(line)


if __name__ == "__main__":
    main()
