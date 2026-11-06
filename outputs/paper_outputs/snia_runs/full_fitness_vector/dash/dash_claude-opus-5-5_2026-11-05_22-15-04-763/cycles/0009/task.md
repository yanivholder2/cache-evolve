# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on 0007 v002's stash presence mask and went after its upkeep cost in different ways. v001 used a coarse counting mask, v002 used per-slot fingerprints on the stash bucket, and v003 swapped in a cheap fmix64-style hasher in place of SipHash. v003 is the Winner. It improved every per-op percentile and kept hit rate flat within noise. Both structural fixes repaired `put` but gave back the `get` p50 gain.

## Patterns

- Hashing cost was a first-order term all along. 0007 v001 cut the number of hash calls and p50 did not move. v003 made each call cheaper and p50 dropped. Making each re-hash cheaper also shrank most of the mask-refresh penalty on `put` and on the `get` tail, without changing the refresh structure.
- Mask upkeep affects only `put`. Both O(1)-maintenance variants (v001, v002) fully recovered `put`. Neither recovered `get` p99, and neither kept p50. Making the update cheap does nothing for reads.
- The read fast path depends on a dense, separate mask word with low false positives. v001 shrank the filter, false positives rose, and the filter ended up costing more than no filter at all. v002 moved presence tracking onto the bucket's own line as a byte scan, which was slower than one AND against a compact mask word even though it was more exact. The "extra cache line" that 0007 suspected turned out to be cheap for reads.
- The `get` tail sits on the stash-hit/promotion path, not in the mask refresh. v002 removed the refresh and the tail still got worse, because adding work to the item shift on promotion lands directly on that path.

## Open questions

- With cheap hashing in place, does a structural O(1) mask update still pay off? Any such update must keep the dense 64-bit (or wider) word so the false-positive rate does not rise.
- Is there more p50 left in the common-path hash, for example the segment-level selection, or in the remaining false positives on the mask test?
- Would the cheap hasher also change the trade-offs on the deferral line (0005 v002 lineage), since that line's costs were partly re-hash-driven?

## Continuation

Build on 0008 v003 (fast hasher plus the 0007 v002 mask). Keep the separate dense mask word for normal-hit rejection. Drop shrunken counting masks and in-bucket fingerprint scans as the reject path. If `put` needs to go lower, try O(1) mask maintenance that keeps at least 64-slot resolution. Also look at the cost of the stash-promotion shift for the `get` tail, without adding work to that shift.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
