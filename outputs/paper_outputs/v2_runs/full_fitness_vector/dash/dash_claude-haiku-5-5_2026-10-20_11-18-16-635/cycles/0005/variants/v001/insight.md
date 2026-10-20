# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Sampling two slots per full bucket instead of scanning all of them means the evicted slot is often not the bucket's lowest-counter resident, so some of the frequency-aware retention v002 gained is given back.
- per-op-time (put): The two-choice victim search was meant to bound eviction work, but put latency rose at both p50 and p99 instead of falling toward the parent's level. The full scan was therefore not the main put-path cost, and the added sampling state (hash-derived pair or rotating cursor) probably costs about as much as the scan it replaced.

## Takeaway

Bounding the victim search to two comparisons did not make put cheaper, which shows that v002's put slowdown was not driven by scan length. The small hit-rate loss matches the expected effect of evicting by sampled rather than true minimum counter. The overlay source could not be read in this run, so the attribution to the sampling logic versus the extra cursor or hashing is unverified. The get p99 increase is not explained by the hypothesis, which left the get path unchanged, so it is most likely measurement noise.
