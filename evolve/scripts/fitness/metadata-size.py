#!/usr/bin/env python3
"""Metadata-size fitness plugin.

Runs the evolve-metadata-size binary, validates its output against the plugin
contract, and forwards it.

Environment:
  REPO_ROOT   path to the dash-cache-rs repo (default: 3 levels up from this file)
"""

import os
import sys
from pathlib import Path

_HERE = Path(__file__).resolve()
_DEFAULT_REPO_ROOT = _HERE.parents[3]
sys.path.insert(0, str(_HERE.parent))

from _common import cargo_build, run_binary  # noqa: E402
from _contract import ContractError, parse_plugin_stdout  # noqa: E402

RUN_TIMEOUT_S = 60


def main():
    repo_root = Path(os.environ.get("REPO_ROOT", _DEFAULT_REPO_ROOT))
    cargo_build(repo_root, ["--bin", "evolve-metadata-size"], label="evolve-metadata-size")
    binary = repo_root / "target" / "debug" / "evolve-metadata-size"

    result = run_binary([str(binary)], label="evolve-metadata-size", timeout=RUN_TIMEOUT_S)
    try:
        parse_plugin_stdout(result.stdout, source="evolve-metadata-size")
    except ContractError as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    sys.stdout.write(result.stdout)


if __name__ == "__main__":
    main()
