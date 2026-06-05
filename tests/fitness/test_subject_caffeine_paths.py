"""End-to-end Caffeine wiring check for every subject (marked `caffeine_needed`).

For each subject, builds the baseline cdylib, runs the Caffeine simulator once,
and asserts a valid per-size hit-rate point comes back. This is the coverage
that catches manifest [caffeine] wiring mismatches — e.g. associative_cache's
numOfBuckets being read Java-side as a list (getDoubleList) while the framework
emitted it as a scalar. Requires a caffeine checkout (a `caffeine/` repo
sibling by default, or $CAFFEINE_DIR) and a working JDK + cargo.
"""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from evolve.config import load_configs
from evolve.fitness.caffeine_simulator import simulate_variant
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout, SourcesLayout
from evolve.subject import load_subject

REPO_ROOT = Path(__file__).parent.parent.parent
SUBJECTS = ["dash", "associative_cache", "lru_cache", "tiny_lfu", "s3_fifo"]
VARIANT = CycleVariant(CycleId(1), VariantId(1))


def _stage_baseline(tmp_path: Path, subject: str) -> Layout:
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject=subject)
    layout.repo_root.mkdir(parents=True, exist_ok=True)
    real = SourcesLayout(root=REPO_ROOT, subject=subject)
    layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_dir.symlink_to(real.subject_dir)
    layout.sources.framework_crate_dir.symlink_to(real.root / "framework-rs")
    shutil.copy(real.config_toml, layout.sources.config_toml)
    # Empty overlay -> the baseline crate builds unchanged.
    layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id).overlay_dir.mkdir(parents=True, exist_ok=True)
    return layout


@pytest.mark.caffeine_needed
@pytest.mark.parametrize("subject", SUBJECTS)
def test_subject_caffeine_path_end_to_end(tmp_path, subject):
    layout = _stage_baseline(tmp_path, subject)
    # Only `subject` is symlinked into the staged repo, so override config.toml's
    # selection (which names a different subject) to keep resolution valid.
    cfg = load_configs(layout.repo_root, subject=subject)
    results = simulate_variant(VARIANT, layout=layout, repo_root=REPO_ROOT, cfg=cfg)

    loaded = load_subject(layout.sources.subject_dir)
    grid = loaded.evaluation
    [expected_trace] = grid.traces
    [expected_size] = grid.sizes

    assert results.results, f"{subject}: caffeine produced no results"
    [single] = results.results
    assert single.trace == expected_trace
    assert single.size == expected_size
    assert single.requests > 0, f"{subject}: simulator recorded no requests"
    assert 0.0 < single.hit_rate <= 1.0, f"{subject}: implausible hit rate {single.hit_rate}"

    caffeine_out = VARIANT.resolve(layout).caffeine_dir
    assert caffeine_out.hit_rate_csv(expected_size).is_file()

    # Only subjects that route their JNI log via logPath land it under our
    # caffeine_out; others (assoc, lru) log to the simulator's own default dir.
    if loaded.caffeine.supports_log_path:
        rust_logs = list(caffeine_out.rust_logs_dir.glob(f"{loaded.cdylib_name}_*.log"))
        assert rust_logs, f"{subject}: no rust JNI log under {caffeine_out.rust_logs_dir}"
