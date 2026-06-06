"""End-to-end caffeine hit-rate test (marked `caffeine_needed`).

Builds the variant cdylib, runs the Caffeine simulator against the good
fixture variant, and checks the hit-rate plugin output for shape +
non-zero hit rate. Requires a caffeine checkout (a `caffeine/` repo sibling
by default, or $CAFFEINE_DIR) and a working JDK + cargo.
"""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from evolve.config import load_configs
from evolve.fitness._contract import validate_plugin_dict
from evolve.fitness.caffeine_simulator import simulate_variant
from evolve.fitness.plugins import hit_rate
from evolve.identifiers import CycleId, VariantId
from evolve.paths import CycleVariant, Layout, SourcesLayout, VariantDir
from evolve.subject import load_subject

from _support import configured_subject

REPO_ROOT = Path(__file__).parent.parent.parent
FIXTURES_DIR = REPO_ROOT / "tests" / "fixtures"
SUBJECT = configured_subject(REPO_ROOT)

VARIANT = CycleVariant(CycleId(1), VariantId(1))


def _stage_good(tmp_path: Path) -> tuple[Layout, CycleVariant]:
    layout = Layout(repo_root=tmp_path / "repo", outputs_root=tmp_path / "out", subject=SUBJECT)
    layout.repo_root.mkdir(parents=True, exist_ok=True)
    real = SourcesLayout(root=REPO_ROOT, subject=SUBJECT)
    layout.sources.subject_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.subject_dir.symlink_to(real.subject_dir)
    layout.sources.framework_crate_dir.symlink_to(real.root / "framework-rs")
    shutil.copy(real.config_toml, layout.sources.config_toml)
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    # `good` fixture overlay is empty — overlay_dir is created empty so the
    # rest of the variant tree matches what a real cycle would produce.
    variant_dir.overlay_dir.mkdir(parents=True, exist_ok=True)
    return layout, VARIANT


@pytest.mark.caffeine_needed
def test_hit_rate_caffeine_end_to_end(tmp_path):
    layout, variant = _stage_good(tmp_path)
    cfg = load_configs(layout.repo_root)
    grid = load_subject(layout.sources.subject_dir).evaluation
    [expected_trace] = grid.traces
    [expected_size] = grid.sizes
    results = simulate_variant(variant, layout=layout, repo_root=REPO_ROOT, cfg=cfg)

    assert results.results, "caffeine produced no results"
    [single] = results.results
    assert single.trace == expected_trace
    assert single.size == expected_size
    assert single.requests > 0
    assert 0.0 < single.hit_rate <= 1.0

    record = hit_rate.compute(results=results)
    validate_plugin_dict(record, source="hit-rate")
    assert record["dim"] == "hit-rate"
    assert record["higher_is_better"] is True
    assert record["summary"] > 0
    assert record["values"] == [{
        "trace": expected_trace,
        "size": expected_size,
        "hit_rate": single.hit_rate,
    }]

    caffeine_out = variant.resolve(layout).caffeine_dir
    assert caffeine_out.hit_rate_csv(expected_size).is_file()
    assert caffeine_out.gradle_stdout_log.is_file()
    assert caffeine_out.gradle_stderr_log.is_file()
    subject = load_subject(layout.sources.subject_dir)
    rust_logs = list(caffeine_out.rust_logs_dir.glob(f"{subject.cdylib_name}_*.log"))
    # Only subjects that declare supports-log-path receive a log path over JNI and
    # write rust logs; the rest (e.g. lru_cache) must not, so assert each way.
    if subject.caffeine.supports_log_path:
        assert rust_logs, f"no rust JNI log under {caffeine_out.rust_logs_dir}"
    else:
        assert not rust_logs, f"subject declares no log-path support but wrote {rust_logs}"
