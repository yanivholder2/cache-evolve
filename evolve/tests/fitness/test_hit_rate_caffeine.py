"""End-to-end caffeine hit-rate test (marked `caffeine_needed`).

Builds the variant cdylib, runs the Caffeine simulator against the good
fixture variant, and checks the hit-rate plugin output for shape +
non-zero hit rate. Requires ~/projects/caffeine and a working JDK + cargo.
"""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from src.config import load_configs
from src.fitness._contract import validate_plugin_dict
from src.fitness.caffeine_simulator import (
    HARDCODED_SIZE,
    HARDCODED_TRACE,
    simulate_variant,
)
from src.fitness.plugins import hit_rate
from src.identifiers import CycleId, VariantId
from src.paths import CycleVariant, Layout

EVOLVE_ROOT = Path(__file__).parent.parent.parent
REPO_ROOT = EVOLVE_ROOT.parent
FIXTURES_DIR = EVOLVE_ROOT / "tests" / "fixtures"

VARIANT = CycleVariant(CycleId(1), VariantId(1))


def _stage_good(tmp_path: Path) -> tuple[Layout, CycleVariant]:
    layout = Layout(evolve_root=tmp_path / "evolve", outputs_root=tmp_path / "out")
    layout.evolve_root.mkdir(parents=True, exist_ok=True)
    layout.sources.rust_scaffold_dir.parent.mkdir(parents=True, exist_ok=True)
    layout.sources.rust_scaffold_dir.symlink_to(EVOLVE_ROOT / "src" / "variant" / "rust_scaffold")
    # load_configs reads <evolve_root>/config.toml.
    shutil.copy(EVOLVE_ROOT / "config.toml", layout.evolve_root / "config.toml")
    variant_dir = layout.outputs.cycle(VARIANT.cycle_id).variant(VARIANT.variant_id)
    shutil.copytree(FIXTURES_DIR / "good" / "impl", variant_dir.impl_dir)
    return layout, VARIANT


@pytest.mark.caffeine_needed
def test_hit_rate_caffeine_end_to_end(tmp_path):
    layout, variant = _stage_good(tmp_path)
    cfg = load_configs(layout.evolve_root)
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

    caffeine_out = variant.resolve(layout).root / "caffeine"
    assert (caffeine_out / f"hit_rate_{HARDCODED_SIZE}.csv").is_file()
    assert (caffeine_out / "gradle.stdout.log").is_file()
    assert (caffeine_out / "gradle.stderr.log").is_file()
    rust_logs = list((caffeine_out / "rust-logs").glob("dash_variant_*.log"))
    assert rust_logs, f"no rust JNI log under {caffeine_out / 'rust-logs'}"
