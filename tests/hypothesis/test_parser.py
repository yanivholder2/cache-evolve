"""Unit tests for `src.hypothesis.parser`."""

from __future__ import annotations

from pathlib import Path

import pytest

from evolve._io import write_utf8
from evolve.hypothesis import parser
from evolve.identifiers import CycleId, VariantId
from evolve.paths import Baseline, CycleVariant, Layout

from _support import STUB_SUBJECT_NAME


def _write(tmp_path: Path, body: str) -> Path:
    p = tmp_path / "H01.md"
    write_utf8(p, body)
    return p


def test_parse_baseline_parent(tmp_path):
    p = _write(
        tmp_path,
        "# H01\n\nBias eviction toward stash to raise hit rate.\n\n"
        "## Parent\n\nbaseline\n\n"
        "## Changes\n\nReplace evict_one() victim selection.\n",
    )
    h = parser.parse(p)
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
    h = parser.parse(p)
    assert h.parent_variant == CycleVariant(cycle_id=CycleId(2), variant_id=VariantId(3))


def test_parse_skips_header_and_blank_lines(tmp_path):
    p = _write(
        tmp_path,
        "\n\n# H01 header\n\nClaim sentence here.\n\n"
        "## Parent\n\nbaseline\n\n## Changes\n\nstuff\n",
    )
    h = parser.parse(p)
    assert h.claim == "Claim sentence here."


def test_parse_missing_parent_raises(tmp_path):
    p = _write(tmp_path, "Claim line.\n\n## Changes\n\nstuff\n")
    with pytest.raises(ValueError, match="missing '## Parent'"):
        parser.parse(p)


def test_parse_missing_changes_raises(tmp_path):
    p = _write(tmp_path, "Claim line.\n\n## Parent\n\nbaseline\n")
    with pytest.raises(ValueError, match="missing '## Changes'"):
        parser.parse(p)


def test_parse_empty_parent_section_raises(tmp_path):
    p = _write(tmp_path, "Claim.\n\n## Parent\n\n\n## Changes\n\nstuff\n")
    with pytest.raises(ValueError, match="'## Parent' section is empty"):
        parser.parse(p)


def test_parse_empty_changes_section_raises(tmp_path):
    p = _write(tmp_path, "Claim.\n\n## Parent\n\nbaseline\n\n## Changes\n\n\n")
    with pytest.raises(ValueError, match="'## Changes' section is empty"):
        parser.parse(p)


def test_parse_malformed_parent_raises(tmp_path):
    p = _write(
        tmp_path,
        "Claim.\n\n## Parent\n\ncycle/2/v3\n\n## Changes\n\nstuff\n",
    )
    with pytest.raises(ValueError, match="malformed parent-variant identifier"):
        parser.parse(p)


def test_parse_no_claim_raises(tmp_path):
    p = _write(tmp_path, "# H01\n\n## Parent\n\nbaseline\n\n## Changes\n\nstuff\n")
    with pytest.raises(ValueError, match="missing claim"):
        parser.parse(p)


def test_baseline_resolves_to_baseline_root(tmp_path):
    repo_root = tmp_path / "repo"
    baseline_dir = repo_root / "inputs" / "subjects" / STUB_SUBJECT_NAME / "baseline"
    baseline_dir.mkdir(parents=True)
    layout = Layout(repo_root=repo_root, outputs_root=tmp_path / "outputs", subject=STUB_SUBJECT_NAME)
    assert Baseline().resolve(layout).root == baseline_dir


def test_cycle_variant_resolves_to_variant_dir(tmp_path):
    outputs_root = tmp_path / "outputs"
    variant_dir = outputs_root / "cycles" / "0002" / "variants" / "v003"
    variant_dir.mkdir(parents=True)
    layout = Layout(repo_root=tmp_path / "evolve", outputs_root=outputs_root, subject=STUB_SUBJECT_NAME)
    assert CycleVariant(cycle_id=CycleId(2), variant_id=VariantId(3)).resolve(layout).root == variant_dir


def test_variant_str_round_trips(tmp_path):
    assert str(Baseline()) == "baseline"
    assert str(CycleVariant(cycle_id=CycleId(2), variant_id=VariantId(3))) == "cycle/0002/v003"


_RS = frozenset({".rs"})


def test_referenced_files_finds_bare_basename():
    assert parser.referenced_files("Edit eviction_policy.rs victim choice.", _RS) == {"eviction_policy.rs"}


def test_referenced_files_finds_full_path():
    found = parser.referenced_files("Thread it through src/dash/segments/promotion_logic.rs here.", _RS)
    assert found == {"src/dash/segments/promotion_logic.rs"}


def test_referenced_files_finds_multiple_and_dedupes():
    found = parser.referenced_files(
        "Touch item.rs and dash_bucket.rs, then item.rs again.", _RS
    )
    assert found == {"item.rs", "dash_bucket.rs"}


def test_referenced_files_ignores_non_matching_suffixes():
    assert parser.referenced_files("See README.md and config.toml.", _RS) == frozenset()


def test_referenced_files_no_tokens():
    assert parser.referenced_files("Raise the hit rate by 0.05 with a smarter victim.", _RS) == frozenset()


def test_referenced_files_empty_suffixes_matches_nothing():
    assert parser.referenced_files("Edit eviction_policy.rs.", frozenset()) == frozenset()


def test_referenced_files_honors_given_suffixes():
    found = parser.referenced_files("Edit build.rs and Cargo.toml.", frozenset({".toml"}))
    assert found == {"Cargo.toml"}
