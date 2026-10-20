# Variant v002 Insight

## Helped

- (none) — narrowing the seven scalar control fields to `u32` shrank `size_of::<LruCache<u64,u64>>()` by packing the control block with no interior padding, but that struct is not the scored footprint, so no scored dimension improved.

## Hurt

- per-op-time: get p50 ticked up despite the get fast path being byte-identical and casts confined to the insert branch (`self.nodes.len() >= self.capacity as usize`); the predicted put p50 drop never appeared, so put held flat.

## Takeaway

Shrinking the `LruCache` control struct in the cheap direction did not buy the put p50 win the footprint-to-p50 coupling predicted — put stayed flat and get even nudged up by a hair. The coupling that moved put p50 in earlier cycles was not causal-and-symmetric on the cache struct itself: trimming its scalar fields here is latency-neutral at best, and because the scored metadata-size tracks `Node` (untouched at 24) rather than the cache struct, the saved bytes don't register as a win.
