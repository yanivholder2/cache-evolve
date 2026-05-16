"""Cohort fan-out tests for cycle-start.sh.

Verifies issue #60:
- cohort_size knob (config.toml) honored
- cohort.md + hypotheses/HNN.md per Hypothesis
- N variants produced, each with impl/ and eval/
- variant-implementer subagent invoked in parallel (P1)
- every variant flows through real gates from #57

Uses COHORT_SIZE=2 to keep the test fast — the default-of-6 invariant is
verified separately by reading config.toml.
"""

import json
import os
import re
import shutil
import subprocess
import time
from pathlib import Path

EVOLVE_SRC = Path(__file__).parent.parent
SCRIPTS_DIR = EVOLVE_SRC / "scripts"
CYCLE_START = SCRIPTS_DIR / "cycle-start.sh"
REAL_REPO_ROOT = EVOLVE_SRC.parent


def _setup_repo(tmp_path: Path) -> Path:
    evolve_src = Path(__file__).parent.parent
    evolve_dst = tmp_path / "evolve"
    shutil.copytree(
        evolve_src,
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


def test_default_cohort_size_in_config_is_six():
    """AC: cohort size knob in config.toml; default 6."""
    config = (EVOLVE_SRC / "config.toml").read_text()
    match = re.search(r"^cohort_size\s*=\s*(\d+)", config, re.MULTILINE)
    assert match, "config.toml must declare cohort_size"
    assert int(match.group(1)) == 6


def test_cohort_produces_n_variants(tmp_path):
    """AC: a cohort run produces N parallel Variants in one cycle directory."""
    repo_root = _setup_repo(tmp_path)
    result = _run_cycle(repo_root, {"COHORT_SIZE": "3"})
    assert result.returncode == 0, result.stderr

    variants_dir = repo_root / "evolve" / "cycles" / "0001" / "variants"
    variants = sorted(p.name for p in variants_dir.iterdir() if p.is_dir())
    assert variants == ["v001", "v002", "v003"]
    for v in variants:
        assert (variants_dir / v / "impl" / "segment.rs").is_file()
        assert (variants_dir / v / "impl" / "manifest.toml").is_file()
        assert (variants_dir / v / "eval" / "fitness-vector.json").is_file()
        assert (variants_dir / v / "insight.md").is_file()


def test_cohort_writes_cohort_md_and_per_hypothesis_docs(tmp_path):
    """AC: orchestrator writes cohort.md and hypotheses/HNN.md per Hypothesis."""
    repo_root = _setup_repo(tmp_path)
    result = _run_cycle(repo_root, {"COHORT_SIZE": "3"})
    assert result.returncode == 0, result.stderr

    cycle = repo_root / "evolve" / "cycles" / "0001"
    cohort_md = cycle / "cohort.md"
    assert cohort_md.is_file()

    hypotheses = cycle / "hypotheses"
    assert hypotheses.is_dir()
    files = sorted(p.name for p in hypotheses.iterdir() if p.suffix == ".md")
    assert files == ["H01.md", "H02.md", "H03.md"]

    body = cohort_md.read_text()
    for hid in ("H01", "H02", "H03"):
        assert hid in body, f"cohort.md must reference {hid}"


def test_variant_implementer_invoked_in_parallel(tmp_path):
    """AC: orchestrator fans out N subagent invocations in parallel (P1).

    Use an overridable VARIANT_IMPLEMENTER that sleeps and records its start
    time; assert all N implementer starts overlap within the sleep window.
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
sleep 1
cp "{good_impl}/segment.rs" "$OUT_IMPL/segment.rs"
cp "{good_impl}/manifest.toml" "$OUT_IMPL/manifest.toml"
date +%s.%N > "{stamp_dir}/$(basename "$(dirname "$OUT_IMPL")").end"
"""
    )
    implementer.chmod(0o755)

    t0 = time.time()
    result = _run_cycle(
        repo_root,
        {"COHORT_SIZE": "4", "VARIANT_IMPLEMENTER": str(implementer)},
    )
    elapsed = time.time() - t0
    assert result.returncode == 0, result.stderr

    # 4 implementers × 1s sleep each: parallel ≈ ~1s, serial ≈ ≥4s.
    # We can't time the whole script (gates+fitness dominate), but we can compare
    # implementer stamps directly: all start timestamps must fall within < 1s of each other.
    starts = sorted(float(p.read_text()) for p in stamp_dir.glob("*.start"))
    assert len(starts) == 4
    spread = starts[-1] - starts[0]
    assert spread < 0.8, f"implementer starts spread by {spread:.2f}s — not parallel"


def test_each_variant_flows_through_real_gates(tmp_path):
    """AC: each Variant flows through the real gates from #57 (G1 + G2)."""
    repo_root = _setup_repo(tmp_path)
    result = _run_cycle(repo_root, {"COHORT_SIZE": "2"})
    assert result.returncode == 0, result.stderr

    # gate-check.sh prints "G1: passed" and "G2: passed" per variant on stderr.
    g1 = result.stderr.count("G1: passed")
    g2 = result.stderr.count("G2: passed")
    assert g1 >= 2, f"expected G1 to run per variant, saw {g1}: {result.stderr}"
    assert g2 >= 2, f"expected G2 to run per variant, saw {g2}: {result.stderr}"


def test_winners_pareto_front_spans_cohort(tmp_path):
    """All variants evaluated and considered in the Pareto front computation."""
    repo_root = _setup_repo(tmp_path)
    result = _run_cycle(repo_root, {"COHORT_SIZE": "3"})
    assert result.returncode == 0, result.stderr

    winners = json.loads((repo_root / "evolve" / "cycles" / "0001" / "winners.json").read_text())
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
    """AC: variant-implementer subagent under .claude/agents/ as plain markdown."""
    agent_md = REAL_REPO_ROOT / ".claude" / "agents" / "variant-implementer.md"
    assert agent_md.is_file(), f"missing {agent_md}"
    body = agent_md.read_text()
    # Must mention what the subagent reads and writes.
    assert "hypothesis" in body.lower()
    assert "impl" in body.lower()
