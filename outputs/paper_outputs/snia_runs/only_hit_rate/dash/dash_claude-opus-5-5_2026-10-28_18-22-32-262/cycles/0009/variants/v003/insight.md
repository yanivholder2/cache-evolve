# Variant v003 Insight

## Helped

- hit-rate (16K, `web_0`): This cell rose slightly, by an amount within noise. At 16K the stash is saturated and overflow spreads fairly evenly across stash buckets. Each sub-ring therefore wraps at about the rate a 1/N slice of the global ring would, and a returning key's lookup in its own sub-ring finds about what the segment-wide scan found. Partitioning is close to neutral here.

## Hurt

- hit-rate (128K, `hm_0`): This is the largest drop, though still small. With `RESCUE_STASH_VICTIMS`, a key leaves the cache only when its target and its probing bucket are both full. At 128K that condition is met unevenly across the segment, so out-of-cache evictions cluster in a few stash buckets. Those sub-rings wrap within `2 × bucket_size` local evictions and lose fingerprints that the shared FIFO would still have held. Meanwhile, slots in quiet sub-rings sit stale and give nothing back. Total capacity is unchanged, but each bucket's reach is now fixed instead of following where evictions actually happen.
- hit-rate (128K, `web_0`; 16K, `hm_0`): These cells dipped by noise-level amounts, consistent with the same loss of reach in the busiest buckets.
- per-op-time / metadata-size: These are not measured in this fitness vector. The claim that motivated the hypothesis, a smaller put-path scan, is still unverified. The partitioned design also adds a `get_index(key, stash_size)` hash on every `put` and on every out-of-cache eviction, plus `u16` heads.

## Takeaway

Giving each stash bucket its own ghost sub-ring keeps total ghost capacity but not its adaptivity. A global FIFO lends its slots to whichever buckets are evicting. Fixed sub-rings of width `2 × bucket_size` shorten reach in exactly the hot buckets where ghost credit pays off, and the rescue path makes evictions more skewed, especially in the larger cache. The hit-rate cost is small enough that the change could still pay off if per-op-time shows a real gain. Until that dimension is measured, this variant has only a cost and no confirmed benefit. A later attempt could keep the global FIFO order and add a per-bucket index or bloom filter, so lookups get cheaper without making reach local.
