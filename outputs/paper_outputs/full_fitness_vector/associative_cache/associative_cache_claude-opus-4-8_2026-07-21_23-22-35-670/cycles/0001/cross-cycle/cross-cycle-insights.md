## Recurring wins

- Cutting dead weight off the eval path (e.g. trimming metadata that only
  off-path policies touch) is cheap and monotonic: hit rate holds while
  per-op cost drops.

## Recurring failures

- Buying hit rate by lengthening a linear scan (wider buckets, scan-based
  eviction protection) keeps taxing every operation — hit-rate and per-op
  time move as an antagonistic pair, so these are never free wins.
- Plain associativity increases barely move hit rate at this working-set
  size; conflict-miss headroom looks thin.

## Open questions

- Can scan resistance be gained without turning eviction into a linear
  scan — e.g. maintaining the eviction candidate incrementally in O(1)?
- Is there a bucket width that captures collision headroom without the full
  scan-cost penalty, or is the trade strictly unfavorable here?
- How much of the small per-op timing deltas is real signal vs measurement
  noise? Prefer changes with a clear structural reason to move.

## Per-policy notes

- ClassicLRU: does not touch timestamp metadata, so that metadata is pure
  dead weight on this path and safe to trim.
- Scan-resistant / reuse-protection eviction: gains scan resistance only by
  converting an O(1) eviction pick into a front-to-back scan — revisit only
  with an O(1)-maintained variant.
