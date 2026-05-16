#!/usr/bin/env python3
"""Render a one-line summary of a FitnessVector's dimensions for insight.md.

Usage: dim-summary.py <fitness-vector.json>
"""

import json
import sys
from pathlib import Path


def main():
    if len(sys.argv) != 2:
        print("usage: dim-summary.py <fitness-vector.json>", file=sys.stderr)
        sys.exit(1)
    path = Path(sys.argv[1])
    try:
        fv = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as e:
        print(f"error: reading {path}: {e}", file=sys.stderr)
        sys.exit(1)
    parts = []
    for d in fv.get("dimensions", []):
        try:
            parts.append(f"{d['dim']}={d['summary']:.4g}")
        except KeyError as e:
            print(f"error: dimension in {path} missing key {e}", file=sys.stderr)
            sys.exit(1)
    print(", ".join(parts))


if __name__ == "__main__":
    main()
