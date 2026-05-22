"""Render a one-line summary of a FitnessVector's dimensions for insight.md."""

from __future__ import annotations


def format_dimensions(fv: dict) -> str:
    """Return e.g. 'hit-rate=0.65, per-op-time=95.0'."""

    return ", ".join(f"{d['dim']}={d['summary']:.4g}" for d in fv.get("dimensions", []))
