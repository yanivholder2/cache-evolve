# Cross-Cycle Insights

_Projection of 1 completed cycle (0001). Nothing below has been repeated across cycles yet, so every item is provisional._

## Recurring wins

- (None confirmed across cycles yet.) Cycle 0001's best lead was fixing the stash's ordering so that items already reused outlast brand-new or never-hit items. Changing where demoted items enter the stash (at the MRU end) improved hit-rate on every trace and cache size.
- A change that only reorders lookups, without touching what gets admitted or evicted, left hit-rate exactly the same. Such changes can be stacked onto a winner without risking hit-rate.

## Recurring failures

- (None confirmed across cycles yet.) A fixed promotion gate (two hits before a stash item promotes) helped small, crowded caches. At large sizes it gained nothing or slightly hurt (web_0). A gate that ignores capacity looks fragile.
- The evaluation only measured hit-rate. That made the cost-focused variants unscorable, and every per-op-time and metadata-size claim is still unverified. A cost-only variant gives no signal until the evaluation measures cost.

## Open questions

- Do the stash-entry position (MRU demotion) and the promotion gate add up when combined, or overlap? Both protect reused items from one-hit arrivals.
- Does a promotion gate that scales with occupancy or capacity keep the small-cache gains without the large-cache regression?
- Does target-first lookup order actually lower get latency? This needs an evaluation that reports per-op-time and metadata-size.

## Per-policy notes

- Stash/two-tier baseline lineage: improvements come from how items move in and out of the stash, and the benefit depends on cache size. Fixes at the entry point (demotion position) help more at large sizes, where more items get demoted. Fixes at the exit point (promotion gate) help more at small sizes, where the normal tier is crowded.
- Current lineage head: v001 (MRU-end demotion), from cycle 0001.
