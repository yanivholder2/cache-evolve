# Variant v003 Insight

## Helped

*(none confirmed — `per-op-time` was not evaluated for this variant)*

## Hurt

- `hit-rate`: Replacing `remove + push` with a single `swap(position, last)` breaks strict temporal ordering of intermediate bucket slots — the former MRU item at `len-1` is displaced into the middle of the list on every access, so the eviction victim at index 0 is no longer reliably the true least-recently-used item, causing the cache to evict warmer items and miss more requests.

## Takeaway

The O(n) `remove + push` in ClassicLRU is not pure overhead — it preserves the full recency gradient across every slot, and that ordering directly governs which item is sacrificed on eviction. Degrading it to an O(1) swap that only maintains the two boundary positions (index 0 = evict, index `len-1` = MRU) is sufficient to corrupt eviction quality measurably. The `per-op-time` gain the hypothesis targeted was not measured for this variant, so it remains unconfirmed whether the latency saving exists at all, let alone whether it compensates for the hit-rate regression.
