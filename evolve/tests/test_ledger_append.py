"""Tests for src/ledger_append.append_row — atomic concurrent append."""

import json
import threading

import pytest

from src.ledger_append import append_row


def test_single_append_writes_one_row(tmp_path):
    ledger = tmp_path / "cycles.jsonl"
    row = {"cycle": "0001", "cycle_dir": "evolve/cycles/0001"}
    append_row(ledger, row)
    lines = ledger.read_text().splitlines()
    assert len(lines) == 1
    assert json.loads(lines[0]) == row


def test_row_must_have_no_absolute_paths(tmp_path):
    ledger = tmp_path / "cycles.jsonl"
    row = {"cycle": "0001", "cycle_dir": "/absolute/path"}
    with pytest.raises(ValueError, match="repo-relative"):
        append_row(ledger, row)


def test_concurrent_appends_produce_no_torn_rows(tmp_path):
    ledger = tmp_path / "cycles.jsonl"
    n_writers = 20
    errors: list[str] = []

    def append_one(i: int):
        row = {"cycle": "0001", "seq": i, "cycle_dir": "evolve/cycles/0001"}
        try:
            append_row(ledger, row)
        except Exception as e:
            errors.append(f"writer {i}: {e}")

    threads = [threading.Thread(target=append_one, args=(i,)) for i in range(n_writers)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert not errors, errors

    lines = ledger.read_text().splitlines()
    assert len(lines) == n_writers, f"expected {n_writers} rows, got {len(lines)}"
    for line in lines:
        json.loads(line)  # must not raise — no torn rows
