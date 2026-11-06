# Variant v002 Insight

## Helped

- per-op-time (`put` p99): The hypothesis's main claim holds. `put` p99 recovered and is now below the parent's level. When target and probing are both full, the second stash hit now stays in stash instead of running `promote_to_target` and its demotion cascade. Target buckets therefore lose fewer residents to stash. That means fewer stash inserts, fewer stash evictions, and fewer full-bucket cascades on later `put`s. Items hit exactly twice never pay the cascade.
- hit-rate (131072 configurations, web_0 and hm_0): Both large-cache configurations moved up a little. Under the old rule, an item with two hits would evict a target resident that had already proven itself. The single deferral stops that and keeps the resident in place. hm_0 at the large size did not slip, which was the risk the hypothesis flagged after cycle 4 v001's unlimited deferral. So a single deferral does not trap items in stash the way unlimited deferral did.

## Hurt

- per-op-time (`get` p50/p99): `get` regressed slightly, and the tail regressed more than the median. Some of the cascade cost moved from the second stash hit to the third. That third hit is a `get` that still runs the full `remove_at` → `promote_to_target` → `demote_from_normal_bucket` chain. Items that keep getting hit also spend one more access on the stash path. Every non-first stash hit now reads two bucket sizes before branching.
- hit-rate (16384 configurations, web_0 and hm_0): Both small-cache configurations slipped a little, so the summary barely moved. Deferred items hold stash slots for longer. Small caches have more stash pressure. A deferred item can therefore be evicted from stash before its third hit, or it can push out other candidates still waiting for their second hit.

## Takeaway

Allowing exactly one deferral when both buckets are full trades `put` tail latency against `get` tail latency. It does not remove the cascade. The deferral shifts the both-full cascade to a later `get` and filters out items that are hit only twice. That fixes the `put` p99 regression without losing the overall hit-rate gain. The cost is a small hit-rate split by cache size: large caches gain from keeping residents in place, and small caches lose stash capacity to deferred items. The next lever is probably to make the deferral depend on stash pressure, or to make the forced promotion itself cheaper. Changing the number of deferrals is unlikely to help.
