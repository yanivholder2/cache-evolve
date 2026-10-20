# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Collapsing the residency probe and insert in `put` into one `HashMap::entry` call left hit-rate identical to the parent, as the hypothesis predicted, since eviction only visits keys already queued and the new key is inserted before any queue is touched. The per-op-time gain is still unmeasured: the fitness vector carries only hit-rate, so the single-hash saving cannot be credited yet. The change also calls `key.clone()` on every `put`, including hits, which the hypothesis did not account for and which may cancel the saving for heap-allocated keys. The implementation inserts through the vacant entry before the ghost check rather than after the evictions as H03 specified; this is sound for the same reason, but it departs from the written plan.
