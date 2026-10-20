## Recurring wins

- None yet: only one cycle completed, and it produced no Winner.

## Recurring failures

- Position-based LRU tweaks (partial promotion, rotate-based move-to-back) don't help. Weaker recency costs hit rate, and item movement doesn't dominate get latency.
- Geometry sweeps (wider buckets) give only marginal hit-rate gains, so associativity isn't the bottleneck on workload-a.
- Variants that add no state leave metadata size unchanged, so none can improve on the baseline's memory profile.

## Open questions

- Is capacity or the eviction policy the real hit-rate limit? Policies carrying a small extra signal (frequency counter, reference bit, ghost/admission filter) are untested.
- Where does get time go (hashing, bucket lookup, dispatch)? No variant has measured it.
- Why did put latency improve in every variant, including those that never touched the put path? It's probably a measurement artifact, so check whether it reproduces before trusting it.

## Per-policy notes

- LRU (baseline): fully promoting a hit on access matters in small buckets. Partial promotion makes a recently hit item the next victim.
- LRU with wider buckets: hit rate is nearly flat, and put latency looked better for unexplained reasons.
- No frequency-, reference-bit- or admission-based policy has been tried yet.
