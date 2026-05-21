"""Aggregate fitness plugin outputs into a FitnessVector for a variant.

Reads all *.json files from the plugin output dir, validates each against the
plugin contract, and returns a FitnessVector dict.

Required plugin JSON shape: { dim, values, summary, higher_is_better }.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "fitness"))

from _contract import ContractError, validate_plugin_dict  # noqa: E402


class FitnessAggregateError(Exception):
    pass


def aggregate(variant: str, plugin_dir: Path) -> dict:
    """Read every *.json under plugin_dir, validate against the plugin
    contract, and return a FitnessVector dict."""

    dimensions = []
    for path in sorted(plugin_dir.glob("*.json")):
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            raise FitnessAggregateError(f"malformed JSON in {path.name}: {e}") from e
        try:
            validate_plugin_dict(data, source=path.name)
        except ContractError as e:
            raise FitnessAggregateError(str(e)) from e
        dimensions.append({
            "dim": data["dim"],
            "values": data["values"],
            "summary": data["summary"],
            "higher_is_better": data["higher_is_better"],
        })

    if not dimensions:
        raise FitnessAggregateError(f"no plugin outputs found in {plugin_dir}")

    return {"variant": variant, "dimensions": dimensions}
