#!/usr/bin/env python3
"""Deterministic bounded-Zipfian key trace in lirs format (one key per line).

Seeded, so the re-baseline and the cohort run read the identical byte stream —
a fair, reproducible workload (caffeine's own synthetic generator draws from an
unseeded ThreadLocalRandom and would give every variant a different trace).
"""

import bisect
import gzip
import random
from pathlib import Path

SEED = 42
ITEMS = 25000  # distinct keys 0..ITEMS-1 (cache 2048 / 128 buckets = 16-way; ~8.2% coverage)
EVENTS = 500_000  # number of requests
SKEW = 0.99  # zipfian exponent; higher = more skewed
OUT = Path(__file__).resolve().parents[1] / "inputs/traces/workload-a.trace.gz"

# Cumulative distribution over ranks: P(rank i) is proportional to 1/(i+1)**SKEW
weights = [1.0 / (i + 1) ** SKEW for i in range(ITEMS)]
total = sum(weights)
cdf: list[float] = []
acc = 0.0
for weight in weights:
    acc += weight / total
    cdf.append(acc)

rng = random.Random(SEED)
OUT.parent.mkdir(parents=True, exist_ok=True)
with gzip.open(OUT, "wt") as out:
    for _ in range(EVENTS):
        key = bisect.bisect_left(cdf, rng.random())
        out.write(f"{min(key, ITEMS - 1)}\n")

print(f"wrote {EVENTS} events over {ITEMS} items (skew {SKEW}) into {OUT}")
