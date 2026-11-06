# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006

## Winners

(none)

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004 v002 (gate + MRU demotion + probing overflow + probing room gate), and each one tested a separate thread. v001 reversed the in-bucket scan (cost only). v002 allowed exactly one deferred promotion when target and probing are both full. v003 sent unreused demotions to the stash LRU end. There are no Winners. v002 was the only one to fix what it targeted: it brought back `put` p99, but the cost moved to `get` tail and to small-cache hit rate.

## Patterns

- Two levers are now ruled out. Scan direction does not affect `get` p50 (v001 kept hit rate identical, so this was a clean test), because the buckets that miss still need full scans. A liveness signal taken from a single normal-tier stay does not work either (v003 lost hit rate everywhere, most on hm_0 at the large size). MRU demotion for every demoted item stays.
- Deferral moves the cascade but does not remove it. The single deferral took the both-full cascade off `put` and put it on a later `get`. This matches cycle 0004: skipping a cascade just moves its cost somewhere else. The difference is that a limited deferral keeps stash draining, so `put` recovered rather than getting worse.
- Cache size decides whether stash residency helps. When an item stays in stash longer (deferral), the large cache gains because proven residents stay in place. The small cache loses because deferred items take up stash slots. hm_0 at the large size again rewards anything that keeps long-gap reusers.
- Every variant made at least one p99 slightly worse, including v001, whose change cannot add work to `put`. Small tail moves of this size may be partly noise.

## Open questions

- Which part of the common `get` hit path holds the p50 gap? The two remaining suspects are how many buckets each hit probes (stash checked first on every lookup) and the shift cost when an item moves to MRU.
- Can the deferral depend on stash pressure (defer only when stash has headroom) so the small-cache hit-rate loss goes away while the `put` p99 gain stays?
- Can the forced promotion on the third hit be made cheaper, or moved off the `get` path?

## Continuation

Take cycle 0005 v002 (single deferral) as the base if the next hypotheses target its `get` tail or its small-cache loss, for example a deferral gated on stash pressure. Otherwise stay on cycle 0004 v002. Aim `get` p50 work at how many buckets each hit probes and at the cost of moving an item to MRU. Drop scan-direction changes and any demotion placement that relies on reuse within a single normal-tier stay.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
