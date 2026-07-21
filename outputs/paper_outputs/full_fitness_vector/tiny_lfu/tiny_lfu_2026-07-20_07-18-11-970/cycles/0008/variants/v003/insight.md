# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Adding a second `sketch.increment` on confirmed hits did not widen the admission gate in a useful direction — resident key counts grew faster but so did their sketch-estimated victims, leaving the relative comparison largely unchanged and slightly degrading overall admission quality.
- per-op-time: Cache hits now execute two full sketch increment calls instead of one, doubling the hash-plus-four-counter-write cost on the hot path; because the workload is ~74% hits, this overhead affects the majority of `get` operations, visibly doubling both p50 and p99 latency.

## Takeaway

Doubling the per-hit frequency signal steepens the gradient the hypothesis intended, but when the admission gate compares an incoming cold key against the LRU victim (itself a resident that also receives the double increment), the relative ordering is not reliably improved — so the hit-rate does not benefit. The cost is concrete and immediate: every cache hit now pays for two sketch writes instead of one, making this change a net loss on both dimensions it touched.
