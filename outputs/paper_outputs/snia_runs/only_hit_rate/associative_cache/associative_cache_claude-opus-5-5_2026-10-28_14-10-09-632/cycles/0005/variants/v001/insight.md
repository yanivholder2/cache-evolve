# Variant v001 Insight

## Helped

- hit-rate: hm_0 at 16384 gained the most of any cell. In 128-slot buckets, hm_0 blocks whose second touch comes just after their `small_len` probation window are now caught by the 96-entry ghost. They re-enter with `lfu_counter = 1` and get promoted into main, where v002 had thrown them away as one-touch.
- hit-rate: hm_0 at 131072 moved up slightly. The ghost is consistent with hm_0's reuse pattern at both sizes. But 1024-slot buckets already gave hm_0 enough probation depth under v002, so there was little left to gain.

## Hurt

- hit-rate: web_0 at 131072 had the largest regression, which undoes part of v002's scan-filtering gain. A 768-entry ghost remembers probation evictions for a long time. So web_0 blocks that come back once and are never used again still match the ghost. They skip the "hit during probation" test, get promoted, and each promotion evicts a main-region victim and runs the GCLOCK sweep, which decrements counters on blocks that really are reused.
- hit-rate: web_0 at 16384 is the cell the hypothesis targeted, and it got slightly worse instead of better. It missed the refutation threshold. Pre-marking returning keys does not help there. The late second touches that ghost admission catches are not followed by further reuse often enough to be worth the main slot they displace.

## Takeaway

On this lineage, a main-sized ghost that marks returning keys with counter 1 is trace-dependent. It recovers near-miss reuse on hm_0, but on web_0 it re-admits single-return blocks that v002's probation filter was correctly rejecting, so it costs web_0 at both sizes. That refutes the hypothesis, whose test was web_0 at 16384. A ghost match is weaker evidence than a hit during probation, so promoting on a ghost match alone (and resetting the counter to 0 on promotion) adds churn in main. Per-op-time was requested but is not in the fitness vector. The extra cost is one linear ghost scan per miss, up to 768 compares in large buckets.
