# Variant v003 Insight

## Helped

_None—no dimension improved meaningfully._

## Hurt

- **per-op-time (get)**: Back-insert landing at MRU incurs insertion overhead (2 ns penalty) that outweighs any amortized throughput gain from faster promotion paths.

- **per-op-time (put)**: MRU-insertion cost extends to put operations (1 ns penalty), confirming that the overhead is fundamental to the back-insert mechanism, not specific to get.

## Takeaway

Back-insert promotion fails to improve on 75/25 geometry despite being the most aggressive hash-indexed allocation tested. The hypothesis that composition succeeds at the boundary of large primary-tier capacity is disproven: the MRU-insertion overhead consistently dominates, and the already-sufficient normal-bucket allocation (27 of 32 buckets) provides no absorption for the extra cost. Hit-rate barely moves within measurement noise, while latency regresses. This confirms that simple geometry-only variants remain optimal, and back-insert composition is a net loss across all tested concentrations.
