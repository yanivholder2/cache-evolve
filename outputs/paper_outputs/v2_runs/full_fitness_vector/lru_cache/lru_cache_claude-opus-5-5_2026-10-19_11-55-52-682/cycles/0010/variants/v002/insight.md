# Variant v002 Insight

## Helped

(none) No dimension improved. `per-op-time`'s summary (`get` p50) stayed flat, as expected: the adaptation only adds one masked compare and a bounded add or subtract on the `put` ghost-hit path, and `get` is unchanged.

## Hurt

- hit-rate: The adaptive `small_target` lost hit-rate against the fixed 10% split, so it failed the hypothesis's own bar. The likely cause is how the signal is built. Small-head evictions far outnumber SIEVE main evictions, so most entries in the shared half-size ghost carry the small tag. A ±1 step per hit with no ARC-style normalization by each source's ghost population then pushes `small_target` up toward `max_small` (40% of capacity). That shrinks main, which earlier cycles already showed to be the scarce resource on workload-a. The eval output does not record the trajectory of `small_target`, so whether it actually pinned to a bound is unconfirmed.
- metadata-size: The tracked summary (`Node<u64,u64>`) did not change. The `LruCache` struct still grew by the two new `usize` fields, `min_small` and `max_small`. The adaptation could have computed these bounds from `capacity` on the fly instead of storing them.
- per-op-time: `put` p99 rose a little and `get` p99 rose very slightly. This fits the extra tag test and branch on the ghost-hit path, but the change is within noise. Reserving bit 0 for the source tag also takes one bit away from the 32-bit fingerprint, which slightly raises the chance that two keys alias. That effect is too small to matter at this ghost size.

## Takeaway

Reusing the one shared ghost as a two-sided ARC signal fails on this trace. The two tags come from eviction streams of very different volume. A raw ±1 step therefore turns eviction volume into a steady push toward a larger small queue, not a measure of which queue was really too short. That push leads back to the wide-split regime that static tuning had already ruled out. To pursue adaptive sizing further, the step should be scaled by the relative frequency of each ghost source, as ARC does with `|B1|/|B2|`, or the two sources should be tracked separately. Raw tag counts in a shared table are biased by construction.
