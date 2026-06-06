# Variant v002 Insight

## Helped

- hit-rate: raising `MAX_FREQ` from 3 to 7 lets the highest-reuse main entries bank more second chances, so they survive extra `evict_main` decrement passes and the narrow `capacity / 16` main queue retains a few more reaccessed keys on workload-a.

## Hurt

- (none) metadata-size is untouched — `freq` already lived in `Entry`'s existing `u8`, so a higher ceiling changes a stored value, not struct layout. per-op-time only jittered (put p99 moved within noise); `get`'s `.min(MAX_FREQ)` clamp is the same single comparison regardless of the ceiling.

## Takeaway

On the best-so-far narrow-split parent, the freq-ceiling raise still pays — the two levers are orthogonal, not interfering like split-and-ghost did, since durability (how long a proven key survives) is independent of capacity allocation (how many proven keys fit). But the combined gain is marginal and smaller than the split sweep gave, so deeper retention is nearing the point where extra second chances protect keys that workload-a no longer reaccesses.
