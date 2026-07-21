# Variant v001 Insight

## Helped

(none)

## Hurt

(none)

## Takeaway

Pre-sizing `data`, `freq_map`, and `ghost_set` at 2× capacity had zero measurable effect on any fitness dimension: hit-rate, metadata-size, and per-op-time are all identical to the parent. This rules out hash-table load factor as the source of the per-op-time regression that the 1/80 small queue fraction introduced in cycle/0006 — the bottleneck is the sheer call-rate of `evict_small()` driven by the tighter queue, not probe-chain depth inside the hashmaps. Reducing contention in the allocations is inert when the dominant cost is invocation frequency, not per-call hash overhead.
