"""Cohort fan-out tests for cycle-start.sh.

Covers:
- cohort_size knob (config.toml) honored
- cohort.md + hypotheses/HNN.md per Hypothesis
- N variants produced, each with impl/ and eval/
- VARIANT_IMPLEMENTER hook invoked in parallel (P1 parallelism)
- HYPOTHESES env overrides COHORT_SIZE and maps per line to HNN.md
- every variant flows through real G1/G2/G3 gates
- invalid COHORT_SIZE rejected
- failing variant-implementer aborts the cycle with each failure reported

Tests pass small `COHORT_SIZE` (2 or 3) to keep runtime short; the
default-of-6 invariant is verified separately by reading `config.toml`.
"""

import json
import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest

EVOLVE_SRC = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_SRC / "scripts"
CYCLE_START = SCRIPTS_DIR / "cycle-start.sh"
REAL_REPO_ROOT = EVOLVE_SRC.parent

EXPECTED_DEFAULT_COHORT_SIZE = 6
PARALLEL_COHORT_SIZE = 4
IMPLEMENTER_SLEEP_SECONDS = 1
# Parallel total wall time should stay well under the serial lower bound
# (PARALLEL_COHORT_SIZE * sleep). 70% leaves room for fork/exec jitter on
# loaded CI runners while still failing a serial regression.
PARALLEL_WALL_TIME_FRACTION = 0.7


def _setup_repo(tmp_path: Path) -> Path:
    evolve_dst = tmp_path / "evolve"
    shutil.copytree(
        EVOLVE_SRC,
        evolve_dst,
        ignore=shutil.ignore_patterns(".venv", "cycles", "ledger", "__pycache__", "*.pyc"),
    )
    return tmp_path


def _run_cycle(repo_root: Path, extra_env=None) -> subprocess.CompletedProcess:
    env = {
        **os.environ,
        "EVOLVE_ROOT": str(repo_root / "evolve"),
        "REPO_ROOT": str(REAL_REPO_ROOT),
        # default to a small cohort for test speed
        "COHORT_SIZE": "2",
    }
    if extra_env:
        env.update(extra_env)
    return subprocess.run(
        [str(CYCLE_START)],
        capture_output=True,
        text=True,
        cwd=repo_root,
        env=env,
    )


@dataclass
class CohortRun:
    cycle: Path
    stderr: str


@pytest.fixture(scope="module")
def shared_cohort_run(tmp_path_factory):
    """One COHORT_SIZE=3 cycle reused across read-only assertions.

    Each cycle does ~10 cargo invocations (gates + fitness per variant), so
    sharing this fixture cuts the suite by ~3 full cycle runs.
    """
    repo_root = _setup_repo(tmp_path_factory.mktemp("shared"))
    result = _run_cycle(repo_root, {"COHORT_SIZE": "3"})
    assert result.returncode == 0, result.stderr
    return CohortRun(cycle=repo_root / "evolve" / "cycles" / "0001", stderr=result.stderr)


def test_default_cohort_size_in_config_is_six():
    """cohort_size knob in config.toml defaults to 6."""
    config = (EVOLVE_SRC / "config.toml").read_text()
    match = re.search(r"^cohort_size\s*=\s*(\d+)", config, re.MULTILINE)
    assert match, "config.toml must declare cohort_size"
    assert int(match.group(1)) == EXPECTED_DEFAULT_COHORT_SIZE


def test_cohort_produces_n_variants(shared_cohort_run):
    """A cohort run produces N parallel Variants in one cycle directory."""
    variants_dir = shared_cohort_run.cycle / "variants"
    variants = sorted(p.name for p in variants_dir.iterdir() if p.is_dir())
    assert variants == ["v001", "v002", "v003"]
    for v in variants:
        assert (variants_dir / v / "impl" / "segment.rs").is_file()
        assert (variants_dir / v / "impl" / "manifest.toml").is_file()
        assert (variants_dir / v / "eval" / "fitness-vector.json").is_file()
        assert (variants_dir / v / "insight.md").is_file()


def test_cohort_writes_cohort_md_and_per_hypothesis_docs(shared_cohort_run):
    """Orchestrator writes cohort.md and hypotheses/HNN.md per Hypothesis."""
    cohort_md = shared_cohort_run.cycle / "cohort.md"
    assert cohort_md.is_file()

    hypotheses = shared_cohort_run.cycle / "hypotheses"
    assert hypotheses.is_dir()
    files = sorted(p.name for p in hypotheses.iterdir() if p.suffix == ".md")
    assert files == ["H01.md", "H02.md", "H03.md"]

    body = cohort_md.read_text()
    for hid in ("H01", "H02", "H03"):
        assert hid in body, f"cohort.md must reference {hid}"


def test_variant_implementer_invoked_in_parallel(tmp_path):
    """Orchestrator fans out N VARIANT_IMPLEMENTER invocations in parallel (P1).

    Use an overridable VARIANT_IMPLEMENTER that sleeps and stamps start+end
    times; assert total wall time is well below the serial lower bound.
    """
    repo_root = _setup_repo(tmp_path)
    stamp_dir = tmp_path / "stamps"
    stamp_dir.mkdir()

    # Sleeping implementer: stamps start time, sleeps, then writes a minimal impl.
    implementer = tmp_path / "sleep-implementer.sh"
    good_impl = EVOLVE_SRC / "fixtures" / "good" / "impl"
    implementer.write_text(
        f"""#!/usr/bin/env bash
set -euo pipefail
HYPOTHESIS_FILE="$1"
OUT_IMPL="$2"
date +%s.%N > "{stamp_dir}/$(basename "$(dirname "$OUT_IMPL")").start"
sleep {IMPLEMENTER_SLEEP_SECONDS}
cp "{good_impl}/segment.rs" "$OUT_IMPL/segment.rs"
cp "{good_impl}/manifest.toml" "$OUT_IMPL/manifest.toml"
date +%s.%N > "{stamp_dir}/$(basename "$(dirname "$OUT_IMPL")").end"
"""
    )
    implementer.chmod(0o755)

    result = _run_cycle(
        repo_root,
        {"COHORT_SIZE": str(PARALLEL_COHORT_SIZE), "VARIANT_IMPLEMENTER": str(implementer)},
    )
    assert result.returncode == 0, result.stderr

    # Total wall time across all implementer stamps: serial would be
    # >= PARALLEL_COHORT_SIZE * sleep; parallel approaches just `sleep`.
    starts = sorted(float(p.read_text()) for p in stamp_dir.glob("*.start"))
    ends = sorted(float(p.read_text()) for p in stamp_dir.glob("*.end"))
    assert len(starts) == PARALLEL_COHORT_SIZE
    assert len(ends) == PARALLEL_COHORT_SIZE
    total_wall = ends[-1] - starts[0]
    serial_lower_bound = PARALLEL_COHORT_SIZE * IMPLEMENTER_SLEEP_SECONDS
    assert total_wall < serial_lower_bound * PARALLEL_WALL_TIME_FRACTION, (
        f"implementer wall time {total_wall:.2f}s exceeds "
        f"{PARALLEL_WALL_TIME_FRACTION:.0%} of serial bound "
        f"{serial_lower_bound:.2f}s — not parallel"
    )


def test_each_variant_flows_through_real_gates(shared_cohort_run):
    """Each Variant flows through the real G1 + G2 + G3 gates."""
    # gate-check.sh prints "G{1,2,3}: passed" per variant on stderr.
    g1 = shared_cohort_run.stderr.count("G1: passed")
    g2 = shared_cohort_run.stderr.count("G2: passed")
    g3 = shared_cohort_run.stderr.count("G3: passed")
    assert g1 == 3, f"expected G1 once per variant, saw {g1}: {shared_cohort_run.stderr}"
    assert g2 == 3, f"expected G2 once per variant, saw {g2}: {shared_cohort_run.stderr}"
    assert g3 == 3, f"expected G3 once per variant, saw {g3}: {shared_cohort_run.stderr}"


def test_winners_pareto_front_spans_cohort(shared_cohort_run):
    """All variants evaluated and considered in the Pareto front computation."""
    winners = json.loads((shared_cohort_run.cycle / "winners.json").read_text())
    assert len(winners["fitness_vectors"]) == 3
    fv_ids = sorted(fv["variant"] for fv in winners["fitness_vectors"])
    assert fv_ids == ["v001", "v002", "v003"]
    # Pareto front must be a non-empty subset of the cohort. Identical
    # implementations would normally tie on every dimension, but per-op-time
    # microbench measurements carry real noise, so a single variant may emerge
    # as dominant.
    front = winners["pareto_front"]
    assert 1 <= len(front) <= 3
    assert set(front).issubset({"v001", "v002", "v003"})


def test_variant_implementer_agent_markdown_exists():
    """variant-implementer subagent ships under .claude/agents/ as plain markdown."""
    agent_md = REAL_REPO_ROOT / ".claude" / "agents" / "variant-implementer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = agent_md.read_text()
    # Must mention what the subagent reads and writes.
    assert "hypothesis" in body.lower()
    assert "impl" in body.lower()


def test_failing_variant_implementer_aborts_cycle(tmp_path):
    """A failing variant-implementer aborts the cycle with every failure logged."""
    repo_root = _setup_repo(tmp_path)
    failing = tmp_path / "fail-implementer.sh"
    failing.write_text(
        "#!/usr/bin/env bash\n"
        'echo "boom on $(basename "$1")" >&2\n'
        "exit 7\n"
    )
    failing.chmod(0o755)
    result = _run_cycle(
        repo_root,
        {"COHORT_SIZE": "2", "VARIANT_IMPLEMENTER": str(failing)},
    )
    assert result.returncode != 0
    # Both variants must be reported, not just the first.
    assert "variant-implementer for v001 failed (exit 7)" in result.stderr
    assert "variant-implementer for v002 failed (exit 7)" in result.stderr
    assert result.stderr.count("boom on") == 2
    # Cycle never completed → no winners.json.
    assert not (repo_root / "evolve" / "cycles" / "0001" / "winners.json").exists()


def test_hypotheses_env_overrides_cohort_size(tmp_path):
    """HYPOTHESES sets the variant count, overriding COHORT_SIZE; each line lands in HNN.md."""
    repo_root = _setup_repo(tmp_path)
    hypotheses = "first hypothesis text\nsecond hypothesis text\nthird hypothesis text"
    result = _run_cycle(
        repo_root,
        {"COHORT_SIZE": "10", "HYPOTHESES": hypotheses},
    )
    assert result.returncode == 0, result.stderr
    assert "HYPOTHESES has 3 entries; COHORT_SIZE=10 ignored" in result.stderr

    cycle = repo_root / "evolve" / "cycles" / "0001"
    files = sorted(p.name for p in (cycle / "hypotheses").iterdir() if p.suffix == ".md")
    assert files == ["H01.md", "H02.md", "H03.md"]
    assert "first hypothesis text" in (cycle / "hypotheses" / "H01.md").read_text()
    assert "second hypothesis text" in (cycle / "hypotheses" / "H02.md").read_text()
    assert "third hypothesis text" in (cycle / "hypotheses" / "H03.md").read_text()
    variants = sorted(p.name for p in (cycle / "variants").iterdir() if p.is_dir())
    assert variants == ["v001", "v002", "v003"]


def test_hypotheses_skips_empty_lines(tmp_path):
    """Empty lines in HYPOTHESES are dropped, not counted as Variants."""
    repo_root = _setup_repo(tmp_path)
    hypotheses = "alpha\n\nbeta\n"
    result = _run_cycle(repo_root, {"HYPOTHESES": hypotheses, "COHORT_SIZE": ""})
    assert result.returncode == 0, result.stderr

    cycle = repo_root / "evolve" / "cycles" / "0001"
    files = sorted(p.name for p in (cycle / "hypotheses").iterdir() if p.suffix == ".md")
    assert files == ["H01.md", "H02.md"]


def test_invalid_cohort_size_rejected(tmp_path):
    """COHORT_SIZE must be a positive integer."""
    repo_root = _setup_repo(tmp_path)
    for bad in ("0", "abc", "-1"):
        result = _run_cycle(repo_root, {"COHORT_SIZE": bad})
        assert result.returncode != 0, f"expected failure for COHORT_SIZE={bad!r}"
        assert "COHORT_SIZE must be a positive integer" in result.stderr
    # No cycle directory should have been created.
    assert not (repo_root / "evolve" / "cycles" / "0001").exists()
