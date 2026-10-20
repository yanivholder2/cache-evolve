# Variant v003 Insight

## Helped

- **hit-rate**: The bounded second-chance scan in `eviction_candidate_index` preferentially evicts items whose `lfu_counter` is still 0 (inserted but never accessed) rather than blindly evicting the positional tail, so Zipfian working-set items that have been touched at least once survive eviction rounds they would have lost under strict positional-LRU.
- **metadata-size**: Replacing `Instant` with `lfu_counter: usize` + `_pad: u64` removes the 16-byte wall-clock timestamp and keeps the struct at 32 bytes, saving 8 bytes per item compared to the 40-byte baseline layout.

## Hurt

- **per-op-time**: The 3-swap promotion loop unconditionally writes up to three positions in the items vector on every cache hit (vs. the baseline's cheaper positional reorder), increasing write pressure per `get` and pushing the p50 get latency above the baseline floor.

## Takeaway

Combining a write-once binary "ever-accessed" flag with a shallow second-chance tail scan is a highly effective eviction-quality improvement: the flag costs one write per item lifetime (not per access), and the scan can skip past recently-touched items to find a cold newcomer to evict instead. The tradeoff is that the same access path that sets the flag also runs a 3-swap promotion loop, which raises per-get write cost enough to measurably hurt latency — suggesting the hit-rate gain and the latency regression should be decoupled in future variants by pairing the second-chance scan with a cheaper (or no-op) position-update strategy.
