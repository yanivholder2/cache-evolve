# Variant v003 Insight

## Helped

- **hit-rate**: Pushing to 75% normal / 25% stash allocation maximized the hash-indexed capacity, allowing the hot-set to be caught more reliably in the primary tier on Zipfian workloads where item access follows power-law distribution.

- **per-op-time (get)**: Direct hash indexing in the normal tier is faster than linear probing through the stash; with 75% of buckets now hash-indexed, the average lookup path is shorter for both hits and misses.

- **per-op-time (put)**: Increased normal-bucket capacity reduces insertion conflicts and stash overflows, lowering the amortized cost of put operations.

## Hurt

_None—all dimensions improved or remained neutral._

## Takeaway

The 75/25 geometry concentration continues the improvement trend from earlier cycles, disproving the hypothesis that 62.5/37.5 represented the asymptote. Hit-rate, get latency, and put latency all improved simultaneously, confirming that further prioritization of hash-indexed capacity over stash capacity remains beneficial on this workload, at least to the 75/25 point.
