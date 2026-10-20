# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three kinds of change: recency rotation of rejected admission victims (v001), a constant-factor guard that skips relinks at the MRU tail (v002), and a probation/protected segmentation of the resident list (v003). The two Winners are v002, a cost-only change with an unchanged hit rate, and v003, a small hit-rate gain that adds metadata.

## Patterns

- Rotating a rejected victim to the MRU tail hurt both hit rate and per-op time. The frequency-based admission gate already did the protective work, so recency rotation added cost without improving admission.
- Changes that leave eviction order alone and only trim pointer work on repeated accesses to the newest key are a safe way to gain speed.
- Segmentation produced the only hit-rate movement, but it is small and paid for with per-node and per-cache metadata growth.
- Every variant's evidence comes from one trace (lirs workload-a), so none of the Winners' gains has been shown to generalize.

## Open questions

- Does v003's marginal hit-rate gain hold on other traces, or is it specific to lirs workload-a?
- v003's latency drop is unexplained by its added bookkeeping and may be measurement variance. Is it real?
- How much of v002's saving depends on the share of accesses that hit the newest key?

## Continuation

Pursue further constant-factor, eviction-order-preserving guards like v002, and re-test v003's segmentation on at least one additional trace before building on it. Drop recency rotation of rejected victims (v001) as a direction.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
