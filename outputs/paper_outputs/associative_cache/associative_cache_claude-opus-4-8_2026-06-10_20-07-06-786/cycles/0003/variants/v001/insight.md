# Variant v001 Insight

## Helped

- hit-rate: Replacing the position-based tiebreak with a live `(lfu_counter, timestamp)` composite and dropping the per-hit move-to-back held hit-rate exactly flat versus the parent — the simpler, reorder-free path costs nothing in retention quality on this trace.

## Hurt

(none — no measured dimension regressed.)

## Takeaway

On this collision-bound trace the equal-frequency tiebreak granularity is inert: true last-access recency (`timestamp`) and the parent's vector-position proxy pick the same victims, so hit-rate is identical. The change's real value is structural — it removes the O(bucket) per-hit shuffle while preserving the frequency signal that drives the win — but with only hit-rate measured here, that per-op-time benefit is unobserved.
