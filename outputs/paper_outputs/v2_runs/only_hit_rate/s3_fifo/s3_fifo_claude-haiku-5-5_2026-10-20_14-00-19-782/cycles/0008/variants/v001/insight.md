# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Swapping SipHash for a multiplicative hasher on the `resident` index left hit-rate unchanged from the parent, which is what the hypothesis predicted: the index is never iterated and eviction runs off the `small`/`main` queues and `freq`, so the hasher cannot reorder evictions. The speed claim is untested, though. This variant's fitness vector has no per-op-time or metadata-size entry, so the constant-factor gain on `get`/`put` cannot be credited or refuted yet. Even if it lands, the gain is bounded: `put` on a miss still runs a linear `ghost.iter().any` and `ghost.retain` scan, which this change does not touch.
