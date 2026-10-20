# Variant v002 Insight

## Helped

- (none) — capacity_bound and slot_legality held (probationary front-insert and routing untouched), but no scored dimension improved.

## Hurt

- hit-rate: The elite tier (`lfu_counter` bumped to 2 in `get_from_position`, and eviction preferring counter-1 victims) made the all-protected fallback discard still-hot once-protected keys to keep twice-hot keys longer; on this trace those evicted keys are re-referenced, so retention narrowed and hit-rate fell below the parent's flat protected-LRU eviction.

## Takeaway

The "elite" third tier was net-negative for this trace. The hypothesis bet that protecting twice-hot keys one extra eviction round would harden retention; instead, reworking the all-protected fallback in `eviction_candidate_index` to evict the *least*-protected resident (preferring a `lfu_counter == 1` victim over a `== 2` victim) means merely-once-hot keys are now sacrificed first — and on a "once-hot immortality" workload those keys are still being re-referenced, so each one discarded early costs more future hits than the shielded elite key saves. Discriminating among already-protected keys also breaks the parent's clean prefix/suffix ordering: the min-counter `position` scan evicts the LRU-end *within the counter-1 tier*, not the true bucket LRU, so the victim is chosen on tier before recency and can drop a counter-1 key that is hotter by recency than a retained elite key.
