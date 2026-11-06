# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three changes to how Dash moves items between stash and the normal tier were proposed: demote to the stash MRU end (H01), require two stash hits before promotion (H02), and balance promotions between target and probing buckets (H03). Only the two-hit gate (v002) got evaluated. The other two variants (v001, v003) failed because the simulator build hit HTTP 429 rate limits from Maven/Gradle, so the failure was infrastructure, not the policy. There are no Winners.

## Patterns

- The cohort's working theory is that the small normal tier gets flushed by weakly re-referenced items. v002 is the only result, and it supports that theory: gating admission to the protected tier raised hit rate on every trace and size tested. The gain was largest at the larger size, on hm_0 most of all.
- The hit-rate gain cost latency. `get` got slower at both p50 and p99, which contradicts the hypothesis's guess that a first stash hit would be cheaper. `put` slowed too, possibly because stash pressure rises when items stay in stash longer. Benchmark noise has not been ruled out as the cause of the `put` slowdown.
- Reusing the unused `lfu_counter` field kept metadata size unchanged. That makes it a good pattern to follow for cheap per-item state.
- H01 and H03 are still untested. Their failures say nothing about the ideas themselves.

## Open questions

- Is the `put` slowdown real (from stash buckets filling and evicting more often) or benchmark noise?
- Can the two-hit gate keep its hit-rate gain without the extra MRU move on the first stash hit?
- Would H01 (MRU demotion) or H03 (balanced promotion) stack with the gate? H03 claims it lowers cascade cost, which could offset the gate's latency.

## Continuation

Re-run H01 and H03 unchanged, since their ideas were never measured. Build on v002's two-hit gate: try a variant that leaves the item in place on the first stash hit (just set the counter, no MRU move), and try combining the gate with H03's balanced promotion to win back per-op time. Keep the gate, and do not trade away its hit-rate gain for latency.


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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Two variants made cost-only changes to the `get` p50 gap on 0004 v002: v001 hashes each key once per lookup, and v002 adds a stash presence mask. The third, v003, changed the deferral to depend on whether the stash LRU item is marked (on 0005 v002). v002 is the Winner. It is the first variant to move `get` p50 with bit-identical hit rate, but it costs a lot of `put` and `get` p99.

## Patterns

- The stash probe is now confirmed as the source of the `get` p50 gap. Repeated hashing inside the segment is ruled out: v001 kept hit rate identical and p50 stayed flat. Skipping the stash scan with a mask test lowered p50. Every other common-path suspect from earlier cycles is now eliminated.
- v002's cost is mask upkeep, not the idea itself. Recomputing the mask on every stash removal re-hashes the whole bucket. Stash removals are on the common put/eviction path, so the cost shows up in both `put` percentiles and in the `get` tail. This is the same lesson as earlier cycles in a new form: work moved onto stash churn lands on the hot path.
- Removing extra hashes (v001) trims the stash-promotion tail a little at no hit-rate cost. That makes it a cheap change to stack on others, not a p50 lever.
- The deferral line is close to exhausted. After fill level (0006), the marked-LRU signal (v003) also failed to recover the small-cache hit-rate loss. All it did was change how often the both-full cascade runs, which moved cost between `get` and `put` and gave back some large-cache hit rate. Signals local to a single stash bucket do not separate a harmful deferral from a harmless one.
- Unexplained `put` p99 drift on a path the change doesn't touch (v001) appeared again. Small tail changes still need a rerun.

## Open questions

- Can the mask support O(1) removal, for example with small per-bit counters or with mask bits stored next to the stash bucket metadata, while keeping the p50 gain?
- How much of the remaining `get` tail comes from false positives rather than from the refresh on promotion? Would a wider or two-bit fingerprint change that?
- Does the Dash-level segment hash, which v001 left alone, account for any remaining p50 gap once the stash probe is cheap?

## Continuation

Build on 0007 v002 and make mask maintenance cheap. Removals must avoid re-hashing the bucket and avoid an extra cache line, using counting bits or metadata stored next to the bucket. Folding in v001's single hash is a cheap addition. Drop deferral-gating signals taken from stash bucket state. Return to the deferral line only with a new signal type, and not before the mask's tail cost is fixed.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
