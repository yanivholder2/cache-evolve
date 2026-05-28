"""End-to-end caffeine hit-rate test (marked `caffeine_needed`).

Builds the variant cdylib, runs the Caffeine simulator against the good
fixture variant, and checks the hit-rate plugin output for shape +
non-zero hit rate. Requires ~/projects/caffeine and a working JDK + cargo.
"""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from evolve.config import load_configs
from evolve.fitness._contract import validate_plugin_dict
from evolve.fitness.caffeine_simulator import (
    HARDCODED_SIZE,
    HARDCODED_TRACE,
    simulate_variant,
)
from evolve.fitness.plugins import hit_rate
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout, SourcesLayout, VariantDir

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = REPO_ROOT / "tests" / "fixtures"

VARIANT = CycleVariant(CycleId(1), VariantId(1))


def _stage_good(tmp_path: Path) -> tuple[Layout, CycleVariant]:
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out")
    layout.repo_root.mkdir(parents=True, exist_ok=True)
    real = SourcesLayout(root=REPO_ROOT)
    layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_dir.symlink_to(real.subject_dir)
    layout.sources.framework_crate_dir.symlink_to(real.root / "framework-rs")
    shutil.copy(real.config_toml, layout.sources.config_toml)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    # `good` fixture overlay is empty — files_dir is created empty so the
    # rest of the variant tree matches what a real cycle would produce.
    variant_dir.files_dir.mkdir(parents=True, exist_ok=True)
    return layout, VARIANT


@pytest.mark.caffeine_needed
def test_hit_rate_caffeine_end_to_end(tmp_path):
    layout, variant = _stage_good(tmp_path)
    cfg = load_configs(layout.repo_root)
    results = simulate_variant(variant, layout=layout, repo_root=REPO_ROOT, cfg=cfg)

    assert results.results, "caffeine produced no results"
    [single] = results.results
    assert single.trace == HARDCODED_TRACE
    assert single.size == HARDCODED_SIZE
    assert single.requests > 0
    assert 0.0 < single.hit_rate <= 1.0

    record = hit_rate.compute(results=results)
    validate_plugin_dict(record, source="hit-rate")
    assert record["dim"] == "hit-rate"
    assert record["higher_is_better"] is True
    assert record["summary"] > 0
    assert record["values"] == [{
        "trace": HARDCODED_TRACE,
        "size": HARDCODED_SIZE,
        "hit_rate": single.hit_rate,
    }]

    caffeine_out = variant.resolve(layout).caffeine_dir
    assert caffeine_out.hit_rate_csv(HARDCODED_SIZE).is_file()
    assert caffeine_out.gradle_stdout_log.is_file()
    assert caffeine_out.gradle_stderr_log.is_file()
    rust_logs = list(caffeine_out.rust_logs_dir.glob("dash_*.log"))
    assert rust_logs, f"no rust JNI log under {caffeine_out.rust_logs_dir}"
