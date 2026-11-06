# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005

## Winners

(none)

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants were built on cycle 0001's two-hit gate (v002) and all were evaluated: leave the item in place on the first stash hit (v001), send twice-hit items into free probing space (v002), and demote normal-tier victims to the stash MRU end (v003). Each one improved median `get` slightly, and none improved tail latency. There are no Winners. v003 came closest, with higher hit rate everywhere and slightly lower `get` p50, but its tail got slightly worse.

## Patterns

- The cohort settled one question: the gate's per-op cost is not on the first-hit path. Removing the first-hit MRU move (v001) helped p50 only a little, left p99 unchanged, and cost some hit rate. The tail cost comes from the second-hit promote/demote cascade and from lookup length.
- Changes that keep proven items alive longer helped hit rate. v002 did this by using free probing space instead of evicting a target resident. v003 did it by giving demoted items a full stash residency. v003's gain was largest on hm_0 at the large size, the same configuration that gained most from the gate in cycle 0001 and lost most in v001. That configuration is the main place where re-reference gaps decide the result.
- Tail latency got slightly worse in every variant, each for its own reason: the cascade (v001), a three-bucket lookup for items placed in probing (v002), and stash occupancy pressure on `put` (v003). Stash pressure is now the leading explanation for the `put` regression, which v003 made worse. v002 lowered demotions into stash and barely changed `put`, so the cause is not settled.
- v002 and v003 work on different parts of the policy. v002 changes where promotions land, and v003 changes where demotions land. Neither has been tried together with the other.

## Open questions

- Does combining v003 (MRU demotion) with v002 (probing overflow) stack the hit-rate gains? Would v002's fewer demotions offset the stash pressure v003 adds?
- Can the probing-overflow lookup penalty be removed, for example by changing the search order or moving items back into target once it has space, without giving up its hit-rate gain?
- Is the remaining `put` p99 regression caused by stash fullness, or is it benchmark noise? None of the variants has separated the two.

## Continuation

Take v003 (MRU demotion on the gate) as the new base. It is the strongest hit-rate result, and its only cost is a small tail regression. Next, try v003 combined with v002's probing overflow, and look for a way to shorten the lookup for items placed in probing. Drop the first-hit no-reorder idea (v001): it costs hit rate and does not help the tail. Aim latency work at the second-hit cascade and at stash occupancy.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants were evaluated. Two built on cycle 0002's v003 (two-hit gate plus MRU demotion): v001 reordered the lookup to check the normal tier before stash, and v002 added probing overflow. The third, v003, applied MRU demotion to the baseline with no gate. There are no Winners. v002 was the best result, with a very small hit-rate gain on every cell and better tails on both `put` and `get`. v003 showed that MRU demotion raises hit rate even without the gate, but it did not recover baseline latency.

## Patterns

- Lookup order is not the lever for `get` latency. Searching stash last (v001) left hit rate unchanged and made `get` p50 and p99 worse. Stash hits make up a large enough share of all hits that stash-first is the better order.
- The `get` slowdown follows stash-hit traffic, not the gate. Ungated MRU demotion (v003) reached the same slower `get` band as the gated variants. The likely cost is the stash-hit promotion chain (remove, promote, demote cascade), which runs more often when stash keeps items longer.
- Promotion placement and demotion placement are not independent hit-rate levers. Once demoted items land at stash MRU, overflowing promotions into probing (v002) adds very little retention. What overflow does deliver is fewer demotion cascades, which reversed most of the `put` p99 regression caused by stash pressure. This is the clearest evidence so far that cascade count drives the tail.
- MRU demotion's hit-rate gain is again largest on hm_0 at the large size, with or without the gate. That cell is consistently where retention changes decide the result.

## Open questions

- Can the stash-hit promotion path itself be made cheaper, for example by skipping or deferring the demotion cascade when it is not needed, without losing retention?
- With v002's fewer cascades, how far is the gated, MRU-demoting line from dominating the baseline? Is the remaining gap only `get` p50?
- Would probing overflow added to ungated MRU demotion (v003 here) recover enough tail latency to make that cheaper branch competitive?

## Continuation

Take v002 (gate + MRU demotion + probing overflow) as the new base. It has the best combined hit rate and tail profile so far. Aim latency work at the cost of each stash-hit promotion and at reducing demotion cascades further. Drop lookup-reordering ideas and stop treating the gate as the source of the `get` slowdown. Optionally try overflow on the ungated MRU branch as a cheaper alternative line.


### Cycle 0004

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
