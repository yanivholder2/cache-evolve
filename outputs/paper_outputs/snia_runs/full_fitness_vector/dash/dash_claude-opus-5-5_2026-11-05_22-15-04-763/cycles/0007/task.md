# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants followed up on cycle 0005's open questions. v001 allowed the deferral only when the stash bucket has free slots (parent 0005 v002). v002 skipped the MRU reorder on near-MRU normal-tier hits (parent 0004 v002). v003 fused the stash-promotion chain into one in-place rotate (parent 0005 v002). There are no Winners. Each variant moved the metric it aimed at only slightly, or not at all, and each paid for it on a different axis.

## Patterns

- A gate based on how full the stash bucket is does nothing in practice. At steady state the stash is almost always full, so v001 hardly ever deferred and behaved much like 0004 v002 again. It lost the `put` p99 gain, and the hit-rate change only moved from the large cache to the small cache. Fill level cannot tell a harmful deferral from a harmless one.
- The MRU shift is not where the `get` p50 gap comes from. v002 kept hit rate bit-identical and left p50 flat. Scan order, scan direction, cascades and now the MRU shift have all been ruled out, which leaves the stash probe on every normal-tier hit as the last suspect on the common hit path.
- Moving data within buckets accounts for part of the `get` tail. v003's fused rotate kept the same policy and made `get` p99 better. The gain is small, though, and `put` p99 got worse by more. The policy is identical, so the regression comes from cost or code layout, probably from the second call site or from inlining.
- Both cost-only variants (v002, v003) made a p99 worse on a path they were not expected to touch. This matches cycle 0005's warning about noise. It may also come from codegen or inlining side effects. Small tail changes need a rerun before anyone draws conclusions from them.

## Open questions

- If the stash probe holds the `get` p50 gap, how can it be made cheaper without reordering the lookup? Cycle 0003 v001 already checked stash last, and `get` got worse because stash hits make up a large share of all hits. Possible routes: a cheaper stash membership check, or skipping the stash probe for keys known not to be there.
- If `swap_promote` is limited to the third-hit path or marked non-inline, does its `get` p99 gain stay without the `put` regression?
- Is there a better signal for the deferral than fill level, for example whether the stash slot that would be evicted holds an item still waiting for its second hit?

## Continuation

Stay on cycle 0005 v002, or on 0004 v002 for p50 work. Drop the fill-level stash gate and the near-MRU reorder skip. For `get` p50, go after the cost of the stash probe itself. Do not move stash to the end of the lookup again, since cycle 0003 showed that is worse. For `get` p99, carry the fused rotate only if it is kept to the third-hit path or isolated from inlining, and rerun any result that rests on a small tail change.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
