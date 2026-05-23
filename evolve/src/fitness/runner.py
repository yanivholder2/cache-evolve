"""Per-variant fitness computation: run every plugin, aggregate into a
fitness vector, and persist both to the variant layout."""

from __future__ import annotations

import json
import logging
from pathlib import Path

from src._io import read_utf8, write_utf8
from src.fitness._contract import ContractError, validate_plugin_dict
from src.fitness.plugins import hit_rate, metadata_size, per_op_time
from src.paths import Layout, Variant

log = logging.getLogger(__name__)


class FitnessError(Exception):
    pass


def compute_fitness(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
) -> None:
    """Run every fitness plugin for one variant, write per-plugin JSONs,
    and write the aggregated fitness vector."""

    variant_dir = variant.resolve(layout)
    target_dir = layout.target_dir
    fitness_dir = variant_dir.fitness_dir
    fitness_dir.mkdir(parents=True, exist_ok=True)

    plugins = {
        "hit-rate": lambda: hit_rate.compute(repo_root=repo_root, target_dir=target_dir),
        "per-op-time": lambda: per_op_time.compute(evolve_root=layout.evolve_root, target_dir=target_dir),
        "metadata-size": lambda: metadata_size.compute(repo_root=repo_root, target_dir=target_dir),
    }
    for name, fn in plugins.items():
        log.debug("[%s] plugin %s", variant, name)
        record = fn()
        write_utf8(fitness_dir / f"{name}.json", json.dumps(record, indent=2) + "\n")

    log.debug("[%s] aggregate fitness-vector", variant)
    fv = _aggregate(variant.label, fitness_dir)
    write_utf8(variant_dir.fitness_vector_json, json.dumps(fv, indent=2) + "\n")


def _aggregate(variant: str, plugin_dir: Path) -> dict:
    """Read every *.json under plugin_dir, validate against the plugin
    contract, and return a FitnessVector dict."""

    dimensions = []
    for path in sorted(plugin_dir.glob("*.json")):
        try:
            data = json.loads(read_utf8(path))
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
