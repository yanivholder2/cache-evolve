"""Per-variant fitness computation: run every plugin, aggregate into a
fitness vector, and persist both to the variant layout."""

from __future__ import annotations

import json
import logging
from pathlib import Path

from src.fitness._contract import ContractError, validate_plugin_dict
from src.fitness.plugins import hit_rate, metadata_size, per_op_time
from src.paths import Layout, Variant, VariantPaths

log = logging.getLogger(__name__)


class FitnessError(Exception):
    pass


def compute_fitness(
    paths: VariantPaths,
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
) -> None:
    """Run every fitness plugin for one variant, write per-plugin JSONs,
    and write the aggregated fitness vector."""

    target_dir = layout.target_dir
    fitness_dir = paths.fitness_dir
    fitness_dir.mkdir(parents=True, exist_ok=True)

    plugins = {
        "hit-rate": lambda: hit_rate.compute(repo_root=repo_root, target_dir=target_dir),
        "per-op-time": lambda: per_op_time.compute(evolve_root=layout.evolve_root, target_dir=target_dir),
        "metadata-size": lambda: metadata_size.compute(repo_root=repo_root, target_dir=target_dir),
    }
    for name, fn in plugins.items():
        log.debug("[%s] plugin %s", variant, name)
        record = fn()
        (fitness_dir / f"{name}.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    log.debug("[%s] aggregate fitness-vector", variant)
    fv = _aggregate(variant.label, fitness_dir)
    paths.fitness_vector_json.write_text(json.dumps(fv, indent=2) + "\n", encoding="utf-8")


def _aggregate(variant: str, plugin_dir: Path) -> dict:
    """Read every *.json under plugin_dir, validate against the plugin
    contract, and return a FitnessVector dict."""

    dimensions = []
    for path in sorted(plugin_dir.glob("*.json")):
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            raise FitnessError(f"malformed JSON in {path.name}: {e}") from e
        try:
            validate_plugin_dict(data, source=path.name)
        except ContractError as e:
            raise FitnessError(str(e)) from e
        dimensions.append({
            "dim": data["dim"],
            "values": data["values"],
            "summary": data["summary"],
            "higher_is_better": data["higher_is_better"],
        })

    if not dimensions:
        raise FitnessError(f"no plugin outputs found in {plugin_dir}")

    return {"variant": variant, "dimensions": dimensions}
