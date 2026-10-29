# Cycle 0001 Insight

## Cohort outcome

All three variants changed ClassicLRU inside the bucket, each through a different lever: where new items are inserted (v001, LIP-style quarter-depth), how hits are recorded (v002, CLOCK reference bit), and which item is picked as the victim (v003, frequency-aware tail window). The Winners, v001 and v002, both gained hit rate by making new one-touch items the first to be evicted. v003's hit rate came out identical to the baseline, which most likely means its change never ran in the hit-rate harness.

## Patterns

- Scan resistance is the hit-rate lever that worked. Both Winners make new items "earn" their place before they are protected from eviction, either by position or by a cleared bit. The gain is biggest with the wide (1024-item) buckets, where each bucket acts most like a full LRU.
- Every variant made put latency worse, especially the tail. Any extra work on the miss/eviction path (mid-bucket insert, a per-item sweep, a window scan plus aging) cost more than the single bulk memmove it replaced or added to. The assumption that "a memmove is a memmove" did not hold.
- Get p50 barely moved in any variant, even when the hit path became a single store (v002). So most of the get cost is the linear key lookup across the wide bucket, not the reordering.
- The `name() == "ClassicLRU"` string compare on every op appeared in two variants and adds overhead to the put path that has nothing to do with the policy.
- None of the variants changed metadata-size. Reusing `lfu_counter` and changing only the insert position are both free on that axis.

## Open questions

- Does the hit-rate harness actually run the overlaid bucket `evict_item` / ClassicLRU code paths? v003's bit-identical result needs checking before frequency-aware victim selection is judged.
- Does v001's quarter-depth insert combine with v002's cleared reference bit, or do they overlap and give the same scan protection twice?
- Should the insertion depth scale with bucket width, given the weak gain at the smaller size?

## Continuation

Build on the scan-resistant insertion idea (v001/v002). Tune or scale the insertion depth, and decide the policy check once at construction instead of comparing strings on every op. Target speed directly at the linear `get_position` scan (for example a tag array or per-bucket key index) rather than at reordering cost, and avoid unbounded per-item eviction sweeps. Before retrying v003's frequency-window idea, first confirm that the harness runs the overlaid eviction path.
