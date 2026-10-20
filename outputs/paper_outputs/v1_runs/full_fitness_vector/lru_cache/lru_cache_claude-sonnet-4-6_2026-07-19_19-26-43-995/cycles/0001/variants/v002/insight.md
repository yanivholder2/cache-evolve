# Variant v002 Insight

## Helped

- **metadata-size**: Replacing `Option<usize>` prev/next fields (16 bytes each on a 64-bit target due to alignment padding) with `u32` sentinels (4 bytes each) halved `Node<K,V>` size; converting `head`/`tail` on `LruCache` from `Option<usize>` to `u32` shrank the cache struct by 24 bytes as well.
- **per-op-time**: The denser node layout packs more nodes per cache line during linked-list traversal in `get`, driving the p50 latency to the benchmark's timer-resolution floor.

## Hurt

- **per-op-time**: The `put` p50 crept up slightly; the eviction path now performs explicit `as u32` / `as usize` casts at every link-field read and write, adding marginal cost on the insertion and slot-reuse code paths.

## Takeaway

Sentinel-based `u32` links are a direct structural win for metadata footprint: the hypothesis's predicted 48 → 24 byte node reduction landed exactly. The `get` latency improvement is consistent with better cache-line utilization (more nodes per line), but the 0 ns p50 result is at the timer-resolution floor, so the magnitude should not be taken literally. The slight `put` regression suggests the cast overhead is real, even if small.
