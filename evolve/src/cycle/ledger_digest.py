"""Bounded digest of prior Cycle insights, sourced from the ledger row inlined body."""

from __future__ import annotations

import json
import logging
from pathlib import Path

from src._io import read_utf8

log = logging.getLogger("ledger-digest")


def digest_prior_cycles(ledger: Path, n: int) -> str:
    """Return a markdown body with the last `n` cycles from `ledger`.

    Each cycle renders as `### Cycle <id>\\n\\n<body>\\n\\n`. Rows missing
    `cycle_insight` are skipped with a debug log. Returns empty string when the
    ledger does not exist or yields no usable rows.
    """

    all_rows = _load_rows(ledger)
    selected = all_rows[-n:]
    body = _render(selected)
    log.info(
        "digest_prior_cycles: ledger=%s total_rows=%d window=%d selected_cycles=%s body_chars=%d",
        ledger, len(all_rows), n,
        [row.get("cycle") for row in selected],
        len(body),
    )
    return body


def digest_all_cycles(ledger: Path) -> str:
    """Like `digest_prior_cycles` but cumulative — every row from the ledger."""

    rows = _load_rows(ledger)
    body = _render(rows)
    log.info(
        "digest_all_cycles: ledger=%s total_rows=%d cycles=%s body_chars=%d",
        ledger, len(rows),
        [row.get("cycle") for row in rows],
        len(body),
    )
    return body


def _load_rows(ledger: Path) -> list[dict]:
    if not ledger.exists():
        return []
    rows = []
    for raw in read_utf8(ledger).splitlines():
        line = raw.strip()
        if not line:
            continue
        rows.append(json.loads(line))
    return rows


def _render(rows: list[dict]) -> str:
    parts: list[str] = []
    for row in rows:
        body = row.get("cycle_insight")
        if body is None:
            log.debug("skipping ledger row without cycle_insight for cycle %s", row.get("cycle"))
            continue
        parts.append(f"### Cycle {row['cycle']}\n\n{body}\n\n")
    return "".join(parts)
