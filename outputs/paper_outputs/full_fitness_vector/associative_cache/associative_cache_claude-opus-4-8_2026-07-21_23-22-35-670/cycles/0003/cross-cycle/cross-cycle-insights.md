# Cross-cycle insights

## Recurring wins

- Trimming dead metadata off the eval path is the most reliable move: it
  peels off field-by-field across every cycle, monotonic and cheap, with no
  hit-rate cost — only tail-latency noise.
- Proven mechanisms compose additively when they touch disjoint code:
  scan-resistant reuse-protection and slim item layout stacked with zero
  interaction and zero extra bytes.
- The size metric weighs only the item, so state parked off the measured
  item (e.g. in the bucket struct) is free on the size axis — this
  repeatedly dissolves the old hit-rate-vs-metadata antagonism.
- Stateless probationary-front insertion buys scan resistance from
  geometry alone, hitting hit-rate targets at the metadata floor with
  cleaner tails than stateful equivalents.

## Recurring failures

- Frequency/LFU victim signals lose to plain recency-front every time they
  are tried (second-chance aging, LFU tiebreak) — recency is the better
  signal on this trace.
- Weakening permanent binary protection (self-clearing/second-chance/CLOCK
  aging) sheds immunity too fast and collapses back toward plain LRU;
  protection is the feature, not a bug.
- Plain associativity increases and bucket widening tax every operation via
  a longer linear scan for little-to-no conflict-miss headroom.

## Open questions

- Where is the metadata floor — how far can fields shrink before colliding
  with one the eval path actually reads?
- Can the stateful bucket-bit reuse path shed its quadratic key-matched
  rebuild via index-preserving swaps and then match the stateless
  geometric path's clean tails?
- Does stateless probation leave any hit-rate on the table versus explicit
  reuse tracking?
- Is there a way to bound how long a hot item stays protected without
  shedding protection too fast?
- The recurring upward p99 tail drift under flat p50 on metadata cuts —
  structural, or persistent measurement noise?

## Per-policy notes

- LRU / recency-front: the consistent baseline to beat; front-based victim
  choice wins across cycles.
- Permanent binary reuse-protection: the right fit for this trace;
  refinements should preserve it, not weaken it.
- Second-chance / CLOCK-style aging: fails — collapses toward LRU.
- Frequency / LFU: worse victim signal than recency in every probe.
- Bucket geometry: probationary-front insertion is the cheap, stateless
  incumbent; wide-bucket associativity plays are structurally expensive due
  to linear scan cost.
