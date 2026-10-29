# Variant v001 Insight

## Helped

- hit-rate (large-cache cells): Both 131072-size cells improved, and hm_0 improved the most. That is where the parent lost hits. With a third stash hit required to enter a *full* target, a 2-hit stash key can no longer push out a reused target resident. This recovers part of the protection the parent's victim comparison used to give, without reading the victim's counter. The large-cache loss the parent left open has been partly won back.
- per-op-time (put median and tail): Puts got much cheaper at both percentiles. This supports the parent's put-p99 theory. Deferring promotions into full targets means fewer forced `demote_from_normal_bucket` → stash inserts, so the stash churns less and puts land in less crowded stash buckets. `put` itself was not edited, so this effect is indirect.
- per-op-time (get tail): Get p99 dropped noticeably. A hit that reaches the threshold against a full target now often ends as a cheap `get_from_position` touch. Before, it went through `remove_at` + `promote_to_target` + demotion cascade, so the slow promotion path runs less often.
- per-op-time (get median, the summary statistic): Get p50 fell slightly. The target and probing hit paths are byte-for-byte unchanged, and `is_full()` runs only after `count >= PROMOTE_THRESHOLD`. The likely cause is less stash/bucket churn, or noise. The guard itself did not save time.

## Hurt

- hit-rate (small-cache cells): Both 16384-size cells slipped very slightly. In a small cache, stash residence is short. A key that would have promoted after two hits can now be evicted from stash before its third hit, so the guard costs some useful promotions there. The summary hit-rate is roughly flat because the large-cache gains and small-cache losses mostly cancel.

## Takeaway

An occupancy-only guard (one extra stash hit, required only when the target is full) gets most of the effect of a victim comparison while adding nothing to the common hit path. It helps large caches, where a stable reused working set is worth protecting. The largest win is not hit-rate but latency, put latency in particular: cutting forced demotions into stash lowers stash churn across the cache. The small-cache slip suggests the extra-hit requirement should scale with cache or stash size rather than be a fixed 3. One option is to apply it only when the stash is large enough that a key can plausibly collect three hits.
