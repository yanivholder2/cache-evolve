# Variant v003 Insight

## Helped

- hit-rate (held, did not improve): Moving entries into a slab with an FxHash key→slot index and lazy-deleted ghost membership left graduation, demotion, ghost re-admission, and second-chance eviction order unchanged. Hit-rate matches the parent exactly, which is evidence that the rewrite preserved policy behaviour.

## Hurt

(No measured dimension moved for the worse. v003's fitness vector reports only `hit-rate`, so `per-op-time` and `metadata-size` cannot be compared with the parent.)

## Takeaway

Replacing the parent's linear scans with a hash-indexed slab (`index: HashMap<K, u32>`, `ghost_set` with sequence-number lazy deletion, `VecDeque<u32>` queues) is behaviour-neutral on this trace, so it is safe to reuse as a base for later hit-rate variants. The variant's main claim is unverified because `per-op-time` was not reported for v003: O(1) `get` and O(1) ghost lookup on `put` should lower get p50 and put p99. The structure also adds per-entry overhead that the parent did not have: an index map entry, a ghost map entry with a `u64` sequence, and stale ghost-deque slots bounded at about 2×`main_size`. `metadata-size` needs to be re-measured before v003 is judged on the trade-off it was built to test.
