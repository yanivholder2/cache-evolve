"""Append one JSONL row to the evolve insight ledger.

All path-valued fields must be repo-relative (no leading /).
"""

from __future__ import annotations

import json
from pathlib import Path

from evolve._io import append_utf8

_NON_PATH_FIELDS = frozenset({"cycle_insight"})


def _validate(row: dict) -> None:
    for key, value in row.items():
        if key in _NON_PATH_FIELDS:
            continue
        if isinstance(value, str) and value.startswith("/"):
            raise ValueError(f"field '{key}' must be repo-relative, got absolute path: {value!r}")


def append_row(ledger: Path, row: dict) -> None:
    """Append `row` to `ledger` as one JSONL line."""

    _validate(row)
    line = json.dumps(row, separators=(",", ":"))

    ledger.parent.mkdir(parents=True, exist_ok=True)
    append_utf8(ledger, line + "\n")
