---
trace: scan-mixed
suite: dev
hidden: false
generator: scan-mixed
seed: 105
events: 10000
hot_universe: 100
scan_universe: 400
scan_every: 500
signed_off_by:
---

# scan-mixed

A 100-key hot set, interrupted every 500 accesses by a 400-key sequential
scan over a disjoint cold range. Each scan flushes a naive LRU cache and
then the hot set must be re-warmed.

## Workload characteristics

- **Skew**: uniform within hot set; scans are sequential by construction.
- **Working-set size**: 100 hot + 400 scan = 500 distinct keys total.
- **Scan behavior**: heavy. Roughly 4% of total ops are scan; 4% × 8 scans
  per trace ≈ 3200 scan accesses out of 10000.
- **Temporal locality**: very high *between* scans; zero *during* scans.

## Why it's in the dev suite

The textbook scan-resistance benchmark. Policies that protect a "probation"
or "filter" tier from being trampled by scans (S3-FIFO, SLRU, segmented
admission) should beat plain LRU by a wide margin.
