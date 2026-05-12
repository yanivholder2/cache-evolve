"""Tests for scripts/ledger-append.py — atomic concurrent append."""

import json
import subprocess
import sys
import threading
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent.parent / "scripts"
LEDGER_APPEND = SCRIPTS_DIR / "ledger-append.py"


def run_append(ledger_path: Path, row: dict, repo_root: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(LEDGER_APPEND), str(ledger_path)],
        input=json.dumps(row),
        capture_output=True,
        text=True,
        cwd=repo_root,
    )


def test_single_append_writes_one_row(tmp_path):
    ledger = tmp_path / "cycles.jsonl"
    row = {"cycle": "0001", "worker_id": "local", "cycle_dir": "evolve/cycles/0001"}
    result = run_append(ledger, row, tmp_path)
    assert result.returncode == 0, result.stderr
    lines = ledger.read_text().splitlines()
    assert len(lines) == 1
    assert json.loads(lines[0]) == row


def test_row_must_have_worker_id(tmp_path):
    ledger = tmp_path / "cycles.jsonl"
    row = {"cycle": "0001", "cycle_dir": "evolve/cycles/0001"}  # missing worker_id
    result = run_append(ledger, row, tmp_path)
    assert result.returncode != 0
    assert not ledger.exists()


def test_row_must_have_no_absolute_paths(tmp_path):
    ledger = tmp_path / "cycles.jsonl"
    row = {"cycle": "0001", "worker_id": "local", "cycle_dir": "/absolute/path"}
    result = run_append(ledger, row, tmp_path)
    assert result.returncode != 0


def test_concurrent_appends_produce_no_torn_rows(tmp_path):
    ledger = tmp_path / "cycles.jsonl"
    n_writers = 20
    errors = []

    def append_one(i: int):
        row = {"cycle": "0001", "worker_id": "local", "seq": i, "cycle_dir": "evolve/cycles/0001"}
        result = run_append(ledger, row, tmp_path)
        if result.returncode != 0:
            errors.append(f"writer {i}: {result.stderr}")

    threads = [threading.Thread(target=append_one, args=(i,)) for i in range(n_writers)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert not errors, errors

    lines = ledger.read_text().splitlines()
    assert len(lines) == n_writers, f"expected {n_writers} rows, got {len(lines)}"
    for line in lines:
        parsed = json.loads(line)  # must not raise — no torn rows
        assert "worker_id" in parsed
