# Variant v001 Insight

## Helped

- metadata-size: Undoing the 4x width multiplier shrinks the sketch's heap allocation to a quarter. The `Node` size that the summary tracks did not change, so the saving only shows in the per-type breakdown.

## Hurt

- hit-rate: Going back to the narrower sketch means more unrelated keys share counters, which slightly inflates frequency estimates in the strict-greater admission test. The loss is small and about what the hypothesis predicted (within ~0.001 of the parent).
- per-op-time: `get` and `put` p50 and p99 all got slower, even though the hypothesis predicted a speedup from a more cache-friendly table. The sketch is 16 KiB at this size, and the 64 KiB version was already fast enough. Shrinking it removed no real locality cost. The slowdown is more likely measurement noise or an environment effect, since the cheaper-looking change cannot explain it. This also suggests the parent's per-op-time gain was noise, which fits the doubt in the parent's insight.

## Takeaway

Sketch width is a weak lever in both directions. Narrowing it saves hidden memory and costs a hair of hit-rate, but it did not deliver the predicted latency win. Treat per-op-time differences of this size as noise unless repeated, and look to the admission policy or the hot-path work for real gains.
