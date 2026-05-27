"""Tests for src/cycle/ledger_digest.digest_prior_cycles."""

from __future__ import annotations

import json

from evolve._io import append_utf8
from evolve.cycle.ledger_digest import digest_prior_cycles


def _row(cycle: str, body: str | None) -> str:
    row: dict = {"cycle": cycle, "cycle_dir": f"cycles/{cycle}"}
    if body is not None:
        row["cycle_insight"] = body
    return json.dumps(row) + "\n"


def test_missing_ledger_returns_empty_string(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    assert digest_prior_cycles(ledger, 3) == ""


def test_empty_ledger_returns_empty_string(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    ledger.touch()
    assert digest_prior_cycles(ledger, 3) == ""


def test_fewer_rows_than_n_returns_all(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    append_utf8(ledger, _row("0001", "first body"))
    append_utf8(ledger, _row("0002", "second body"))

    digest = digest_prior_cycles(ledger, 5)
    assert "### Cycle 0001" in digest
    assert "first body" in digest
    assert "### Cycle 0002" in digest
    assert "second body" in digest


def test_more_rows_than_n_returns_last_n_in_order(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    for n in range(1, 6):
        append_utf8(ledger, _row(f"{n:04d}", f"body {n}"))

    digest = digest_prior_cycles(ledger, 2)
    assert "### Cycle 0001" not in digest
    assert "### Cycle 0003" not in digest
    assert "### Cycle 0004" in digest
    assert "### Cycle 0005" in digest
    assert digest.index("### Cycle 0004") < digest.index("### Cycle 0005")


def test_row_without_cycle_insight_field_is_skipped(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    append_utf8(ledger, _row("0001", None))
    append_utf8(ledger, _row("0002", "second body"))

    digest = digest_prior_cycles(ledger, 5)
    assert "### Cycle 0001" not in digest
    assert "### Cycle 0002" in digest
    assert "second body" in digest


def test_output_structure_matches_format(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    append_utf8(ledger, _row("0007", "the body"))

    digest = digest_prior_cycles(ledger, 5)
    assert digest == "### Cycle 0007\n\nthe body\n\n"
