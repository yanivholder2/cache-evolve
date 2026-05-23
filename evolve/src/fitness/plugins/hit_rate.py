"""Hit-rate fitness plugin."""

from __future__ import annotations

from pathlib import Path

from src.fitness._contract import parse_plugin_stdout
from src.fitness.plugins._common import cargo_build, run_binary

CONFIGS = [
    {"cache": "assoc", "size": 16},
    {"cache": "assoc", "size": 32},
    {"cache": "dash", "size": 16},
    {"cache": "dash", "size": 32},
]
EVENTS = 2000
SEED = 42
RUN_TIMEOUT_S = 60


def compute(*, repo_root: Path, target_dir: Path | None = None) -> dict:
    """Runs the hit-rate plugin across CONFIGS and returns the aggregated plugin record."""

    cargo_build(repo_root, ["--bin", "evolve-hit-rate"], label="evolve-hit-rate", target_dir=target_dir)
    binary = (target_dir or repo_root / "target") / "debug" / "evolve-hit-rate"

    values = []
    for cfg in CONFIGS:
        label = f"evolve-hit-rate --cache {cfg['cache']} --size {cfg['size']}"
        result = run_binary(
            [str(binary), "--cache", cfg["cache"], "--size", str(cfg["size"]),
             "--events", str(EVENTS), "--seed", str(SEED)],
            label=label,
            timeout=RUN_TIMEOUT_S,
        )
        record = parse_plugin_stdout(result.stdout, source=label)
        for entry in record["values"]:
            values.append({"cache": cfg["cache"], "size": cfg["size"], "hit_rate": entry["hit_rate"]})

    # Exclude baseline (assoc) so the Pareto scalar reflects Dash alone, not a mix with the baseline.
    dash_rates = [v["hit_rate"] for v in values if v["cache"] == "dash"]
    if not dash_rates:
        raise RuntimeError("no dash configurations produced a hit_rate")
    summary = sum(dash_rates) / len(dash_rates)
    return {
        "dim": "hit-rate",
        "values": values,
        "summary": round(summary, 6),
        "higher_is_better": True,
    }
