# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three baseline-derived variants each attacked a different fitness axis —
geometry (set-associativity), eviction policy (scan resistance), and item
metadata size — with no clean Winner emerging. Every hit-rate gain was
paid for with a per-op-time regression, and the only unambiguously
positive move was cutting dead metadata.

## Patterns

- Hit-rate and per-op-time moved as an antagonistic pair: both the wider-
  bucket geometry (v001) and the reuse-protection eviction tweak (v002)
  bought hit rate by lengthening a linear bucket/eviction scan, so neither
  is a free win.
- The bucket is scanned linearly, so anything that widens buckets or turns
  an O(1) eviction pick into a front-to-back scan directly taxes every
  operation — scan cost is the dominant hidden price in this cache.
- Metadata carried only by policies that are off the eval path (the
  `ClassicLRU` path never touches timestamps) is pure dead weight and can
  be cut with zero hit-rate cost, unlike the two hit-rate plays.
- Conflict-miss headroom at this working-set size appears thin: aggressively
  raising associativity barely moved hit rate.

## Open questions

- Can scan resistance be gained without converting eviction into a linear
  scan — e.g. maintaining the eviction candidate incrementally rather than
  re-scanning?
- Is there a bucket width that captures the collision headroom without the
  full scan-cost penalty, or is the trade strictly unfavorable here?
- Was v003's p50 drift real or measurement noise? Timing signals this
  cycle were small enough to warrant caution.

## Continuation

Pursue metadata trimming and other "cut dead weight off the eval path"
moves — they are cheap and monotonic. Treat both bucket-widening and
scan-based eviction protection as expensive: only revisit them with an
O(1)-maintained variant that avoids adding a linear scan. Drop plain
associativity increases at this working-set size. Be wary of small per-op
timing deltas; prefer changes with a clear structural reason to move.


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
