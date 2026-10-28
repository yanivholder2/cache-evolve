# Variant v001 Insight

## Helped

- hit-rate: Items pushed out of a normal bucket now go to the stash MRU end (`STASH_DEMOTION_POS = Back`) and no longer to its LRU end, so the next stash victim is an older or never-hit item and not one that just earned a normal-bucket slot. Hit-rate rose on both MSR traces at both cache sizes. The largest gain was on `hm_0` at the large size, where more demotions happen per stash lifetime. The smallest gain was on `web_0` at the large size.

## Hurt

(none measured — v001's fitness vector reports only hit-rate, so the hypothesis's claim that metadata-size and per-op-time stay the same was not checked. The code change is one extra `InsertPosition` argument and a constant, with no new per-item state or extra bucket operations, so neither is expected to have moved.)

## Takeaway

In the baseline, a demoted item landed in the stash *behind* (closer to eviction than) brand-new `put` arrivals. That made the normal→stash demotion path act as an anti-recency rule: proven-reused items were evicted ahead of one-hit items. Moving the stash insert to the Back fixes this without touching promotion placement (stash→target and target→probing still use Front). This suggests that where demoted items land in the stash affects hit-rate on its own, separately from where promoted items land. Stash ordering, ranking demoted items against fresh inserts, is a lever worth exploring further, for example a position between Front and Back, or ordering by reuse count.
