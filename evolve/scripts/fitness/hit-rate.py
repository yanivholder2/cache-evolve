#!/usr/bin/env python3
"""Hit-rate fitness plugin.

Iterates the configured trace suite (default: `evolve/traces/dev/`) crossed
with the cache-size sweep, invoking `evolve-hit-rate --trace ... --size ...`
once per combination. Merges the per-invocation plugin records into a single
output:
  { "dim": "hit-rate", "values": [...], "summary": float, "higher_is_better": true }

Environment:
  REPO_ROOT     path to the dash-cache-rs repo (default: 3 levels up from this file)
  EVOLVE_ROOT   path to the evolve/ directory (default: 2 levels up from this file)
  TRACE_SUITE   subdirectory of evolve/traces/ to use (default: 'dev').
                The held-out runner sets this to 'holdout' for Pareto winners.
"""

import json
import os
import sys
from pathlib import Path

_HERE = Path(__file__).resolve()
_DEFAULT_REPO_ROOT = _HERE.parents[3]
_DEFAULT_EVOLVE_ROOT = _HERE.parents[2]
sys.path.insert(0, str(_HERE.parent))

from _common import cargo_build, run_binary  # noqa: E402
from _contract import ContractError, parse_plugin_stdout  # noqa: E402

CACHE_TYPES = ("assoc", "dash")
SIZES = (16, 32, 64)
RUN_TIMEOUT_S = 60


def _discover_traces(traces_dir: Path) -> list[Path]:
    """Return *.trace files in `traces_dir` sorted by name (deterministic order)."""
    if not traces_dir.is_dir():
        print(f"error: trace suite directory {traces_dir} does not exist", file=sys.stderr)
        sys.exit(1)
    traces = sorted(traces_dir.glob("*.trace"))
    if not traces:
        print(f"error: trace suite directory {traces_dir} contains no *.trace files", file=sys.stderr)
        sys.exit(1)
    return traces


def main():
    repo_root = Path(os.environ.get("REPO_ROOT", _DEFAULT_REPO_ROOT))
    evolve_root = Path(os.environ.get("EVOLVE_ROOT", _DEFAULT_EVOLVE_ROOT))
    suite = os.environ.get("TRACE_SUITE", "dev")
    traces_dir = evolve_root / "traces" / suite

    cargo_build(repo_root, ["--bin", "evolve-hit-rate"], label="evolve-hit-rate")
    binary = repo_root / "target" / "debug" / "evolve-hit-rate"

    traces = _discover_traces(traces_dir)

    values = []
    for trace in traces:
        for cache in CACHE_TYPES:
            for size in SIZES:
                label = f"evolve-hit-rate --trace {trace.name} --cache {cache} --size {size}"
                result = run_binary(
                    [str(binary), "--trace", str(trace), "--cache", cache, "--size", str(size)],
                    label=label,
                    timeout=RUN_TIMEOUT_S,
                )
                try:
                    record = parse_plugin_stdout(result.stdout, source=label)
                except ContractError as e:
                    print(f"error: {e}", file=sys.stderr)
                    sys.exit(1)
                for entry in record["values"]:
                    values.append({
                        "trace": trace.stem,
                        "cache": cache,
                        "size": size,
                        "hit_rate": entry["hit_rate"],
                    })

    # Summary excludes baseline (assoc) entries so the Pareto scalar reflects
    # the cache under test (Dash), not an arithmetic mix with an unrelated baseline.
    dash_rates = [v["hit_rate"] for v in values if v["cache"] == "dash"]
    if not dash_rates:
        print("error: no dash configurations produced a hit_rate", file=sys.stderr)
        sys.exit(1)
    summary = sum(dash_rates) / len(dash_rates)
    out = {
        "dim": "hit-rate",
        "values": values,
        "summary": round(summary, 6),
        "higher_is_better": True,
        "suite": suite,
    }
    print(json.dumps(out))


if __name__ == "__main__":
    main()
