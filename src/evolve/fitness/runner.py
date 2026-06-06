"""Per-variant fitness computation: run every plugin, aggregate into a
fitness vector, and persist both to the variant layout."""

from __future__ import annotations

import json
import logging
import shutil
from pathlib import Path
from tempfile import mkdtemp
from typing import Callable

from evolve._io import read_utf8, write_json
from evolve.config import Configs
from evolve.fitness._contract import ContractError, validate_plugin_dict
from evolve.fitness.caffeine_simulator import CaffeineSimulatorError, simulate_variant
from evolve.fitness.plugins import hit_rate, metadata_size, per_op_time
from evolve.fitness.plugins._common import PluginError
from evolve.paths import Layout, Variant
from evolve.subject import load_subject

log = logging.getLogger(__name__)


class FitnessError(Exception):
    pass


def compute_fitness(
    variant: Variant,
    *,
    layout: Layout,
    repo_root: Path,
    cfg: Configs,
) -> None:
    """Run every fitness plugin for one variant, write per-plugin JSONs,
    and write the aggregated fitness vector."""

    variant_dir = variant.resolve(layout)
    fitness_dir = variant_dir.fitness_dir
    fitness_dir.mkdir(parents=True, exist_ok=True)

    # One cargo target shared by this variant's cdylib + bench bin builds, so the
    # dependency crates compile once instead of once per build. Per-variant (the
    # builds are sequential here, and a per-run target would let variants clobber
    # each other's same-named artifacts). Cleaned once all builds are done.
    target_dir = Path(mkdtemp(prefix="variant-target-"))
    try:
        log.debug("[%s] caffeine simulate (build cdylib + run)", variant)
        try:
            caffeine_results = simulate_variant(
                variant, layout=layout, repo_root=repo_root, cfg=cfg, target_dir=target_dir
            )
        except CaffeineSimulatorError as exception:
            log.error("[%s] caffeine simulator failed: %s", variant, exception)
            raise FitnessError(f"caffeine simulator failed for {variant}: {exception}") from exception

        subject = load_subject(layout.sources.subject_dir)
        # Capability (an active dimension's subject can measure it) is enforced once at
        # the boundary by cli._validate_active_dimensions; the runner trusts it and just
        # builds a plugin per active dimension.
        active = set(cfg.fitness.dimensions)
        plugins: dict[str, Callable[[], dict]] = {}
        if "hit-rate" in active:
            plugins["hit-rate"] = lambda: hit_rate.compute(results=caffeine_results)
        if "per-op-time" in active:
            plugins["per-op-time"] = lambda: per_op_time.compute(
                variant=variant,
                layout=layout,
                repo_root=repo_root,
                binary_name=subject.bench.binary_name,
                target_dir=target_dir,
            )
        if "metadata-size" in active:
            plugins["metadata-size"] = lambda: metadata_size.compute(
                variant=variant,
                layout=layout,
                repo_root=repo_root,
                binary_name=subject.metadata.binary_name,
                target_dir=target_dir,
            )
        for name, fn in plugins.items():
            log.debug("[%s] plugin %s", variant, name)
            try:
                record = fn()
            except PluginError as exception:
                raise FitnessError(f"plugin {name} failed for {variant}: {exception}") from exception
            write_json(fitness_dir / f"{name}.json", record)
    finally:
        shutil.rmtree(
            target_dir,
            onexc=lambda fn, path, exception: log.warning(
                "shared target cleanup failed for %s: %s", path, exception
            ),
        )

    log.debug("[%s] aggregate fitness-vector", variant)
    fv = _aggregate(variant.label, fitness_dir)
    write_json(variant_dir.fitness_vector_json, fv)


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
