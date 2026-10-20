# Variant v001 Insight

## Hurt

- hit-rate: Removing the `freq_bucket` field, `quantize_frequency()` function, and all frequency update logic reverted the cache to pure LRU, eliminating protection for frequently-accessed items and allowing the eviction path to discard hot entries based solely on recency.

## Takeaway

Pure LRU eviction is insufficient for this workload. By stripping frequency discrimination entirely—removing quantized buckets, access counting, and frequency-aware victim selection—the cache degraded from 0.7334 to 0.6652 hit-rate, confirming that the parent variant's frequency tracking mechanism is essential. Recency alone permits the cache to evict a rarely-used item that was accessed slightly more recently than a frequently-used item, causing unnecessary misses. The result validates that frequency discrimination, even in coarse 3-bit bucketed form, is a critical component of cache performance on this workload.
