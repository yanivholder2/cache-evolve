# Variant v003 Insight

## Helped

- **hit-rate**: LFU-minimum eviction (scanning `lfu_counter` to find the least-frequently-used victim) retains high-frequency items that positional LRU would have discarded, and MRU-end insertion of stash-promoted items shields them from being immediately re-evicted.
- **metadata-size**: Removing the `timestamp: Instant` field from `Item<K, V>` shrank the struct from 40 to 24 bytes, eliminating the 16-byte wall-clock field that was only used by the now-stub `TimestampLRU` path.

## Hurt

- **per-op-time**: The linear scan over all bucket items to find the minimum `lfu_counter` in `ClassicLRU::eviction_candidate_index` replaced the baseline's O(1) constant-zero return, adding work to every eviction path and increasing median latency on both get and put operations.

## Takeaway

LFU-minimum eviction and MRU-end insertion together deliver a meaningful hit-rate gain over the baseline without any frequency-gated inter-tier promotion, confirming that better eviction selection is itself a potent lever — but the O(n) victim scan has a real per-op cost, suggesting that the throughput benefit of the timestamp removal is outweighed by the scan overhead introduced in the same eviction path.
