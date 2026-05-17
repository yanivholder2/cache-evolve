#!/usr/bin/env python3
"""Deterministic generator for evolve trace suites.

Produces text trace files under `evolve/traces/{dev,holdout}/`. Each trace is
plain text: header lines starting with `#` (ignored by the replayer) followed
by one decimal `u64` key per non-empty line.

The generator is deterministic — re-running it yields byte-identical files —
so regenerating only changes traces when this script's parameters change.

Usage: gen-traces.py [--evolve-root <path>] [--suite dev|holdout|all]
"""

import argparse
import math
import random
from pathlib import Path

DEFAULT_EVENTS = 10_000


def _zipf_keys(rng: random.Random, *, alpha: float, key_universe: int, events: int) -> list[int]:
    """Zipfian sampler over keys 1..=key_universe."""
    weights = [1.0 / (k ** alpha) for k in range(1, key_universe + 1)]
    return rng.choices(range(1, key_universe + 1), weights=weights, k=events)


def _wiki_like_keys(rng: random.Random, *, key_universe: int, events: int, churn_window: int) -> list[int]:
    """High-skew Zipfian with gradual hot-set shift, mimicking Wikipedia popularity churn."""
    alpha = 1.1
    base_weights = [1.0 / (k ** alpha) for k in range(1, key_universe + 1)]
    out = []
    for i in range(events):
        shift = (i // churn_window) % key_universe
        keys = [((k - 1 + shift) % key_universe) + 1 for k in range(1, key_universe + 1)]
        out.append(rng.choices(keys, weights=base_weights, k=1)[0])
    return out


def _oltp_keys(rng: random.Random, *, hot_universe: int, cold_universe: int, events: int, hot_prob: float) -> list[int]:
    """OLTP-class: small hot working set + sporadic cold-key insertions."""
    out = []
    for _ in range(events):
        if rng.random() < hot_prob:
            out.append(rng.randint(1, hot_universe))
        else:
            out.append(rng.randint(hot_universe + 1, hot_universe + cold_universe))
    return out


def _scan_mixed_keys(rng: random.Random, *, hot_universe: int, scan_universe: int, events: int, scan_every: int) -> list[int]:
    """Hot set with periodic full scan bursts."""
    out = []
    i = 0
    while i < events:
        if i > 0 and i % scan_every == 0:
            scan_len = min(scan_universe, events - i)
            out.extend(range(hot_universe + 1, hot_universe + 1 + scan_len))
            i += scan_len
        else:
            out.append(rng.randint(1, hot_universe))
            i += 1
    return out[:events]


def _uniform_keys(rng: random.Random, *, key_universe: int, events: int) -> list[int]:
    return [rng.randint(1, key_universe) for _ in range(events)]


def _scan_burst_keys(rng: random.Random, *, hot_universe: int, scan_universe: int, events: int, burst_length: int, burst_every: int) -> list[int]:
    """Hot set punctuated by long sequential scan bursts (more adversarial than scan-mixed)."""
    out = []
    i = 0
    while i < events:
        if i > 0 and i % burst_every == 0:
            scan = list(range(hot_universe + 1, hot_universe + 1 + scan_universe))
            for _ in range(burst_length // scan_universe + 1):
                if i >= events:
                    break
                for k in scan:
                    out.append(k)
                    i += 1
                    if i >= events:
                        break
        else:
            out.append(rng.randint(1, hot_universe))
            i += 1
    return out[:events]


def _write_trace(path: Path, header: dict[str, str | int | float], keys: list[int]) -> None:
    lines = [f"# {key}: {value}" for key, value in header.items()]
    lines.append("")
    lines.extend(str(k) for k in keys)
    path.write_text("\n".join(lines) + "\n")


def _gen_dev(traces_root: Path) -> None:
    dev = traces_root / "dev"
    dev.mkdir(parents=True, exist_ok=True)

    rng = random.Random(101)
    _write_trace(
        dev / "zipf-small.trace",
        {"trace": "zipf-small", "generator": "zipfian", "alpha": 0.99, "seed": 101, "events": DEFAULT_EVENTS, "key_universe": 200},
        _zipf_keys(rng, alpha=0.99, key_universe=200, events=DEFAULT_EVENTS),
    )

    rng = random.Random(102)
    _write_trace(
        dev / "zipf-large.trace",
        {"trace": "zipf-large", "generator": "zipfian", "alpha": 0.85, "seed": 102, "events": DEFAULT_EVENTS, "key_universe": 2000},
        _zipf_keys(rng, alpha=0.85, key_universe=2000, events=DEFAULT_EVENTS),
    )

    rng = random.Random(103)
    _write_trace(
        dev / "wiki-like.trace",
        {"trace": "wiki-like", "generator": "wiki-like", "alpha": 1.1, "seed": 103, "events": DEFAULT_EVENTS, "key_universe": 500, "churn_window": 1000},
        _wiki_like_keys(rng, key_universe=500, events=DEFAULT_EVENTS, churn_window=1000),
    )

    rng = random.Random(104)
    _write_trace(
        dev / "oltp.trace",
        {"trace": "oltp", "generator": "oltp", "seed": 104, "events": DEFAULT_EVENTS, "hot_universe": 150, "cold_universe": 2000, "hot_prob": 0.92},
        _oltp_keys(rng, hot_universe=150, cold_universe=2000, events=DEFAULT_EVENTS, hot_prob=0.92),
    )

    rng = random.Random(105)
    _write_trace(
        dev / "scan-mixed.trace",
        {"trace": "scan-mixed", "generator": "scan-mixed", "seed": 105, "events": DEFAULT_EVENTS, "hot_universe": 100, "scan_universe": 400, "scan_every": 500},
        _scan_mixed_keys(rng, hot_universe=100, scan_universe=400, events=DEFAULT_EVENTS, scan_every=500),
    )


def _gen_holdout(traces_root: Path) -> None:
    holdout = traces_root / "holdout"
    holdout.mkdir(parents=True, exist_ok=True)

    rng = random.Random(201)
    _write_trace(
        holdout / "zipf-medium.trace",
        {"trace": "zipf-medium", "generator": "zipfian", "alpha": 0.95, "seed": 201, "events": DEFAULT_EVENTS, "key_universe": 800},
        _zipf_keys(rng, alpha=0.95, key_universe=800, events=DEFAULT_EVENTS),
    )

    rng = random.Random(202)
    _write_trace(
        holdout / "scan-burst.trace",
        {"trace": "scan-burst", "generator": "scan-burst", "seed": 202, "events": DEFAULT_EVENTS, "hot_universe": 120, "scan_universe": 600, "burst_length": 1200, "burst_every": 800},
        _scan_burst_keys(rng, hot_universe=120, scan_universe=600, events=DEFAULT_EVENTS, burst_length=1200, burst_every=800),
    )

    rng = random.Random(203)
    _write_trace(
        holdout / "uniform.trace",
        {"trace": "uniform", "generator": "uniform", "seed": 203, "events": DEFAULT_EVENTS, "key_universe": 1500},
        _uniform_keys(rng, key_universe=1500, events=DEFAULT_EVENTS),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate evolve trace suites")
    parser.add_argument("--evolve-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--suite", choices=("dev", "holdout", "all"), default="all")
    args = parser.parse_args()

    traces_root = args.evolve_root / "traces"
    if args.suite in ("dev", "all"):
        _gen_dev(traces_root)
    if args.suite in ("holdout", "all"):
        _gen_holdout(traces_root)


if __name__ == "__main__":
    main()
