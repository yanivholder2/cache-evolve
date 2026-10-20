# Variant v003 Insight

## Helped

- metadata-size: Narrowing the arena links (`prev`/`next`, `head`/`tail`, and the `HashMap` values) from `usize` to `u32` removes four bytes per link field, which is the entire reduction in `Node<u64,u64>` and the smaller `LruCache` header.

## Hurt

- per-op-time: The `get` median got slower even though the eviction and move-to-MRU logic is unchanged, which points at the cost of the `as usize` conversions at every arena access site rather than at any algorithmic change.

## Takeaway

Shrinking the index width is a clean, policy-neutral way to cut per-node metadata, and hit-rate stayed put as the hypothesis predicted. The cost shows up on the hot path: each arena access now pays a width conversion, so the memory win did not translate into a speed win for `get`. Any follow-up should keep the narrow indices but check whether the casts are avoidable at the hottest call sites.
