# Variant v002 Insight

## Helped

- hit-rate: Bumping `lfu_counter` on every hit and evicting the minimum-frequency item makes proven-hot keys resist eviction, so each bucket retains its reusable working set instead of discarding it on recency alone.

## Hurt

- per-op-time: `eviction_candidate_index` now does an O(bucket_size) `min_by_key` scan on every evicting put instead of delegating to the policy's cheaper default — a cost not re-measured here but inherent to scanning for the least-frequent victim.

## Takeaway

Repurposing the dormant `lfu_counter` as a live hit count — without touching the `EvictionPolicyKind` enum or its JNI ordinal mapping — converts intra-bucket eviction from pure LRU to frequency-first with recency as the front-biased tie-break, and that frequency signal is what lifts hit rate on this workload. The win comes entirely from victim selection inside the existing bucket; the resident set, bucket assignment, and capacity invariants are unchanged.
