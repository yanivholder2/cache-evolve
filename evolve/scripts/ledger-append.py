#!/usr/bin/env python3
"""Atomic append of one JSONL row to the evolve insight ledger.

Usage: echo '<json>' | python3 ledger-append.py <ledger-path>

Enforces v2-ready invariants (ADR-0003):
- Row must carry a `worker_id` field.
- All path-valued fields must be repo-relative (no leading /).
- Append is atomic under concurrent writers via flock + atomic rename.
"""

import fcntl
import json
import os
import sys
import tempfile
from pathlib import Path


def _validate(row: dict) -> None:
    """Validate row for v2-ready invariants (worker_id and repo-relative paths)."""
    if "worker_id" not in row:
        raise ValueError("row missing required field: worker_id")
    for key, value in row.items():
        if isinstance(value, str) and value.startswith("/"):
            raise ValueError(f"field '{key}' must be repo-relative, got absolute path: {value!r}")


def _atomic_append(ledger: Path, line: str) -> None:
    """Atomic append of a line to the ledger using flock + atomic rename."""
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


def main() -> None:
    """Entry point: read JSON from stdin, validate, and append to ledger."""
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <ledger-path>", file=sys.stderr)
        sys.exit(1)

    ledger = Path(sys.argv[1])
    raw = sys.stdin.read().strip()

    try:
        row = json.loads(raw)
    except json.JSONDecodeError as e:
        print(f"invalid JSON: {e}", file=sys.stderr)
        sys.exit(1)

    try:
        _validate(row)
    except ValueError as e:
        print(f"validation error: {e}", file=sys.stderr)
        sys.exit(1)

    _atomic_append(ledger, json.dumps(row, separators=(",", ":")))


if __name__ == "__main__":
    main()
