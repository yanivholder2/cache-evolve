# Cycle 0001 Insight

## Cohort outcome

All three variants branched from baseline and each changed one thing: v001 changed where demoted items land in the stash, v002 added a two-hit gate before stash items promote, and v003 changed the lookup order as a cost-only control. The Winner is v001, which moved demoted items to the stash MRU end. It improved hit-rate on every trace and cache size. v002 also gained hit-rate in most cases, but not all.

## Patterns

- The baseline stash was ordering items badly: items that had already been reused were evicted before brand-new or never-hit items. Both hit-rate variants fixed part of this. v001 changed where items enter the stash. v002 changed what it takes for an item to leave the stash. The two are close to independent and could be combined.
- How much each change helped depended on cache size in different ways. v001 helped more at the large cache size, where more items get demoted. v002 helped most at the small size, where the normal tier is crowded, and gave nothing or slightly worse on web_0 at the large size.
- The whole cohort was scored on hit-rate only. per-op-time and metadata-size were never measured. All three variants' claims about cost (no change for v001, cheaper for v002 and v003) are unverified. v003, whose only purpose was per-op-time, produced no usable signal. It did confirm that the lookup reorder leaves hit-rate exactly unchanged.

## Open questions

- Do v001's demotion position and v002's two-hit gate add up when combined, or do they overlap? Both protect reused items from one-hit arrivals.
- Should the promotion gate scale with capacity or occupancy instead of staying fixed at two hits, so it stops costing hit-rate in roomy caches?
- Does the target-first lookup order actually lower get latency? This needs an evaluation that reports per-op-time.

## Continuation

Build on v001. Next, test it combined with the two-hit gate, and include a variant where the gate scales with occupancy, to see whether the small-cache gains hold without the large-cache regression. Make sure the evaluation reports per-op-time and metadata-size, then re-test the lookup reorder (v003). Its hit-rate result is identical to baseline, so it can be added to any winner without risk. Do not count it as a cost win until latency is measured.
