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


def _validate(row: dict) -> None:
    for key, value in row.items():
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
            existing = ledger.read_text() if ledger.exists() else ""
            new_content = existing + line + "\n"
            tmp = ledger.parent / f".tmp-{os.getpid()}"
            tmp.write_text(new_content)
            tmp.rename(ledger)
        finally:
            fcntl.flock(lf, fcntl.LOCK_UN)
