"""Tests for scripts/convert_msr_trace.py (MSR Cambridge CSV -> lirs keys)."""

from __future__ import annotations

import gzip
from pathlib import Path

import pytest

from scripts import convert_msr_trace


def _write_csv(path: Path, rows: list[str]) -> Path:
    with gzip.open(path, "wt", newline="") as f:
        f.write("".join(f"{row}\n" for row in rows))
    return path


def _row(kind: str, offset: int, size: int) -> str:
    return f"128166372003061629,web,0,{kind},{offset},{size},1331"


def _convert(tmp_path: Path, rows: list[str], *, reads_only: bool = False,
             max_events: int | None = None, name: str = "out.trace.gz") -> list[int]:
    src = _write_csv(tmp_path / "in.csv.gz", rows)
    dst = tmp_path / name
    convert_msr_trace.convert(src, dst, 4096, reads_only, max_events)
    with gzip.open(dst, "rt") as f:
        return [int(line) for line in f.read().splitlines()]


def test_unaligned_request_spans_both_blocks(tmp_path):
    assert _convert(tmp_path, [_row("Read", 4095, 2)]) == [0, 1]


def test_aligned_request_expands_to_each_block(tmp_path):
    assert _convert(tmp_path, [_row("Read", 8192, 3 * 4096)]) == [2, 3, 4]


def test_zero_size_request_is_skipped(tmp_path):
    assert _convert(tmp_path, [_row("Read", 0, 0), _row("Read", 4096, 1)]) == [1]


def test_reads_only_drops_writes(tmp_path):
    rows = [_row("Write", 0, 1), _row("Read", 4096, 1), _row("Write", 8192, 1)]
    assert _convert(tmp_path, rows) == [0, 1, 2]
    assert _convert(tmp_path, rows, reads_only=True) == [1]


def test_unknown_request_type_raises(tmp_path):
    with pytest.raises(ValueError, match="unknown request type 'Trim'"):
        _convert(tmp_path, [_row("Trim", 0, 1)])


def test_wrong_field_count_raises(tmp_path):
    with pytest.raises(ValueError, match="expected 7 fields, got 6"):
        _convert(tmp_path, ["128166372003061629,web,0,Read,0,4096"])


def test_max_events_stops_exactly_mid_request(tmp_path):
    rows = [_row("Read", 0, 3 * 4096), _row("Read", 40960, 4096)]
    assert _convert(tmp_path, rows, max_events=2) == [0, 1]
    assert _convert(tmp_path, rows, max_events=4) == [0, 1, 2, 10]


def test_conversion_is_byte_reproducible(tmp_path):
    # Different output names and separate runs: neither the gzip header's
    # mtime nor its embedded filename may leak into the bytes.
    rows = [_row("Read", 4095, 2), _row("Write", 81920, 9000)]
    _convert(tmp_path, rows, name="a.trace.gz")
    _convert(tmp_path, rows, name="b.trace.gz")
    first = (tmp_path / "a.trace.gz").read_bytes()
    assert first == (tmp_path / "b.trace.gz").read_bytes()
    assert first[4:8] == b"\x00\x00\x00\x00"  # gzip header MTIME field
