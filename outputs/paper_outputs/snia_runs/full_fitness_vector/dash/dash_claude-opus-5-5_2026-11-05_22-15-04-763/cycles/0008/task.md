# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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

Exactly one file: `cycle-insight.md` in the output dir.
