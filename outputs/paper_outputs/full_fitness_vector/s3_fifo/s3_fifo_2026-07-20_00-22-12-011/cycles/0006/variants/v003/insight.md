# Variant v003 Insight

## Helped

(none)

## Hurt

- per-op-time: Splitting `resident_map` into `data` and `freq_map` meant every mutation path (`put`, `evict_main`, `evict_small`) now writes to two hashmaps instead of one, which drove p99 and put latencies higher even though get p50 was unchanged — the hot `get()` cache-line benefit the hypothesis predicted did not materialise.
- metadata-size: Adding a second `HashMap` field grew the `S3Fifo` struct from 208 to 256 bytes; the per-entry `Entry` footprint was unaffected, but the struct itself became heavier than the parent's single-map layout.

## Takeaway

Splitting `resident_map<K,(V,u8)>` into parallel `data<K,V>` and `freq_map<K,u8>` maps failed to reduce `get()` p50 latency: the cache-line density improvement on the read path was offset (or masked) by the two-hashmap write overhead on every admission, eviction, and update, leaving latency flat or worse across the board. The layout variable — not the queue fraction — is responsible, since queue fraction was held constant from the parent; the result suggests that for `K=V=u64` at this capacity, the cost of synchronising two independent hashmaps on writes dominates any L1/L2 benefit from a narrower frequency map on reads.
