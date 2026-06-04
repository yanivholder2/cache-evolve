# Variant v002 Insight

## Helped

- metadata-size: Repurposing the parent's per-bucket `Cell<u8>` into a `Cell<u16>` ref-bitmask kept the new aging state entirely on `AssociativeCacheBucket`, so `Item<u64,u64>` stays at 40 bytes and `AssociativeCache<u64,u64>` is unchanged — confirming the "per-bucket vehicle is metadata-flat" claim the parent established.

## Hurt

- hit-rate: The mass-clear sweep destroys the long-lived "ever touched" signal that `lfu_counter > 0` carried for free: once every resident slot has been hit at least once, one eviction sweep zeroes every bit and the bucket falls back to plain leftmost-eviction, throwing away frequency information about hot items that the v003 filter would have kept protecting. On a LIRS-style workload where re-reference distances span many evictions, periodically wiping the touched/untouched discrimination is strictly worse than letting `lfu_counter` accumulate, so the bitmask actually trails the parent's cursor variant rather than recovering toward v003.

## Takeaway

Aging a per-bucket signal "for free" via en-masse clear is not free when the signal it replaces (`lfu_counter`) was already monotone and durable: the clear synchronously erases protection on every hot slot at once, collapsing the policy to LRU-on-position until the next round of touches rebuilds the bitmask. For Dash bucket-movement, temporal aging only pays off when the underlying counter would otherwise saturate or lie — against an unsaturated `lfu_counter` filter, replacing "touched ever" with "touched since last sweep" demotes hot residents into eviction candidates immediately after the sweep and loses hits the parent's signal would have kept.
