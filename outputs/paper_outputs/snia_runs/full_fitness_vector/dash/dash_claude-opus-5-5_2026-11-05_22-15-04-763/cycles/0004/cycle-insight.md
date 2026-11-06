# Cycle 0004 Insight

## Cohort outcome

All three variants were evaluated, and each one tried to remove or avoid a demotion cascade. Two built on the cycle 0003 v002 line. v001 deferred the second-hit promotion when both normal buckets are full. v002 promoted target-hottest items into probing only when probing has room. The third, v003, added probing overflow to the ungated MRU line. Each variant gained a little hit rate, and each made at least one tail metric worse. There are no Winners.

## Patterns

- Cascade count is not the cost to minimize. Stash fill level and where the cascade cost lands matter more. Skipping a cascade without giving items another way out of stash (v001) kept deferred items in stash permanently, and `put` p99 got much worse. Skipping cascades elsewhere moved their cost rather than removing it: to `put` through fuller target buckets (v002), or to a later `get` through foreign residents in probing (v003). Cycle 0003's overflow helped because it cut cascades and also kept items flowing out of stash.
- "Don't evict a proven resident" works for hit rate. The probing room gate gained a little everywhere, most at the small size. Deferral and ungated overflow gained retention at the small size or held it, but deferral slipped slightly on hm_0 at the large size, which is again the cell that exposes retention loss.
- `get` p50 did not respond to anything that skips cascades. The remaining median gap to baseline is on the common hit path: the multi-bucket scans and stash checks. It is not in the rarer cascade paths.
- Items placed in probing pay a lookup cost: they are found last and can be evicted by a neighbour's promotion. On the ungated line this cost more `get` p99 than overflow saved on `put`, so the ungated line is not the cheaper route.

## Open questions

- Can a deferred promotion be bounded, for example by demoting or force-promoting after a limit, so stash stays drained without bringing the full cascade back?
- Which part of the common hit path holds the `get` p50 gap: the stash check, the scan of a full target, or the cost of touching an item to MRU?
- Does v002's probing room gate keep its hit-rate gain if the stash-promotion fallback it now triggers more often is made cheaper?

## Continuation

Keep cycle 0003 v002 as the base. v002's probing room gate is a cheap hit-rate gain worth carrying if its `put` cost can be fixed at the stash-promotion fallback. Drop the ungated overflow line and unbounded deferral. Any change that skips promotions must keep items flowing out of stash. Point `get` p50 work at the common hit path (scan cost, stash checks), not at cascades.
