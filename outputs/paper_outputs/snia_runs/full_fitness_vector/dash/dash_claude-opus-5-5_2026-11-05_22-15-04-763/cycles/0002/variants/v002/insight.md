# Variant v002 Insight

## Helped

- hit-rate: On a second stash hit with a full target bucket, the item now goes into free probing space when there is any. The target's LRU resident is no longer pushed into stash with its mark cleared, so the normal tier keeps one more item with proven reuse. The gain is small but shows on every trace and size. That fits the hypothesis's "hold or rise" prediction: the change only adds capacity use and never evicts anything new.
- per-op-time (get p50): Most second-hit promotions now skip the full `promote_to_target` → `demote_from_normal_bucket` → stash insert/evict chain and do a single `put_at(Slot::Probing)` with no eviction. That makes the typical promoting `get` a bit cheaper, and this metric is the dimension's summary.

## Hurt

- per-op-time (get p99): The tail got worse, against the hypothesis's claim that p99 would gain the most. Items placed by balanced promotion now live in their probing bucket. `promotion_get` searches stash, then target, then probing, so every later hit on such an item scans all three buckets before it matches. The first version scanned two. That puts a slower lookup on exactly the reused items. The three extra `size()`/`get_max_size()` reads on every second hit also add a small fixed cost.
- per-op-time (put): Movement was mixed and small: p50 slightly up, p99 slightly down. So cutting stash pressure did not clearly fix the parent's `put` slowdown. Either the cascade was not the main source of stash pressure, or the effect is below benchmark noise.

## Takeaway

Using free probing space as an overflow for twice-hit stash items is a safe win for hit rate and median `get`, because it adds no evictions. Its cost lands in the tail: overflowed items sit in the last bucket the lookup checks. A follow-up should keep the balanced placement but cut that extra lookup cost, for example by checking probing before stash for keys known to have overflowed, or by moving an item back into target once target has space. The parent's `put` regression still needs a separate explanation, since lowering demotions into stash barely changed it.
