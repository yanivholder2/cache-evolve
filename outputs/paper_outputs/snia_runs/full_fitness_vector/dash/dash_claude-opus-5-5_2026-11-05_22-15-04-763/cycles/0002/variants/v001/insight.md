# Variant v001 Insight

## Helped

- per-op-time: On the first stash hit, `promotion_get` now writes `lfu_counter` through `get_items_mut()[item_idx]` and returns a reference in place. It no longer calls `get_from_position`, which under ClassicLRU removed the item and re-pushed it at the MRU end, shifting the whole bucket `Vec`. Dropping that shift (and the policy hook) cut the `get` median, which is the summary metric. The gain is small: the remaining cost is mostly the two-hit gate's second touch and the promote/demote cascade, and this change does not affect either. `put` tail latency also improved even though `put`'s code did not change. A plausible cause is that marked items no longer get pushed back to MRU, so they leave stash sooner and insert-time eviction scans see less churn. Benchmark noise is also possible.

## Hurt

- hit-rate: Hit rate dropped slightly on every trace and size. The item no longer moves to MRU on its first hit, so a marked item keeps its insertion-order position and can be evicted from the stash LRU end before its second hit arrives. The filter now works like 2Q's A1in FIFO window, which is stricter than v002's "refresh on first hit" window. The largest loss was at the larger cache size on hm_0, the same configuration that gained most from the two-hit gate in v002. That suggests some of hm_0's reused items have re-reference gaps that fit within one-and-a-bit stash residencies but not within one.
- per-op-time (get p99): The `get` tail did not improve and got slightly worse. That fits the view that tail latency comes from the second-hit promotion cascade, which this change left alone, and not from the first-hit `Vec` shift.

## Takeaway

The MRU move on the first stash hit is not essential to the two-hit gate. Removing it keeps almost all of v002's hit-rate gain, so the mark alone does most of the filtering. However, removing it buys only a small improvement in `get` p50 and nothing at the tail. The first-hit reorder was not the main source of v002's `get` regression. Further latency work should target the second-hit `remove_at` + `promote_to_target` + demotion cascade, not the first-hit path.
