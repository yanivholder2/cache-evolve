"""Tests for the caffeine CSV parser."""

from pathlib import Path

import pytest

from evolve.fitness.caffeine_simulator import CaffeineSimulatorError, _parse_csv


HEADER = "Policy,Hit Rate,Miss Rate,Hits,Misses,Requests,Evictions"


def _write(tmp_path: Path, body: str) -> Path:
    csv_path = tmp_path / "hit_rate_512.csv"
    csv_path.write_text(f"{HEADER}\n{body}\n", encoding="utf-8")
    return csv_path


def test_parse_csv_populated_row(tmp_path):
    csv_path = _write(tmp_path, "dashvariant,42.0,58.0,420,580,1000,10")
    [result] = _parse_csv(csv_path, trace="lirs:multi1.trace.gz", size=512)
    assert result.policy == "dashvariant"
    assert result.hit_rate == pytest.approx(0.42)
    assert result.miss_rate == pytest.approx(0.58)
    assert result.hits == 420
    assert result.misses == 580
    assert result.requests == 1000
    assert result.evictions == 10


def test_parse_csv_empty_numeric_cells_treated_as_zero(tmp_path):
    # Caffeine's Metrics.java emits "" for any rate/long equal to zero.
    csv_path = _write(tmp_path, "dashvariant,,100.0,,1000,1000,")
    [result] = _parse_csv(csv_path, trace="lirs:multi1.trace.gz", size=512)
    assert result.hit_rate == 0.0
    assert result.hits == 0
    assert result.evictions == 0
    assert result.miss_rate == pytest.approx(1.0)
    assert result.misses == 1000
    assert result.requests == 1000


def test_parse_csv_empty_policy_rejected(tmp_path):
    csv_path = _write(tmp_path, ",42.0,58.0,420,580,1000,10")
    with pytest.raises(CaffeineSimulatorError, match="missing required fields.*policy"):
        _parse_csv(csv_path, trace="lirs:multi1.trace.gz", size=512)
