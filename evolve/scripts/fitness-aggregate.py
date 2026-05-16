#!/usr/bin/env python3
"""Aggregate fitness plugin outputs into a FitnessVector for a variant.

Usage: fitness-aggregate.py <variant-id> <plugin-output-dir>

Reads all *.json files from the plugin output dir, validates each against the
plugin contract, and emits a FitnessVector to stdout as JSON.

Required plugin JSON shape: { dim, values, summary, higher_is_better }.
"""

import json
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "fitness"))

from _contract import ContractError, validate_plugin_dict  # noqa: E402


def main():
    if len(sys.argv) != 3:
        print("usage: fitness-aggregate.py <variant-id> <plugin-output-dir>", file=sys.stderr)
        sys.exit(1)

    variant = sys.argv[1]
    plugin_dir = Path(sys.argv[2])
    worker_id = os.environ.get("WORKER_ID", "local")

    dimensions = []
    for path in sorted(plugin_dir.glob("*.json")):
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            print(f"error: malformed JSON in {path.name}: {e}", file=sys.stderr)
            sys.exit(1)
        try:
            validate_plugin_dict(data, source=path.name)
        except ContractError as e:
            print(f"error: {e}", file=sys.stderr)
            sys.exit(1)
        dimensions.append({
            "dim": data["dim"],
            "values": data["values"],
            "summary": data["summary"],
            "higher_is_better": data["higher_is_better"],
        })

    if not dimensions:
        print(f"error: no plugin outputs found in {plugin_dir}", file=sys.stderr)
        sys.exit(1)

    fv = {"variant": variant, "worker_id": worker_id, "dimensions": dimensions}
    print(json.dumps(fv))


if __name__ == "__main__":
    main()
