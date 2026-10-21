# Variant v003 Insight

I could not list or read the overlay source files in this session. This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- hit-rate: The segmented LRU keeps first-touch keys in a probation segment of slots/8 positions, which is 4 of 32 ways in the 4x-wide buckets. A key therefore has to be re-referenced before it can displace a protected entry. This filters one-shot keys better than the baseline's plain per-bucket replacement and clearly improves hit-rate on workload-a.

## Hurt

- metadata-size: v003's fitness vector has no entry for this dimension, so it was not measured. The design adds no per-entry state, so any regression is likely small, but the data can't confirm it.
- per-op-time: This dimension is also missing from v003's vector. Scanning 32 ways per bucket and shifting entries between segments on probation hits probably costs more per operation than the baseline, but nothing was measured.

## Takeaway

Wider buckets combined with a lean positional probation/protected split raise hit-rate without ghost lists or counters. This supports the idea that scan-resistance comes mainly from segmenting, even when the segments are only positions inside a bucket. The vector does not compare slots/8 against the slots/16 setting from cycle 0008 v002, so it cannot tell us whether the optimal probation fraction moves upward at higher associativity. The missing metadata-size and per-op-time dimensions also leave the cost of the 4x associativity unmeasured.
