"""Unit tests for `src.hypothesis_parser`."""

from __future__ import annotations

from pathlib import Path

import pytest

from src import hypothesis_parser
from src.hypothesis_parser import Baseline, CycleVariant
from src.identifiers import CycleId, VariantId
from src.paths import Layout


def _write(tmp_path: Path, body: str) -> Path:
    p = tmp_path / "H01.md"
    p.write_text(body, encoding="utf-8")
    return p


def test_parse_baseline_parent(tmp_path):
    p = _write(
        tmp_path,
        "# H01\n\nBias eviction toward stash to raise hit rate.\n\n"
        "## Parent\n\nbaseline\n\n"
        "## Changes\n\nReplace evict_one() victim selection.\n",
    )
    h = hypothesis_parser.parse(p)
    assert h.claim == "Bias eviction toward stash to raise hit rate."
    assert h.parent_variant == Baseline()
    assert h.changes == "Replace evict_one() victim selection."


def test_parse_cycle_variant_parent(tmp_path):
    p = _write(
        tmp_path,
        "Skip promotion when source is cold to cut per-op time.\n\n"
        "## Parent\n\ncycle/0002/v003\n\n"
        "## Changes\n\nGuard the promote() call.\n",
    )
    h = hypothesis_parser.parse(p)
    assert h.parent_variant == CycleVariant(cycle_id=CycleId(2), variant_id=VariantId(3))


def test_parse_skips_header_and_blank_lines(tmp_path):
    p = _write(
        tmp_path,
        "\n\n# H01 header\n\nClaim sentence here.\n\n"
        "## Parent\n\nbaseline\n\n## Changes\n\nstuff\n",
    )
    h = hypothesis_parser.parse(p)
    assert h.claim == "Claim sentence here."


def test_parse_missing_parent_raises(tmp_path):
    p = _write(tmp_path, "Claim line.\n\n## Changes\n\nstuff\n")
    with pytest.raises(ValueError, match="missing '## Parent'"):
        hypothesis_parser.parse(p)


def test_parse_missing_changes_raises(tmp_path):
    p = _write(tmp_path, "Claim line.\n\n## Parent\n\nbaseline\n")
    with pytest.raises(ValueError, match="missing '## Changes'"):
        hypothesis_parser.parse(p)


def test_parse_empty_parent_section_raises(tmp_path):
    p = _write(tmp_path, "Claim.\n\n## Parent\n\n\n## Changes\n\nstuff\n")
    with pytest.raises(ValueError, match="'## Parent' section is empty"):
        hypothesis_parser.parse(p)


def test_parse_empty_changes_section_raises(tmp_path):
    p = _write(tmp_path, "Claim.\n\n## Parent\n\nbaseline\n\n## Changes\n\n\n")
    with pytest.raises(ValueError, match="'## Changes' section is empty"):
        hypothesis_parser.parse(p)


def test_parse_malformed_parent_raises(tmp_path):
    p = _write(
        tmp_path,
        "Claim.\n\n## Parent\n\ncycle/2/v3\n\n## Changes\n\nstuff\n",
    )
    with pytest.raises(ValueError, match="malformed parent-variant identifier"):
        hypothesis_parser.parse(p)


def test_parse_no_claim_raises(tmp_path):
    p = _write(tmp_path, "# H01\n\n## Parent\n\nbaseline\n\n## Changes\n\nstuff\n")
    with pytest.raises(ValueError, match="missing claim"):
        hypothesis_parser.parse(p)


def test_baseline_resolves_to_baseline_root(tmp_path):
    evolve_root = tmp_path / "evolve"
    (evolve_root / "subject" / "baseline").mkdir(parents=True)
    layout = Layout(evolve_root=evolve_root, outputs_root=tmp_path / "outputs")
    assert Baseline().resolve(layout) == evolve_root / "subject" / "baseline"


def test_cycle_variant_resolves_to_variant_dir(tmp_path):
    outputs_root = tmp_path / "outputs"
    variant_dir = outputs_root / "cycles" / "0002" / "variants" / "v003"
    variant_dir.mkdir(parents=True)
    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=outputs_root)
    assert CycleVariant(cycle_id=CycleId(2), variant_id=VariantId(3)).resolve(layout) == variant_dir


def test_cycle_variant_missing_dir_raises(tmp_path):
    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=tmp_path / "outputs")
    with pytest.raises(FileNotFoundError, match="parent variant dir not found"):
        CycleVariant(cycle_id=CycleId(1), variant_id=VariantId(1)).resolve(layout)


def test_parent_variant_str_round_trips(tmp_path):
    """`str(parent_variant)` returns the on-disk citation form."""

    assert str(Baseline()) == "baseline"
    assert str(CycleVariant(cycle_id=CycleId(2), variant_id=VariantId(3))) == "cycle/0002/v003"
