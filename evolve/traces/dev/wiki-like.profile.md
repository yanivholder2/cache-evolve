---
trace: wiki-like
suite: dev
hidden: false
generator: wiki-like
seed: 103
events: 10000
key_universe: 500
churn_window: 1000
signed_off_by:
---

# wiki-like

High-skew Zipf (α=1.1) with a *rotating* hot set: every 1000 events the
permutation that maps keys to popularity ranks shifts by one position, so
yesterday's hot keys cool off and new keys heat up. Coarse approximation of
the diurnal/event-driven churn seen in Wikipedia request logs.

## Workload characteristics

- **Skew**: very heavy-headed (α=1.1) at any instant in time.
- **Working-set size**: 500 keys total; instantaneous hot-set ~20–30.
- **Scan behavior**: none, but hot-set membership changes over time.
- **Temporal locality**: high within a churn window, weakening across windows.

## Why it's in the dev suite

Penalizes policies that lock onto initial winners. Variants that adapt
quickly to membership changes (recency-biased) should beat strict
frequency-biased ones here.
