"""Tests for src/cycle/ledger_append.append_row."""

import json

import pytest

from src._io import read_utf8
from src.cycle.ledger_append import append_row


def test_single_append_writes_one_row(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    row = {
        "cycle": "0001",
        "cycle_dir": "evolve/cycles/0001",
        "cycle_insight": "# Cycle 0001 Insight\n\nbody\n",
    }
    append_row(ledger, row)
    lines = read_utf8(ledger).splitlines()
    assert len(lines) == 1
    assert json.loads(lines[0]) == row


def test_row_must_have_no_absolute_paths(tmp_path):
    ledger = tmp_path / "cycles-ledger.jsonl"
    row = {"cycle": "0001", "cycle_dir": "/absolute/path"}
    with pytest.raises(ValueError, match="repo-relative"):
        append_row(ledger, row)
