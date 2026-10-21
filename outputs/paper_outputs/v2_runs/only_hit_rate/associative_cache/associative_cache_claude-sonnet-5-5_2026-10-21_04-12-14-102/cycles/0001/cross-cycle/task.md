# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants tried: wider buckets (geometry, v001), cheaper one-step hit promotion (v002), and 2Q-style half-promotion for scan resistance (v003). Only the geometry change (v001) won, with a small hit-rate gain. Both policy-tweak variants slightly lowered hit rate versus plain move-to-back LRU.

## Patterns

- Only hit-rate was reliably measured. Per-op-time and metadata-size were missing from every variant's fitness vector, so the speed and size claims in all three hypotheses went untested.
- Both variants that weakened promotion in short buckets (v002, v003) lost hit rate. Full move-to-back recency seems hard to beat at small associativity on this LIRS trace.
- The geometry gain was marginal, so hash imbalance across sets is a minor source of misses. Most misses look like capacity or policy misses.
- Winning by a small hit-rate margin is unproven, because the likely timing cost of 64-way scans and shifts was never checked.

## Open questions

- Why was per-op-time not reported, and can the evaluation be fixed so speed-targeting hypotheses can be judged?
- Does the wider-bucket gain survive once get/remove/push cost is measured?
- Would scan resistance work with a separate probationary region instead of a shallower promotion within one short bucket?

## Continuation

Build on v001's geometry, but first make sure per-op-time and metadata-size are measured so trade-offs can be judged. Drop promotion-weakening ideas (transposition, midpoint) in plain LRU buckets. Next, try policies that change eviction or admission (e.g. a separate probationary segment, frequency-aware victim choice) rather than hit-path promotion.


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
