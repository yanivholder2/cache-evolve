# Variant v001 Insight

## Helped

- per-op-time: Replacing the O(n) `position()` linear scans in both `get()` and `put()` with O(1) `HashMap` lookups (`data` for values, `freq_map` for frequencies) eliminated the dominant bottleneck identified in the parent, collapsing get p50 to zero and cutting put tail latency by roughly 6×.
- metadata-size: Stripping the `value: V` field from `Entry<K,V>` and relocating values to the side `data` HashMap halved the per-entry deque footprint from 16 bytes to 8 bytes, since entries now carry only a key and a `PhantomData` marker.

## Hurt

## Takeaway

Swapping O(n) deque scans for O(1) HashMap lookups was a clean win: latency fell dramatically across all percentiles while hit rate and eviction semantics remained identical to the parent. The tradeoff is that the `S3Fifo` struct itself grew by 32 bytes to accommodate the two new maps, but this per-instance overhead is amortized over the full working set and did not affect any measured fitness dimension. The hypothesis's core claim — that `position()` was the binding latency bottleneck — is fully confirmed.
