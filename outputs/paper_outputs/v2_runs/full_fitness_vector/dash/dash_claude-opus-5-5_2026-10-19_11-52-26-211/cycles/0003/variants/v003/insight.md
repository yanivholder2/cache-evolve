# Variant v003 Insight

## Helped

- per-op-time: The `get` tail got shorter, as H03 predicted. With `PROTECT_NATIVE_NEIGHBOURS`, a hottest-in-target hit whose probing neighbour has a native LRU victim now ends in a single `get_from_position`. Before, it ran `remove_at` → `promote_to_probing` → `demote_from_normal_bucket` → a stash insert. Cutting that worst-case cascade lowers p99, while p50 stays the same because most gets never reach the `is_hottest` branch. The `put` p99 moved the other way, but `DashSegment::put` was not changed, so that movement is most likely run-to-run noise rather than an effect of this diff.

## Hurt

- hit-rate: Hit rate fell slightly, to or below the parent's. That meets the hypothesis's own falsification condition. The victim that `probing_promotion_allowed` protects sits at the probing bucket's LRU end, so by construction it is the coldest resident there. Blocking the promotion protects that cold native key and pins the proven-hot key in its target. There it takes a slot that would otherwise cushion its colder target-mates, so they spill to the stash sooner. In practice the rule swaps a cold eviction in the neighbour for a warmer eviction at home.

(metadata-size did not change. The rule only reads existing bucket state through `eviction_candidate_index`, as the hypothesis required: no new fields, and `Item` stays at 40 bytes.)

## Takeaway

Using ownership (native vs foreign) to filter the target→probing edge picks the wrong signal. Being native says nothing about recency, and the victim being protected is always the probing bucket's LRU item. So on workload-a, letting hot keys overflow into the neighbour is worth more than keeping cold native keys there. The result falsifies the "probing-tier pollution" idea that cycles 1 and 2 put forward. The rule's one clear benefit is a shorter `get` tail. That suggests the cost of the promotion cascade, not its admission choices, is what a later change could cut.
