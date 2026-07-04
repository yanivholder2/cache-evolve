# Variant v003 Insight

## Helped

- (none): The protected cap with LRU demotion left hit-rate exactly where the unbounded parent stood, so capping protection neither helped nor hurt the outcome the change was aimed at.

## Hurt

- per-op-time: Every probationary promotion now runs an extra `filter(...).count()` plus a `position(...)` scan to check the cap and find the protected LRU victim — a per-hit linear pass over the bucket on top of the existing victim scan, with no offsetting gain (not in the vector, but a structural cost of the demotion logic).

## Takeaway

On this workload the protected segment never grew past `(max_size*4)/5`, so the cap-and-demote branch effectively never fired and the bounded SLRU behaved identically to v003's unbounded version — the hypothesized starvation of probation by a fully-proven working set does not occur here. The canonical capped-SLRU fix is inert until protected saturation actually happens, so it only adds promotion-path scan cost on this trace.
