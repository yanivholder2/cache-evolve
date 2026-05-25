"""Atomic append of one JSONL row to the evolve insight ledger.

Enforces invariants (ADR-0003):
- All path-valued fields must be repo-relative (no leading /).
- Append is atomic under concurrent writers via flock + atomic rename.
"""

from __future__ import annotations

import fcntl
import json
import os
from pathlib import Path

from src._io import read_utf8, write_utf8

# Free-form body fields exempt from the repo-relative path rule.
_NON_PATH_FIELDS = frozenset({"cycle_insight"})


def _validate(row: dict) -> None:
    for key, value in row.items():
        if key in _NON_PATH_FIELDS:
            continue
        if isinstance(value, str) and value.startswith("/"):
            raise ValueError(f"field '{key}' must be repo-relative, got absolute path: {value!r}")


def append_row(ledger: Path, row: dict) -> None:
    """Atomically appends `row` to `ledger` as one JSONL line."""

    _validate(row)
    line = json.dumps(row, separators=(",", ":"))

    ledger.parent.mkdir(parents=True, exist_ok=True)
    lock = ledger.with_suffix(".lock")

    with open(lock, "w") as lf:
        fcntl.flock(lf, fcntl.LOCK_EX)
        try:
            existing = read_utf8(ledger) if ledger.exists() else ""
            new_content = existing + line + "\n"
            tmp = ledger.parent / f".tmp-{os.getpid()}"
            write_utf8(tmp, new_content)
            tmp.rename(ledger)
        finally:
            fcntl.flock(lf, fcntl.LOCK_UN)
